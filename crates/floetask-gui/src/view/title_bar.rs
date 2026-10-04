//! The custom title bar: logo, file tabs, app actions and window buttons,
//! like Firefox and Zed. Empty space drags the window; a double click
//! maximises it.

use iced::widget::{Row, button, center, container, mouse_area, row, rule, space, text};
use iced::{Alignment, Element, Fill, Length};

use super::icons::{Icon, icon};
use super::widgets::{icon_button, menu_button, with_tooltip};
use crate::app::{Floetask, Message};
use crate::i18n::tr;
use crate::theme::{self, Colors};

const HEIGHT: f32 = 44.0;

pub fn view(app: &Floetask) -> Element<'_, Message> {
    let colors = app.colors();
    let drag = |content: Element<'static, Message>| -> Element<'static, Message> {
        mouse_area(content)
            .on_press(Message::WindowDrag)
            .on_double_click(Message::WindowToggleMaximize)
            .into()
    };

    let logo = drag(
        container(crate::logo::view(22.0))
            .padding([0, 12])
            .center_y(HEIGHT)
            .into(),
    );
    let bar = row![
        logo,
        tabs(app, colors),
        drag(space().width(Fill).height(HEIGHT).into()),
        actions(app, colors),
        rule::vertical(1).style(move |_| rule::Style {
            color: colors.border,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Padded(10),
            snap: true,
        }),
        window_controls(colors),
    ]
    .spacing(4)
    .height(HEIGHT)
    .align_y(Alignment::Center);

    container(bar).width(Fill).style(theme::app(colors)).into()
}

/// One tab per registered file. The active tab has a "⋯" menu.
fn tabs(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    if !app.state.tabs_visible {
        return space().into();
    }
    Row::with_children(app.state.files.iter().enumerate().map(|(index, entry)| {
        let active = index == app.state.active_file;
        let mut label = row![text(entry.file_name()).size(13)]
            .spacing(6)
            .align_y(Alignment::Center);
        if active {
            let open = app.tab_menu == Some(index);
            label = label.push(
                button(icon(Icon::ChevronDown, 14.0, colors.muted))
                    .padding(2)
                    .style(theme::ghost(colors, open))
                    .on_press(Message::TabMenu(if open { None } else { Some(index) })),
            );
        }
        button(label)
            .padding([5, 12])
            .style(theme::tab(colors, active))
            .on_press(Message::SelectFile(index))
            .into()
    }))
    .spacing(2)
    .align_y(Alignment::Center)
    .into()
}

/// App actions next to the window buttons.
fn actions(app: &Floetask, colors: Colors) -> Element<'_, Message> {
    let has_file = app.active_document().is_some();
    let can_archive =
        app.state.active_entry().is_some_and(|entry| entry.done_path.is_some()) && app.listing.counts.completed > 0;
    let new_todo = with_tooltip(
        button(
            row![icon(Icon::Plus, 16.0, colors.primary), text(tr("new")).size(13)]
                .spacing(4)
                .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .style(theme::ghost(colors, true))
        .on_press_maybe(has_file.then_some(Message::NewTodo)),
        tr("add_todo"),
        colors,
    );
    let mut items = row![new_todo].spacing(2).align_y(Alignment::Center);
    items = items.push(icon_button(
        Icon::Search,
        tr("search"),
        has_file.then_some(Message::ToggleSearch),
        colors,
        app.state.search_visible,
    ));
    items = items.push(icon_button(
        Icon::Filter,
        tr("toggle_drawer"),
        has_file.then_some(Message::ToggleDrawer),
        colors,
        app.state.drawer_open || app.state.view.has_active_filters(),
    ));
    if can_archive {
        items = items.push(icon_button(
            Icon::Archive,
            tr("archive"),
            Some(Message::ArchiveCompleted),
            colors,
            false,
        ));
    }
    let theme_icon = if colors.dark { Icon::Sun } else { Icon::Moon };
    items
        .push(icon_button(
            Icon::Folder,
            tr("open_file"),
            Some(Message::OpenFileDialog),
            colors,
            false,
        ))
        .push(icon_button(
            theme_icon,
            tr("toggle_theme"),
            Some(Message::ToggleTheme),
            colors,
            false,
        ))
        .push(icon_button(
            Icon::Settings,
            tr("settings"),
            Some(Message::OpenSettings),
            colors,
            false,
        ))
        .into()
}

fn window_controls(colors: Colors) -> Element<'static, Message> {
    let control = |glyph: Icon, message: Message, close: bool| {
        button(center(icon(glyph, 16.0, colors.text)))
            .width(46)
            .height(HEIGHT)
            .padding(0)
            .style(theme::window_control(colors, close))
            .on_press(message)
    };
    row![
        control(Icon::Minimize, Message::WindowMinimize, false),
        control(Icon::Maximize, Message::WindowToggleMaximize, false),
        control(Icon::Close, Message::Quit, true),
    ]
    .into()
}

/// The menu of the active tab, shown under the title bar.
pub fn tab_menu(app: &Floetask) -> Option<Element<'_, Message>> {
    let index = app.tab_menu?;
    let colors = app.colors();
    Some(
        container(
            row![
                menu_button(
                    Icon::Archive,
                    tr("change_done_file"),
                    Message::ChangeDoneFile(index),
                    colors
                ),
                menu_button(
                    Icon::Folder,
                    tr("reveal_todo_file"),
                    Message::RevealTodoFile(index),
                    colors
                ),
                menu_button(
                    Icon::Folder,
                    tr("reveal_done_file"),
                    Message::RevealDoneFile(index),
                    colors
                ),
                menu_button(Icon::Close, tr("remove_file"), Message::AskRemoveFile(index), colors),
            ]
            .spacing(4),
        )
        .padding(6)
        .width(Length::Shrink)
        .style(theme::panel(colors))
        .into(),
    )
}
