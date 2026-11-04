//! The window's content with the todo being dragged drawn at the cursor.
//!
//! The system draws the mouse pointer straight to the screen, while the app
//! shows a frame only after it is rendered and composited, a few frames
//! later. Anything the app draws at the cursor therefore trails the
//! pointer. So while a todo is dragged the system pointer is hidden and this
//! widget draws a pointer of its own with the todo under it, in the same
//! frame: the two always move together.
//!
//! It wraps the whole page rather than sitting on a layer above it: a layer
//! that sets the pointer would hide the cursor from the layers under it, and
//! the drop targets there would no longer see it. Between mouse moves it
//! only asks for a redraw, so a drag never rebuilds the window.

use std::sync::LazyLock;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell, mouse, overlay, renderer};
use iced::widget::svg;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// Where the todo sits relative to the pointer's tip: under the pointer's
/// tail, so the pointer looks like it holds the todo.
const CARD_OFFSET: Vector = Vector::new(8.0, 12.0);

/// The pointer's tip in the pointer image, which goes on the cursor.
const POINTER_TIP: Vector = Vector::new(1.0, 1.0);

/// An arrow like the system's, white with a dark outline so it shows on
/// light and dark themes.
static POINTER: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="22" viewBox="0 0 16 22">
<path d="M1 1 L1 17.5 L5 13.8 L7.9 20.4 L10.6 19.2 L7.8 12.8 L13.2 12.8 Z"
 fill="#ffffff" stroke="#111111" stroke-width="1.1" stroke-linejoin="round"/>
</svg>"##
            .as_slice(),
    )
});

/// `content` with `card`, when given, following the cursor.
pub struct DragLayer<'a, Message> {
    content: Element<'a, Message>,
    /// The card and the pointer drawn with it.
    dragged: Option<[Element<'a, Message>; 2]>,
}

pub fn drag_layer<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    card: Option<Element<'a, Message>>,
) -> DragLayer<'a, Message> {
    let dragged = card.map(|card| {
        let pointer = svg(POINTER.clone()).width(16).height(22).into();
        [card, pointer]
    });
    DragLayer {
        content: content.into(),
        dragged,
    }
}

impl<Message> DragLayer<'_, Message> {
    fn elements(&self) -> impl Iterator<Item = &Element<'_, Message>> {
        std::iter::once(&self.content).chain(self.dragged.iter().flatten())
    }
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, iced::Renderer> for DragLayer<'a, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn children(&self) -> Vec<Tree> {
        self.elements().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements: Vec<&Element<'_, Message>> = self.elements().collect();
        tree.diff_children(&elements);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let content = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = content.size();
        let mut nodes = vec![content];
        for (element, tree) in self.dragged.iter_mut().flatten().zip(&mut tree.children[1..]) {
            nodes.push(element.as_widget_mut().layout(tree, renderer, &limits.loose()));
        }
        layout::Node::with_children(size, nodes)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut layouts = layout.children();
        let Some(content) = layouts.next() else {
            return;
        };
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, content, cursor, viewport);

        let (Some([card, pointer]), Some(position)) = (&self.dragged, cursor.position()) else {
            return;
        };
        let (Some(card_layout), Some(pointer_layout)) = (layouts.next(), layouts.next()) else {
            return;
        };
        let at = |offset: Vector| position + offset;
        // Each goes on a new layer, drawn over every layer before it: the
        // page's scrollables and clipped areas draw on layers of their own,
        // which would cover anything drawn here on the page's layer. Within a
        // layer text goes over images, so the pointer gets its own layer to
        // sit over the card's text, like the system's.
        for (element, tree, layout, place) in [
            (card, &tree.children[1], card_layout, at(CARD_OFFSET)),
            (
                pointer,
                &tree.children[2],
                pointer_layout,
                at(Vector::ZERO) - POINTER_TIP,
            ),
        ] {
            renderer.with_layer(*viewport, |renderer| {
                draw_at(element, tree, renderer, theme, style, layout, place, viewport);
            });
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let Some(content) = layout.children().next() else {
            return;
        };
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if self.dragged.is_some()
            && let Event::Mouse(mouse::Event::CursorMoved { .. }) = event
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.dragged.is_some() && cursor.is_over(layout.bounds()) {
            return mouse::Interaction::Hidden;
        }
        let Some(content) = layout.children().next() else {
            return mouse::Interaction::None;
        };
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], content, cursor, viewport, renderer)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        if let Some(content) = layout.children().next() {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], content, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let content = layout.children().next()?;
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], content, renderer, viewport, translation)
    }
}

/// Draws `element`, laid out at the window's corner, with its corner at
/// `place`.
#[allow(clippy::too_many_arguments)]
fn draw_at<Message>(
    element: &Element<'_, Message>,
    tree: &Tree,
    renderer: &mut iced::Renderer,
    theme: &iced::Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    place: Point,
    viewport: &Rectangle,
) {
    let shift = place - layout.bounds().position();
    // Move the viewport the other way, so nothing is culled as off screen.
    let viewport = Rectangle {
        x: viewport.x - shift.x,
        y: viewport.y - shift.y,
        ..*viewport
    };
    renderer.with_translation(shift, |renderer| {
        element.as_widget().draw(
            tree,
            renderer,
            theme,
            style,
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );
    });
}

impl<'a, Message: 'a> From<DragLayer<'a, Message>> for Element<'a, Message> {
    fn from(widget: DragLayer<'a, Message>) -> Self {
        Element::new(widget)
    }
}
