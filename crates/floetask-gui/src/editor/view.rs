//! The add/edit dialog.

use iced::keyboard::{Key, key::Named};
use iced::widget::{
    Column, Row, button, checkbox, column, container, pick_list, row, scrollable, space, text, text_editor, text_input,
};
use std::path::Path;

use iced::{Alignment, Border, Element, Fill};
use iced_markdown_editor::markdown_editor;

use floetask_application::NoteService;
use floetask_domain::status::DEFAULT_STATUS;
use floetask_domain::todo::Todo;
use floetask_domain::{NoteLink, Priority, RecurrenceUnit};

use super::date_picker;
use super::{DateKey, Editor};
use crate::app::{Floetask, Message, Target};
use crate::editor;
use crate::i18n::{self, tr, trf};
use crate::list::{self, status_label};
use crate::theme::{self, Colors};
use crate::view::EDITOR_ID;
use crate::view::icons::{Icon, icon};
use crate::view::popover::popover;
use crate::view::widgets::{caption, file_choices, primary_button, secondary_button, title, with_tooltip};

/// Width of the details panel on the right of the dialog.
const DETAILS_WIDTH: f32 = 248.0;
/// What a date field shows while the todo has no such date.
const NO_DATE: &str = "–";

/// The dialog: the todo text and its notes on the left, the attributes the
/// dialog edits for the user in a details panel on the right, and the
/// actions along the bottom.
pub fn view<'a>(app: &'a Floetask, editor: &'a Editor) -> Element<'a, Message> {
    let colors = app.colors();
    let heading = tr(if editor.target.is_some() {
        "edit_todo"
    } else {
        "new_todo"
    });
    let has_suggestions = !editor.suggestions.is_empty();
    let calendar_open = editor.calendar.is_some();
    let input = text_editor(&editor.content)
        .id(EDITOR_ID)
        .placeholder(tr("editor_placeholder"))
        .on_action(|action| Message::Editor(editor::Message::Action(action)))
        .min_height(76)
        .max_height(180)
        .padding(12)
        .size(15)
        .style(field_style(colors))
        .key_binding(move |press| {
            let command = press.modifiers.command();
            let custom = |message| Some(text_editor::Binding::Custom(message));
            match press.key.as_ref() {
                Key::Named(Named::Enter) if command => custom(Message::Editor(editor::Message::Save)),
                Key::Named(Named::Escape) if has_suggestions => {
                    custom(Message::Editor(editor::Message::DismissSuggestions))
                }
                Key::Named(Named::Escape) if calendar_open => custom(Message::Editor(editor::Message::CloseDatePicker)),
                Key::Named(Named::Escape) => custom(Message::CloseDialog),
                Key::Named(Named::ArrowDown) if has_suggestions => {
                    custom(Message::Editor(editor::Message::SuggestionMove(1)))
                }
                Key::Named(Named::ArrowUp) if has_suggestions => {
                    custom(Message::Editor(editor::Message::SuggestionMove(-1)))
                }
                Key::Named(Named::Enter | Named::Tab) if has_suggestions => {
                    custom(Message::Editor(editor::Message::Accept))
                }
                _ => text_editor::Binding::from_key_press(press),
            }
        });

    let mut header = row![title(heading), space::horizontal()]
        .spacing(12)
        .align_y(Alignment::Center);
    if app.state.shows_overview() {
        header = header.push(file_line(app, editor, colors));
    }
    let mut main = Column::new().spacing(14).width(Fill).height(Fill).push(input);
    if has_suggestions {
        main = main.push(suggestions(editor, colors));
    }
    main = main.push(notes(app, editor, colors));
    let body = row![main, details(app, editor, colors)].spacing(20).height(Fill);
    column![header, body, actions(app, editor, colors)]
        .spacing(18)
        .width(Fill)
        .height(Fill)
        .into()
}

/// The look of the dialog's multi-line fields.
fn field_style(colors: Colors) -> impl Fn(&iced::Theme, text_editor::Status) -> text_editor::Style {
    move |_, status| {
        let focused = matches!(status, text_editor::Status::Focused { .. });
        text_editor::Style {
            background: colors.background.into(),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: if focused { colors.primary } else { colors.border },
            },
            placeholder: colors.muted,
            value: colors.text,
            selection: colors.tint(colors.primary, 0.3),
        }
    }
}

/// Autocomplete suggestions as a wrapping row of badges in the project or
/// context colour. The selected one is filled; `Up`/`Down` move, `Enter` or
/// `Tab` insert.
fn suggestions(editor: &Editor, colors: Colors) -> Element<'_, Message> {
    let color = match editor.typed_tag().map(|tag| tag.sigil) {
        Some('@') => colors.success,
        _ => colors.project,
    };
    Row::with_children(editor.suggestions.iter().enumerate().map(|(index, name)| {
        button(text(name.as_str()).size(13))
            .padding([3, 10])
            .style(theme::chip(color, colors, index == editor.selected_suggestion))
            .on_press(Message::Editor(editor::Message::Suggestion(index)))
            .into()
    }))
    .spacing(6)
    .wrap()
    .vertical_spacing(6)
    .into()
}

/// Which file the todo is in, shown in the overview where files mix. A new
/// todo picks its file here; it starts with the active one.
fn file_line<'a>(app: &'a Floetask, editor: &'a Editor, colors: Colors) -> Element<'a, Message> {
    let choices = file_choices(&app.state.drawer_files());
    let current = choices.iter().find(|choice| choice.1 == editor.file).cloned();
    let glyph = icon(Icon::File, 14.0, colors.muted);
    if editor.target.is_some() {
        let name = current.map(|choice| choice.0).unwrap_or_default();
        return row![
            glyph,
            text(trf("editor_file", &[("file", name.into())]))
                .size(13)
                .color(colors.muted)
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into();
    }
    row![
        glyph,
        text(tr("editor_add_to")).size(13).color(colors.muted),
        pick_list(choices, current, |choice| Message::Editor(editor::Message::File(
            choice.1
        )))
        .text_size(13),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// The details panel: one labelled control per attribute, each editing the
/// todo text in place.
fn details<'a>(app: &'a Floetask, editor: &'a Editor, colors: Colors) -> Element<'a, Message> {
    let todo = Todo::from_user_text(&editor.text());
    let mut priorities: Vec<PriorityChoice> = vec![PriorityChoice(None)];
    priorities.extend(Priority::all().map(|p| PriorityChoice(Some(p))));
    let current_status = todo.status_tag().unwrap_or(DEFAULT_STATUS);
    let choice = |key: String| StatusChoice(status_label(app, &key), key);
    let mut statuses: Vec<StatusChoice> = app.status_options().into_iter().map(choice).collect();
    if !statuses.iter().any(|choice| choice.1 == current_status) {
        statuses.push(choice(current_status.to_owned()));
    }
    let priority = select(
        priorities,
        PriorityChoice(todo.priority()),
        |choice| Message::Editor(editor::Message::Priority(choice.0)),
        colors,
    );
    let status = select(
        statuses,
        choice(current_status.to_owned()),
        |choice| Message::Editor(editor::Message::Status(choice.1)),
        colors,
    );

    let units: Vec<UnitChoice> = RecurrenceUnit::ALL.into_iter().map(UnitChoice).collect();
    let count = number(
        &editor.recurrence_count,
        |count| Message::Editor(editor::Message::RecurrenceCount(count)),
        colors,
    );
    let unit = select(
        units,
        UnitChoice(editor.recurrence_unit),
        |unit| Message::Editor(editor::Message::RecurrenceUnit(unit.0)),
        colors,
    );
    let mut repeat = row![count, unit].spacing(8).align_y(Alignment::Center);
    if is_set(&editor.recurrence_count) {
        repeat = repeat.push(clear_button(Message::Editor(editor::Message::ClearRecurrence), colors));
    }
    // Strict sits on the label's line, so the repeat takes one row of controls.
    let repeat_label = row![
        label(Icon::Repeat, "rec", colors),
        checkbox(editor.recurrence_strict)
            .label(tr("strict"))
            .on_toggle(|strict| Message::Editor(editor::Message::RecurrenceStrict(strict)))
            .size(14)
            .text_size(12),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let count = number(
        &editor.pomodoros,
        |count| Message::Editor(editor::Message::Pomodoros(count)),
        colors,
    );
    let mut pomodoros = row![count].spacing(8).align_y(Alignment::Center);
    if is_set(&editor.pomodoros) {
        pomodoros = pomodoros.push(clear_button(Message::Editor(editor::Message::ClearPomodoros), colors));
    }

    let fields = column![
        caption(tr("details"), colors),
        property(label(Icon::Star, "priority", colors), priority),
        property(label(Icon::Status, "status", colors), status),
        property(
            label(Icon::Calendar, DateKey::Due.key(), colors),
            date_field(app, editor, DateKey::Due, todo.due(), colors)
        ),
        property(
            label(Icon::Calendar, DateKey::Threshold.key(), colors),
            date_field(app, editor, DateKey::Threshold, todo.threshold(), colors)
        ),
        property(repeat_label, repeat),
        property(label(Icon::Timer, "pm", colors), pomodoros),
    ]
    .spacing(14);
    container(
        scrollable(fields)
            // Beside the fields, not over them, once the dialog is too
            // low to show them all.
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(6).scroller_width(6).spacing(6),
            ))
            .style(theme::slim_scroller(colors)),
    )
    .padding(16)
    .width(DETAILS_WIDTH)
    .height(Fill)
    .style(theme::inset(colors))
    .into()
}

/// A control in the details panel with its label above it.
fn property<'a>(
    label: impl Into<Element<'a, Message>>,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![label.into(), control.into()].spacing(6).into()
}

/// A property's label: a small icon and the translated name, filling the
/// line so a control can sit at its end.
fn label<'a>(glyph: Icon, key: &'static str, colors: Colors) -> Element<'a, Message> {
    row![
        icon(glyph, 13.0, colors.muted),
        text(tr(key)).size(12).color(colors.muted)
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .width(Fill)
    .into()
}

/// A pick list styled like the dialog's fields, as wide as its space.
fn select<'a, T>(
    options: Vec<T>,
    selected: T,
    on_select: impl Fn(T) -> Message + 'a,
    colors: Colors,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
{
    pick_list(options, Some(selected), on_select)
        .width(Fill)
        .padding([6, 10])
        .text_size(13)
        .style(theme::select(colors))
        .menu_style(theme::select_menu(colors))
        .into()
}

/// A small field for a count, where 0 means none.
fn number<'a>(current: &str, on_input: fn(String) -> Message, colors: Colors) -> Element<'a, Message> {
    text_input("0", current)
        .on_input(on_input)
        .width(56)
        .size(13)
        .padding([6, 10])
        .style(theme::input(colors))
        .into()
}

/// Whether a count field holds more than 0.
fn is_set(count: &str) -> bool {
    count.parse::<u32>().is_ok_and(|count| count > 0)
}

/// Resets a count to 0; shown only while it is set.
fn clear_button<'a>(message: Message, colors: Colors) -> Element<'a, Message> {
    let button = button(icon(Icon::Close, 14.0, colors.muted))
        .padding(6)
        .style(theme::ghost(colors, false))
        .on_press(message);
    with_tooltip(button, tr("clear"), colors)
}

/// A date as a field that opens the date picker under it; the picker
/// floats over the dialog without moving it.
fn date_field<'a>(
    app: &'a Floetask,
    editor: &'a Editor,
    key: DateKey,
    date: Option<floetask_domain::Date>,
    colors: Colors,
) -> Element<'a, Message> {
    let open = editor.calendar.as_ref().is_some_and(|c| c.key == key);
    let value: Element<'a, Message> = match date {
        Some(date) => text(date.to_string()).size(13).into(),
        None => text(NO_DATE).size(13).color(colors.muted).into(),
    };
    let tint = if open || date.is_some() {
        colors.primary
    } else {
        colors.muted
    };
    let anchor = button(
        row![icon(Icon::Calendar, 14.0, tint), value]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .width(Fill)
    .padding([6, 10])
    .style(theme::field_button(colors, open))
    // While open, a click on the field closes the calendar like any outside click.
    .on_press(if open {
        Message::Editor(editor::Message::CloseDatePicker)
    } else {
        Message::Editor(editor::Message::OpenDatePicker(key))
    });
    let calendar = editor.calendar.as_ref().filter(|_| open).map(|calendar| {
        container(date_picker::view(calendar, app.today, app.settings.week_start, colors))
            .padding(12)
            .style(theme::card(colors))
            .into()
    });
    popover(anchor, calendar, Message::Editor(editor::Message::CloseDatePicker)).into()
}

/// The todo's notes, in a Markdown editor. Saving writes them to the
/// `note:` file, adding the tag with a name taken from the todo text when
/// there is none. The header shows the editor's formatting toolbar, links a
/// file from disk instead, opens the note in the note window, or unlinks it.
fn notes<'a>(app: &Floetask, editor: &'a Editor, colors: Colors) -> Element<'a, Message> {
    let target = editor.note_target();
    let toggle = |glyph: Icon, hint: &'static str, message: Message, active: bool| {
        let tint = if active { colors.primary } else { colors.muted };
        let button = button(icon(glyph, 14.0, tint))
            .padding(5)
            .style(theme::ghost(colors, active))
            .on_press(message);
        with_tooltip(button, tr(hint), colors)
    };
    let tool = |glyph: Icon, hint: &'static str, message: Message| toggle(glyph, hint, message, false);
    let in_window = app.dialog_note_in_window();
    let mut header = row![
        icon(Icon::Note, 14.0, colors.muted),
        caption(tr("notes"), colors),
        space::horizontal(),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    if !in_window {
        header = header.push(toggle(
            Icon::Format,
            "note_toolbar",
            Message::Editor(editor::Message::ToggleNoteToolbar),
            editor.note_toolbar,
        ));
    }
    header = header.push(tool(
        Icon::Attach,
        "link_note_file",
        Message::Editor(editor::Message::PickNoteFile),
    ));
    if !matches!(target, Some(Err(_))) {
        header = header.push(tool(
            Icon::Window,
            "open_note_window",
            Message::Editor(editor::Message::OpenNoteWindow),
        ));
    }
    if target.is_some() {
        header = header.push(tool(
            Icon::Close,
            "unlink_note",
            Message::Editor(editor::Message::UnlinkNote),
        ));
    }

    let mut section = column![header].spacing(8).height(Fill);
    if let Some(picked) = &editor.pending_link {
        section = section.push(link_question(&editor.file, picked, colors));
    }
    // While the note window has this note, the dialog leaves it alone.
    let field: Element<'a, Message> = if in_window {
        container(
            row![
                text(tr("note_in_window")).size(13).color(colors.muted).width(Fill),
                secondary_button(
                    tr("show_note_window"),
                    Message::Editor(editor::Message::OpenNoteWindow),
                    colors
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([10, 12])
        .height(Fill)
        .align_y(Alignment::Center)
        .style(theme::note_field(colors, false))
        .into()
    } else {
        let state = editor.note.state();
        container(
            markdown_editor(state)
                .toolbar(editor.note_toolbar)
                .status_bar(false)
                .height(Fill)
                .view()
                .map(|action| Message::Editor(editor::Message::NoteAction(action))),
        )
        .padding(4)
        .height(Fill)
        .style(theme::note_field(colors, state.is_focused()))
        .into()
    };

    // Say where the notes go once there is something to save.
    let folder = NoteService::folder_name(&editor.file);
    let saved_to = |link: &NoteLink| match link {
        NoteLink::Folder(name) => trf("note_saved_to", &[("path", format!("{folder}/{name}").into())]),
        NoteLink::External(path) => trf("note_saved_to", &[("path", path.as_str().into())]),
    };
    let (hint, color) = match target {
        Some(Ok(link)) => (saved_to(&link), colors.muted),
        Some(Err(error)) => (i18n::error_text(&error.into()), colors.danger),
        None if !editor.note.text().trim().is_empty() => {
            (saved_to(&NoteLink::Folder(editor.new_note_name())), colors.muted)
        }
        None => (tr("notes_placeholder").to_owned(), colors.muted),
    };
    section.push(field).push(text(hint).size(12).color(color)).into()
}

/// Asks what to do with a picked file outside the notes folder: move it in
/// with the other notes, or link it where it is.
fn link_question<'a>(todo_file: &Path, picked: &Path, colors: Colors) -> Element<'a, Message> {
    let file_name = picked
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = NoteService::folder_name(todo_file);
    container(
        column![
            text(trf(
                "note_outside_folder",
                &[("file", file_name.as_str().into()), ("folder", folder.as_str().into())]
            ))
            .size(13),
            row![
                primary_button(
                    trf("note_move_into", &[("folder", folder.as_str().into())]),
                    Message::Editor(editor::Message::MovePickedNote),
                    colors
                ),
                secondary_button(
                    tr("note_keep_in_place"),
                    Message::Editor(editor::Message::KeepPickedNote),
                    colors
                ),
                space::horizontal(),
                secondary_button(tr("cancel"), Message::Editor(editor::Message::CancelPickedNote), colors),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .padding(12)
    .width(Fill)
    .style(theme::code(colors))
    .into()
}

fn actions<'a>(app: &'a Floetask, editor: &'a Editor, colors: Colors) -> Element<'a, Message> {
    let save_label = if editor.target.is_some() {
        tr("save").to_owned()
    } else {
        let count = app
            .services
            .todo_files
            .prepare_new_todos(&editor.text(), &app.input_options())
            .len();
        if count > 1 {
            trf("add_count", &[("count", count.into())])
        } else {
            tr("add").to_owned()
        }
    };
    let mut actions = Row::new();
    // Only a todo already in the file can be deleted or completed from here.
    // Deleting asks first, as the row menu does, with the note question when
    // no other todo links to its note.
    if let Some(todo) = &editor.target {
        let target = Target {
            file: editor.file.clone(),
            todo: todo.clone(),
        };
        // Quiet until hovered: the dialog's main actions are on the right.
        let delete = button(
            row![icon(Icon::Trash, 14.0, colors.danger), text(tr("delete")).size(14)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(theme::danger_ghost(colors))
        .on_press(Message::List(list::Message::AskDelete(target)));
        actions = actions.push(delete);
    }
    actions = actions.push(text(tr("editor_hint")).size(12).color(colors.muted).width(Fill));
    if editor.target.is_some() {
        let label = tr(if Todo::from_user_text(&editor.text()).is_complete() {
            "reopen"
        } else {
            "complete"
        });
        actions = actions.push(secondary_button(
            label,
            Message::Editor(editor::Message::ToggleComplete),
            colors,
        ));
    }
    actions
        .push(secondary_button(tr("cancel"), Message::CloseDialog, colors))
        .push(primary_button(
            save_label,
            Message::Editor(editor::Message::Save),
            colors,
        ))
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

/// Priority option for the pick list; `None` shows as a dash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PriorityChoice(Option<Priority>);

impl std::fmt::Display for PriorityChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(priority) => write!(f, "({priority})"),
            None => write!(f, "–"),
        }
    }
}

/// Status option for the pick list: its display name and its key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StatusChoice(String, String);

impl std::fmt::Display for StatusChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnitChoice(RecurrenceUnit);

impl std::fmt::Display for UnitChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tr(&format!("unit_{}", self.0.symbol())))
    }
}
