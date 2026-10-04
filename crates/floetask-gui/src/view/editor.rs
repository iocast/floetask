//! The add/edit dialog.

use iced::keyboard::{Key, key::Named};
use iced::widget::{Column, button, checkbox, column, container, pick_list, row, text, text_editor, text_input};
use iced::{Alignment, Element, Fill};

use floetask_domain::todo::Todo;
use floetask_domain::{Priority, RecurrenceUnit};

use super::EDITOR_ID;
use super::calendar;
use crate::app::{DateKey, Editor, Floetask, Message};
use crate::i18n::{tr, trf};

pub fn view<'a>(app: &'a Floetask, editor: &'a Editor) -> Element<'a, Message> {
    let title = tr(if editor.target.is_some() {
        "edit_todo"
    } else {
        "new_todo"
    });
    let has_suggestions = !editor.suggestions.is_empty();
    let input = text_editor(&editor.content)
        .id(EDITOR_ID)
        .placeholder(tr("editor_placeholder"))
        .on_action(Message::EditorAction)
        .min_height(90)
        .max_height(240)
        .padding(10)
        .key_binding(move |press| {
            let command = press.modifiers.command();
            let custom = |message| Some(text_editor::Binding::Custom(message));
            match press.key.as_ref() {
                Key::Named(Named::Enter) if command => custom(Message::SaveEditor),
                Key::Named(Named::Escape) if has_suggestions => custom(Message::EditorDismissSuggestions),
                Key::Named(Named::Escape) => custom(Message::CloseDialog),
                Key::Named(Named::ArrowDown) if has_suggestions => custom(Message::EditorSuggestionMove(1)),
                Key::Named(Named::ArrowUp) if has_suggestions => custom(Message::EditorSuggestionMove(-1)),
                Key::Named(Named::Enter | Named::Tab) if has_suggestions => custom(Message::EditorAccept),
                _ => text_editor::Binding::from_key_press(press),
            }
        });

    let mut content = Column::new().spacing(12).push(text(title).size(20)).push(input);
    if has_suggestions {
        content = content.push(suggestions(editor));
    }
    content = content.push(pickers(editor));
    if let Some(calendar) = &editor.calendar {
        content = content.push(calendar::view(calendar, app.today, app.settings.week_start));
    }
    content = content.push(repeat_and_pomodoros(editor));
    content.push(actions(app, editor)).width(560).into()
}

fn suggestions(editor: &Editor) -> Element<'_, Message> {
    let sigil = editor.typed_tag().map(|tag| tag.sigil).unwrap_or('+');
    column(editor.suggestions.iter().enumerate().map(|(index, name)| {
        button(text(format!("{sigil}{name}")).size(14))
            .width(Fill)
            .style(if index == editor.selected_suggestion {
                button::primary
            } else {
                button::text
            })
            .on_press(Message::EditorSuggestion(index))
            .into()
    }))
    .into()
}

fn pickers(editor: &Editor) -> Element<'_, Message> {
    let todo = Todo::from_user_text(&editor.text());
    let mut priorities: Vec<PriorityChoice> = vec![PriorityChoice(None)];
    priorities.extend(Priority::all().map(|p| PriorityChoice(Some(p))));
    let date_button = |key: DateKey, date: Option<floetask_domain::Date>| {
        let label = match date {
            Some(date) => format!("{}: {date}", tr(key.key())),
            None => format!("{} ▾", tr(key.key())),
        };
        button(text(label).size(13))
            .style(button::secondary)
            .on_press(Message::EditorOpenCalendar(key))
    };
    row![
        text(tr("priority")).size(13),
        pick_list(priorities, Some(PriorityChoice(todo.priority())), |choice| {
            Message::EditorPriority(choice.0)
        })
        .text_size(13),
        date_button(DateKey::Due, todo.due()),
        date_button(DateKey::Threshold, todo.threshold()),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn repeat_and_pomodoros(editor: &Editor) -> Element<'_, Message> {
    let small =
        |label: &'static str, message: Message| button(text(tr(label)).size(12)).padding([3, 8]).on_press(message);
    let units: Vec<UnitChoice> = RecurrenceUnit::ALL.into_iter().map(UnitChoice).collect();
    let repeat = row![
        text(tr("rec")).size(13),
        text_input("1", &editor.recurrence_count)
            .on_input(Message::EditorRecurrenceCount)
            .width(44)
            .size(13),
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
    .spacing(6)
    .align_y(Alignment::Center);
    let pomodoros = row![
        text(tr("pm")).size(13),
        text_input("1", &editor.pomodoros)
            .on_input(Message::EditorPomodoros)
            .width(44)
            .size(13),
        small("set", Message::EditorApplyPomodoros(true)),
        small("clear", Message::EditorApplyPomodoros(false)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    column![repeat, pomodoros].spacing(8).into()
}

fn actions<'a>(app: &'a Floetask, editor: &'a Editor) -> Element<'a, Message> {
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
    container(
        row![
            button(text(tr("cancel")))
                .style(button::secondary)
                .on_press(Message::CloseDialog),
            button(text(save_label))
                .style(button::primary)
                .on_press(Message::SaveEditor),
        ]
        .spacing(8),
    )
    .align_right(Fill)
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
