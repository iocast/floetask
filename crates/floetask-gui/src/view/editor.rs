//! The add/edit dialog.

use iced::keyboard::{Key, key::Named};
use iced::widget::{Column, button, checkbox, column, container, pick_list, row, text, text_editor, text_input};
use iced::{Alignment, Border, Element, Fill};

use floetask_domain::todo::Todo;
use floetask_domain::{Priority, RecurrenceUnit};

use super::EDITOR_ID;
use super::calendar;
use super::icons::{Icon, icon};
use super::popover::popover;
use super::widgets::{caption, primary_button, secondary_button, title};
use crate::app::{DateKey, Editor, Floetask, Message};
use crate::i18n::{tr, trf};
use crate::theme::{self, Colors};

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
        .on_action(Message::EditorAction)
        .min_height(96)
        .max_height(240)
        .padding(12)
        .size(15)
        .style(move |_, status| {
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
        })
        .key_binding(move |press| {
            let command = press.modifiers.command();
            let custom = |message| Some(text_editor::Binding::Custom(message));
            match press.key.as_ref() {
                Key::Named(Named::Enter) if command => custom(Message::SaveEditor),
                Key::Named(Named::Escape) if has_suggestions => custom(Message::EditorDismissSuggestions),
                Key::Named(Named::Escape) if calendar_open => custom(Message::EditorCloseCalendar),
                Key::Named(Named::Escape) => custom(Message::CloseDialog),
                Key::Named(Named::ArrowDown) if has_suggestions => custom(Message::EditorSuggestionMove(1)),
                Key::Named(Named::ArrowUp) if has_suggestions => custom(Message::EditorSuggestionMove(-1)),
                Key::Named(Named::Enter | Named::Tab) if has_suggestions => custom(Message::EditorAccept),
                _ => text_editor::Binding::from_key_press(press),
            }
        });

    let mut content = Column::new().spacing(16).push(title(heading)).push(input);
    if has_suggestions {
        content = content.push(suggestions(editor, colors));
    }
    content = content.push(pickers(app, editor, colors));
    content = content.push(repeat_and_pomodoros(editor, colors));
    content.push(actions(app, editor, colors)).width(580).into()
}

fn suggestions(editor: &Editor, colors: Colors) -> Element<'_, Message> {
    let sigil = editor.typed_tag().map(|tag| tag.sigil).unwrap_or('+');
    container(
        column(editor.suggestions.iter().enumerate().map(|(index, name)| {
            button(text(format!("{sigil}{name}")).size(14))
                .width(Fill)
                .padding([6, 10])
                .style(theme::ghost(colors, index == editor.selected_suggestion))
                .on_press(Message::EditorSuggestion(index))
                .into()
        }))
        .spacing(2),
    )
    .padding(4)
    .style(theme::panel(colors))
    .into()
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
            Message::EditorCloseCalendar
        } else {
            Message::EditorOpenCalendar(key)
        });
        let calendar = editor.calendar.as_ref().filter(|_| open).map(|calendar| {
            container(calendar::view(calendar, app.today, app.settings.week_start, colors))
                .padding(12)
                .style(theme::card(colors))
                .into()
        });
        popover(anchor, calendar, Message::EditorCloseCalendar).align_left()
    };
    column![
        caption(tr("details"), colors),
        row![
            text(tr("priority")).size(13).color(colors.muted),
            pick_list(priorities, Some(PriorityChoice(todo.priority())), |choice| {
                Message::EditorPriority(choice.0)
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
        number("1", &editor.recurrence_count, Message::EditorRecurrenceCount),
        pick_list(units, Some(UnitChoice(editor.recurrence_unit)), |unit| {
            Message::EditorRecurrenceUnit(unit.0)
        })
        .text_size(13),
        checkbox(editor.recurrence_strict)
            .label(tr("strict"))
            .on_toggle(Message::EditorRecurrenceStrict)
            .text_size(13),
        small("set", Message::EditorApplyRecurrence(true)),
        small("clear", Message::EditorApplyRecurrence(false)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let pomodoros = row![
        icon(Icon::Timer, 14.0, colors.muted),
        text(tr("pm")).size(13).color(colors.muted),
        number("1", &editor.pomodoros, Message::EditorPomodoros),
        small("set", Message::EditorApplyPomodoros(true)),
        small("clear", Message::EditorApplyPomodoros(false)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let suggestion = match editor.note_choice() {
        Some(Ok(name)) if editor.note.trim().is_empty() => name.to_string(),
        _ => "name.md".to_owned(),
    };
    let note = row![
        icon(Icon::Note, 14.0, colors.muted),
        text(tr("note")).size(13).color(colors.muted),
        text_input(&suggestion, &editor.note)
            .on_input(Message::EditorNote)
            .on_submit(Message::EditorApplyNote(true))
            .width(240)
            .size(13)
            .padding(6)
            .style(theme::input(colors)),
        small("set", Message::EditorApplyNote(true)),
        small("clear", Message::EditorApplyNote(false)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    column![repeat, pomodoros, note].spacing(10).into()
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
            trf("add_count", &[&count])
        } else {
            tr("add").to_owned()
        }
    };
    row![
        text(tr("editor_hint")).size(12).color(colors.muted).width(Fill),
        secondary_button(tr("cancel"), Message::CloseDialog, colors),
        primary_button(save_label, Message::SaveEditor, colors),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnitChoice(RecurrenceUnit);

impl std::fmt::Display for UnitChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tr(&format!("unit_{}", self.0.symbol())))
    }
}
