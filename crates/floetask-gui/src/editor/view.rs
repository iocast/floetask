//! The add/edit dialog.

use iced::keyboard::{Key, key::Named};
use iced::widget::{
    Column, Row, button, checkbox, column, container, pick_list, row, space, text, text_editor, text_input,
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
use crate::app::{Floetask, Message};
use crate::editor;
use crate::i18n::{self, tr, trf};
use crate::list::status_label;
use crate::theme::{self, Colors};
use crate::view::EDITOR_ID;
use crate::view::icons::{Icon, icon};
use crate::view::popover::popover;
use crate::view::widgets::{caption, primary_button, secondary_button, title, with_tooltip};

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
        .min_height(96)
        .max_height(240)
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

    let mut content = Column::new().spacing(16).push(title(heading));
    if app.state.shows_overview() {
        content = content.push(file_line(app, editor, colors));
    }
    let mut content = content.push(input);
    if has_suggestions {
        content = content.push(suggestions(editor, colors));
    }
    content = content.push(pickers(app, editor, colors));
    content = content.push(repeat_and_pomodoros(editor, colors));
    content = content.push(notes(app, editor, colors));
    content.push(actions(app, editor, colors)).width(580).into()
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
    let choices = file_choices(app);
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

/// The overview's files for the file picker, in drawer order. A name that
/// several files share gets its folder added, so each choice is clear.
fn file_choices(app: &Floetask) -> Vec<FileChoice> {
    let files = app.state.overview_files();
    let names: Vec<String> = files.iter().map(|entry| entry.file_name()).collect();
    files
        .iter()
        .zip(&names)
        .map(|(entry, name)| {
            let shared = names.iter().filter(|other| *other == name).count() > 1;
            let label = match entry.path.parent() {
                Some(folder) if shared => format!("{name} ({})", folder.display()),
                _ => name.clone(),
            };
            FileChoice(label, entry.path.clone())
        })
        .collect()
}

fn pickers<'a>(app: &'a Floetask, editor: &'a Editor, colors: Colors) -> Element<'a, Message> {
    let todo = Todo::from_user_text(&editor.text());
    let mut priorities: Vec<PriorityChoice> = vec![PriorityChoice(None)];
    priorities.extend(Priority::all().map(|p| PriorityChoice(Some(p))));
    let date_button = |key: DateKey, date: Option<floetask_domain::Date>| {
        let open = editor.calendar.as_ref().is_some_and(|c| c.key == key);
        let label = match date {
            Some(date) => format!("{} {date}", tr(key.key())),
            None => tr(key.key()).to_owned(),
        };
        let tint = if open || date.is_some() {
            colors.primary
        } else {
            colors.text
        };
        let anchor = button(
            row![icon(Icon::Calendar, 14.0, tint), text(label).size(13)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .style(theme::ghost(colors, open || date.is_some()))
        // While open, a click on the button closes the calendar like any outside click.
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
        popover(anchor, calendar, Message::Editor(editor::Message::CloseDatePicker)).align_left()
    };
    let current_status = todo.status_tag().unwrap_or(DEFAULT_STATUS);
    let choice = |key: String| StatusChoice(status_label(app, &key), key);
    let mut statuses: Vec<StatusChoice> = app.status_options().into_iter().map(choice).collect();
    if !statuses.iter().any(|choice| choice.1 == current_status) {
        statuses.push(choice(current_status.to_owned()));
    }
    column![
        caption(tr("details"), colors),
        row![
            text(tr("priority")).size(13).color(colors.muted),
            pick_list(priorities, Some(PriorityChoice(todo.priority())), |choice| {
                Message::Editor(editor::Message::Priority(choice.0))
            })
            .text_size(13),
            text(tr("status")).size(13).color(colors.muted),
            pick_list(statuses, Some(choice(current_status.to_owned())), |choice| {
                Message::Editor(editor::Message::Status(choice.1))
            })
            .text_size(13),
            date_button(DateKey::Due, todo.due()),
            date_button(DateKey::Threshold, todo.threshold()),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8)
    .into()
}

fn repeat_and_pomodoros(editor: &Editor, colors: Colors) -> Element<'_, Message> {
    let small = move |label: &'static str, message: Message| {
        button(text(tr(label)).size(12))
            .padding([4, 10])
            .style(theme::secondary(colors))
            .on_press(message)
    };
    let number = |value: &'static str, current: &str, on_input: fn(String) -> Message| {
        text_input(value, current)
            .on_input(on_input)
            .width(48)
            .size(13)
            .padding(6)
            .style(theme::input(colors))
    };
    let units: Vec<UnitChoice> = RecurrenceUnit::ALL.into_iter().map(UnitChoice).collect();
    let repeat = row![
        icon(Icon::Repeat, 14.0, colors.muted),
        text(tr("rec")).size(13).color(colors.muted),
        number("1", &editor.recurrence_count, |count| Message::Editor(
            editor::Message::RecurrenceCount(count)
        )),
        pick_list(units, Some(UnitChoice(editor.recurrence_unit)), |unit| {
            Message::Editor(editor::Message::RecurrenceUnit(unit.0))
        })
        .text_size(13),
        checkbox(editor.recurrence_strict)
            .label(tr("strict"))
            .on_toggle(|strict| Message::Editor(editor::Message::RecurrenceStrict(strict)))
            .text_size(13),
        small("set", Message::Editor(editor::Message::ApplyRecurrence(true))),
        small("clear", Message::Editor(editor::Message::ApplyRecurrence(false))),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let pomodoros = row![
        icon(Icon::Timer, 14.0, colors.muted),
        text(tr("pm")).size(13).color(colors.muted),
        number("1", &editor.pomodoros, |count| Message::Editor(
            editor::Message::Pomodoros(count)
        )),
        small("set", Message::Editor(editor::Message::ApplyPomodoros(true))),
        small("clear", Message::Editor(editor::Message::ApplyPomodoros(false))),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    column![repeat, pomodoros].spacing(10).into()
}

/// The todo's notes, in a Markdown editor. Saving writes them to the
/// `note:` file, adding the tag with a name taken from the todo text when
/// there is none. The header links a file from disk instead, opens the note
/// in the note window, or unlinks it.
fn notes<'a>(app: &Floetask, editor: &'a Editor, colors: Colors) -> Element<'a, Message> {
    let target = editor.note_target();
    let tool = |glyph: Icon, hint: &'static str, message: Message| {
        let button = button(icon(glyph, 14.0, colors.muted))
            .padding(5)
            .style(theme::ghost(colors, false))
            .on_press(message);
        with_tooltip(button, tr(hint), colors)
    };
    let mut header = row![
        icon(Icon::Note, 14.0, colors.muted),
        caption(tr("notes"), colors),
        space::horizontal(),
        tool(
            Icon::Attach,
            "link_note_file",
            Message::Editor(editor::Message::PickNoteFile)
        ),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
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

    let mut section = column![header].spacing(8);
    if let Some(picked) = &editor.pending_link {
        section = section.push(link_question(&editor.file, picked, colors));
    }
    // While the note window has this note, the dialog leaves it alone.
    let field: Element<'a, Message> = if app.dialog_note_in_window() {
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
        .style(theme::note_field(colors, false))
        .into()
    } else {
        let state = editor.note.state();
        container(
            markdown_editor(state)
                .status_bar(false)
                .height(220)
                .view()
                .map(|action| Message::Editor(editor::Message::NoteAction(action))),
        )
        .padding(4)
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
    row![
        text(tr("editor_hint")).size(12).color(colors.muted).width(Fill),
        secondary_button(tr("cancel"), Message::CloseDialog, colors),
        primary_button(save_label, Message::Editor(editor::Message::Save), colors),
    ]
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

/// File option for the pick list: its label and its path.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileChoice(String, std::path::PathBuf);

impl std::fmt::Display for FileChoice {
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
