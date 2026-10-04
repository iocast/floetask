//! Row interactions in the todo list.

use iced::Task;
use iced::widget::operation;

use floetask_application::TodoRef;
use floetask_domain::listing::ListedTodo;
use floetask_domain::todo::Todo;

use super::{Calendar, DateKey, Dialog, Editor, Floetask, Message, Pending};
use crate::i18n::tr;
use crate::view;

impl Floetask {
    pub(super) fn update_list(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenTodo(target) => {
                let text = Todo::parse(&target.raw).editable_text();
                self.row_menu = None;
                self.dialog = Some(Dialog::Editor(Box::new(Editor::new(Some(target), &text))));
                operation::focus(view::EDITOR_ID)
            }
            Message::ToggleComplete(target) => {
                self.change_active_file(move |service, path, options| service.toggle_complete(path, &target, options))
            }
            Message::RowMenu(line) => {
                self.row_menu = if self.row_menu == line { None } else { line };
                Task::none()
            }
            Message::CopyTodo(raw) => {
                self.row_menu = None;
                Task::batch([iced::clipboard::write(raw), self.toast(tr("copied"))])
            }
            Message::ArchiveOne(target) => self.archive_one(target),
            Message::AskDelete(target) => {
                self.row_menu = None;
                self.dialog = Some(Dialog::Confirm {
                    message: tr("delete_confirm").to_owned(),
                    detail: Some(Todo::parse(&target.raw).editable_text()),
                    pending: Pending::DeleteTodo(target),
                });
                Task::none()
            }
            Message::ChipFilter(attribute, value) => {
                self.state.view.toggle_filter(attribute, &value, false);
                self.refresh();
                self.persist_state()
            }
            Message::OpenLink(uri) => {
                let desktop = self.services.ports.desktop.clone();
                Task::perform(async move { desktop.open_uri(&uri) }, Message::from_result)
            }
            Message::OpenRowDatePicker(target, key) => {
                let todo = Todo::parse(&target.raw);
                let current = match key {
                    DateKey::Due => todo.due(),
                    DateKey::Threshold => todo.threshold(),
                };
                self.dialog = Some(Dialog::RowDate {
                    target,
                    calendar: Calendar::new(key, current, self.today),
                });
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub(crate) fn selected_todo(&self) -> Option<&ListedTodo> {
        self.listing.todos().nth(self.selected?)
    }

    pub(crate) fn select_row(&mut self, index: usize) -> Task<Message> {
        let count = self.listing.counts.visible;
        if count == 0 {
            self.selected = None;
            return Task::none();
        }
        let index = index.min(count - 1);
        self.selected = Some(index);
        self.row_menu = None;
        // Keep the selected row in view: scroll proportionally to its index.
        let offset = if count > 1 {
            index as f32 / (count - 1) as f32
        } else {
            0.0
        };
        operation::snap_to(
            view::LIST_ID,
            iced::widget::scrollable::RelativeOffset { x: 0.0, y: offset },
        )
    }

    pub(crate) fn move_selection(&mut self, step: isize) -> Task<Message> {
        let next = match self.selected {
            Some(index) => index.saturating_add_signed(step),
            None => 0,
        };
        self.select_row(next)
    }

    pub(super) fn delete_todo(&mut self, target: TodoRef) -> Task<Message> {
        self.change_active_file(move |service, path, options| service.delete(path, &target, options))
    }

    fn archive_one(&mut self, target: TodoRef) -> Task<Message> {
        self.row_menu = None;
        let Some(entry) = self.state.active_entry().cloned() else {
            return Task::none();
        };
        let Some(done_path) = entry.done_path else {
            let index = self.state.active_file;
            let notice = self.toast(tr("done_file_needed"));
            return Task::batch([notice, Task::done(Message::ChangeDoneFile(index))]);
        };
        let archive = self
            .change_active_file(move |service, path, options| service.archive_one(path, &done_path, &target, options));
        Task::batch([archive, self.toast(tr("archived_one"))])
    }
}
