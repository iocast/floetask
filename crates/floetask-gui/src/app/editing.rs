//! The add/edit dialog and the date pickers.

use iced::Task;
use iced::widget::operation;

use floetask_domain::Recurrence;
use floetask_domain::todo::Todo;

use super::{Calendar, DateKey, Dialog, Editor, Floetask, Message};
use crate::compose::edit_text;
use crate::i18n::tr;
use crate::view;

impl Floetask {
    pub(super) fn update_editing(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewTodo => self.open_new_editor(""),
            Message::CalendarMonth(months) => {
                if let Some(calendar) = self.active_calendar() {
                    calendar.shift_month(months);
                }
                Task::none()
            }
            Message::CalendarPick(date) => self.pick_date(date),
            Message::SaveEditor => self.save_editor(),
            other => {
                let (projects, contexts) = (self.projects.clone(), self.contexts.clone());
                let today = self.today;
                let Some(Dialog::Editor(editor)) = &mut self.dialog else {
                    return Task::none();
                };
                match other {
                    Message::EditorAction(action) => {
                        editor.content.perform(action);
                        editor.refresh_suggestions(&projects, &contexts);
                        Task::none()
                    }
                    Message::EditorSuggestion(index) => {
                        editor.accept_suggestion(index);
                        operation::focus(view::EDITOR_ID)
                    }
                    Message::EditorSuggestionMove(step) => {
                        let count = editor.suggestions.len().max(1) as isize;
                        editor.selected_suggestion =
                            (editor.selected_suggestion as isize + step).rem_euclid(count) as usize;
                        Task::none()
                    }
                    Message::EditorDismissSuggestions => {
                        editor.suggestions.clear();
                        Task::none()
                    }
                    Message::EditorAccept => {
                        editor.accept_suggestion(editor.selected_suggestion);
                        Task::none()
                    }
                    Message::EditorPriority(priority) => {
                        rewrite(editor, |todo| todo.with_priority(priority));
                        Task::none()
                    }
                    Message::EditorOpenCalendar(key) => {
                        editor.calendar = match &editor.calendar {
                            Some(calendar) if calendar.key == key => None,
                            _ => {
                                let todo = Todo::from_user_text(&editor.text());
                                let current = match key {
                                    DateKey::Due => todo.due(),
                                    DateKey::Threshold => todo.threshold(),
                                };
                                Some(Calendar::new(key, current, today))
                            }
                        };
                        Task::none()
                    }
                    Message::EditorCloseCalendar => {
                        editor.calendar = None;
                        operation::focus(view::EDITOR_ID)
                    }
                    Message::EditorRecurrenceCount(count) => {
                        if count.chars().all(|c| c.is_ascii_digit()) && count.len() <= 3 {
                            editor.recurrence_count = count;
                        }
                        Task::none()
                    }
                    Message::EditorRecurrenceUnit(unit) => {
                        editor.recurrence_unit = unit;
                        Task::none()
                    }
                    Message::EditorRecurrenceStrict(strict) => {
                        editor.recurrence_strict = strict;
                        Task::none()
                    }
                    Message::EditorApplyRecurrence(set) => {
                        let value = set.then(|| Recurrence {
                            strict: editor.recurrence_strict,
                            count: editor.recurrence_count.parse().unwrap_or(1).max(1),
                            unit: editor.recurrence_unit,
                        });
                        rewrite(editor, |todo| {
                            todo.with_extension("rec", value.map(|r| r.to_string()).as_deref())
                        });
                        Task::none()
                    }
                    Message::EditorPomodoros(count) => {
                        if count.chars().all(|c| c.is_ascii_digit()) && count.len() <= 3 {
                            editor.pomodoros = count;
                        }
                        Task::none()
                    }
                    Message::EditorApplyPomodoros(set) => {
                        let value = editor.pomodoros.parse::<u32>().ok().filter(|_| set);
                        rewrite(editor, |todo| {
                            todo.with_extension("pm", value.map(|n| n.to_string()).as_deref())
                        });
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
        }
    }

    pub(crate) fn open_new_editor(&mut self, text: &str) -> Task<Message> {
        self.row_menu = None;
        self.dialog = Some(Dialog::Editor(Box::new(Editor::new(None, text))));
        operation::focus(view::EDITOR_ID)
    }

    fn save_editor(&mut self) -> Task<Message> {
        let Some(Dialog::Editor(editor)) = &self.dialog else {
            return Task::none();
        };
        let text = editor.text();
        if text.trim().is_empty() {
            return self.toast_error(tr("empty_todo"));
        }
        let target = editor.target.clone();
        self.dialog = None;
        let input = self.input_options();
        match target {
            Some(target) => self.change_active_file(move |service, path, options| {
                service.update(path, &target, &text, &input, options)
            }),
            None => self.change_active_file(move |service, path, options| service.add(path, &text, &input, options)),
        }
    }

    fn active_calendar(&mut self) -> Option<&mut Calendar> {
        match &mut self.dialog {
            Some(Dialog::RowDate { calendar, .. }) => Some(calendar),
            Some(Dialog::Editor(editor)) => editor.calendar.as_mut(),
            _ => None,
        }
    }

    fn pick_date(&mut self, date: Option<floetask_domain::Date>) -> Task<Message> {
        match self.dialog.take() {
            Some(Dialog::RowDate { target, calendar }) => self.change_active_file(move |service, path, options| {
                service.set_date(path, &target, calendar.key.key(), date, options)
            }),
            Some(Dialog::Editor(mut editor)) => {
                if let Some(calendar) = editor.calendar.take() {
                    rewrite(&mut editor, |todo| todo.with_date_extension(calendar.key.key(), date));
                }
                self.dialog = Some(Dialog::Editor(editor));
                operation::focus(view::EDITOR_ID)
            }
            other => {
                self.dialog = other;
                Task::none()
            }
        }
    }
}

/// Applies a todo edit to the dialog text.
fn rewrite(editor: &mut Editor, edit: impl FnOnce(&Todo) -> Todo) {
    let text = editor.text();
    if text.trim().is_empty() {
        // Pickers on an empty field still produce a valid todo skeleton.
        editor.set_text(&edit(&Todo::parse("")).editable_text());
    } else {
        editor.set_text(&edit_text(&text, edit));
    }
}
