//! A popover: content that floats over everything else, anchored below a
//! widget, without moving the layout. A click outside closes it.
//!
//! iced 0.14 has no built-in popover, so this is a small custom widget that
//! renders the popup through iced's overlay layer, like tooltips and pick
//! lists do.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell, mouse, overlay, renderer};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// Gap between the anchor and the popup.
const GAP: f32 = 4.0;

pub struct Popover<'a, Message> {
    anchor: Element<'a, Message>,
    popup: Option<Element<'a, Message>>,
    on_dismiss: Message,
}

/// Shows `popup` (when `Some`) under the right edge of `anchor`.
pub fn popover<'a, Message: Clone + 'a>(
    anchor: impl Into<Element<'a, Message>>,
    popup: Option<Element<'a, Message>>,
    on_dismiss: Message,
) -> Popover<'a, Message> {
    Popover {
        anchor: anchor.into(),
        popup,
        on_dismiss,
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, iced::Theme, iced::Renderer> for Popover<'a, Message> {
    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = vec![Tree::new(&self.anchor)];
        children.extend(self.popup.as_ref().map(Tree::new));
        children
    }

    fn diff(&self, tree: &mut Tree) {
        let mut elements = vec![&self.anchor];
        elements.extend(self.popup.as_ref());
        tree.diff_children(&elements);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.anchor
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.anchor
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let anchor = layout.bounds() + translation;
        let (anchor_tree, rest) = tree.children.split_first_mut()?;
        match (&mut self.popup, rest.first_mut()) {
            (Some(popup), Some(popup_tree)) => Some(overlay::Element::new(Box::new(Popup {
                anchor,
                content: popup,
                tree: popup_tree,
                on_dismiss: self.on_dismiss.clone(),
            }))),
            _ => self
                .anchor
                .as_widget_mut()
                .overlay(anchor_tree, layout, renderer, viewport, translation),
        }
    }
}

impl<'a, Message: Clone + 'a> From<Popover<'a, Message>> for Element<'a, Message> {
    fn from(popover: Popover<'a, Message>) -> Self {
        Element::new(popover)
    }
}

struct Popup<'a, 'b, Message> {
    anchor: Rectangle,
    content: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    on_dismiss: Message,
}

impl<Message: Clone> overlay::Overlay<Message, iced::Theme, iced::Renderer> for Popup<'_, '_, Message> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &layout::Limits::new(Size::ZERO, bounds));
        let size = node.size();
        // Right-align with the anchor, below it; flip above if it does not fit.
        let x = (self.anchor.x + self.anchor.width - size.width).clamp(0.0, (bounds.width - size.width).max(0.0));
        let below = self.anchor.y + self.anchor.height + GAP;
        let y = if below + size.height <= bounds.height {
            below
        } else {
            (self.anchor.y - GAP - size.height).max(0.0)
        };
        node.move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content
            .as_widget()
            .draw(self.tree, renderer, theme, style, layout, cursor, &layout.bounds());
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let outside_click =
            matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) && !cursor.is_over(layout.bounds());
        if outside_click {
            shell.publish(self.on_dismiss.clone());
            return;
        }
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(self.tree, layout, cursor, &layout.bounds(), renderer)
    }
}
