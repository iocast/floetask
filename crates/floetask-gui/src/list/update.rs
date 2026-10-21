//! Row interactions in the todo list.

use iced::Task;
use iced::widget::operation;

use floetask_application::NoteService;
use floetask_domain::NoteLink;
use floetask_domain::listing::ListedTodo;
use floetask_domain::todo::Todo;

use crate::app::{Dialog, Floetask, Message, Pending, Target};
use crate::editor::{DateKey, DatePicker, Editor};
use crate::i18n::{tr, trf};
use crate::{editor, files, list, note_window, view};

impl Floetask {
    pub(crate) fn update_list(&mut self, message: list::Message) -> Task<Message> {
        match message {
            list::Message::OpenTodo(target) => {
                let todo = Todo::parse(&target.todo.raw);
                self.row_menu = None;
                self.dialog = Some(Dialog::Editor(Box::new(Editor::new(
                    target.file.clone(),
                    Some(target.todo.clone()),
                    &todo.editable_text(),
                    self.colors(),
                ))));
                let focus = operation::focus(view::EDITOR_ID);
                // Fill the notes field from the note file, if there is one.
                match todo.note_link().and_then(Result::ok) {
                    Some(link) => {
                        let notes = self.services.notes.clone();
                        let path = target.file.clone();
                        let load = Task::perform(async move { notes.read(&path, &link) }, move |result| {
                            Message::Editor(editor::Message::NoteLoaded(target.clone(), result))
                        });
                        Task::batch([focus, load])
                    }
                    None => focus,
                }
            }
            list::Message::ToggleComplete(Target { file, todo }) => self
                .change_file(file, move |service, path, options| {
                    service.toggle_complete(path, &todo, options)
                }),
            list::Message::RowMenu(id) => {
                self.row_menu = if self.row_menu == id { None } else { id };
                Task::none()
            }
            list::Message::Copy(raw) => {
                self.row_menu = None;
                Task::batch([iced::clipboard::write(raw), self.toast(tr("copied"))])
            }
            list::Message::ArchiveOne(target) => self.archive_one(target),
            list::Message::AskDelete(target) => {
                self.row_menu = None;
                self.dialog = Some(Dialog::Confirm {
                    message: tr("delete_confirm").to_owned(),
                    detail: Some(Todo::parse(&target.todo.raw).editable_text()),
                    pending: Pending::DeleteTodo(target),
                });
                Task::none()
            }
            list::Message::ToggleGroup(key) => {
                self.state.toggle_list_group(&key);
                // Selection indexes count only rows that are shown.
                self.selected = None;
                self.persist_state()
            }
            list::Message::ChipFilter(attribute, value) => {
                self.state.view.toggle_filter(attribute, &value, false);
                self.refresh();
                self.persist_state()
            }
            list::Message::OpenNote(target) => {
                let path = target.file;
                let service = self.services.notes.clone();
                let todo = Todo::parse(&target.todo.raw);
                let todo_file = path.clone();
                Task::perform(async move { service.load_or_create(&path, &todo) }, move |result| {
                    Message::NoteWindow(note_window::Message::Loaded(todo_file.clone(), result))
                })
            }
            list::Message::OpenDatePicker(target, key) => {
                let todo = Todo::parse(&target.todo.raw);
                let current = match key {
                    DateKey::Due => todo.due(),
                    DateKey::Threshold => todo.threshold(),
                };
                self.dialog = Some(Dialog::RowDate {
                    target,
                    calendar: DatePicker::new(key, current, self.today),
                });
                Task::none()
            }
            list::Message::Hover(id) => {
                self.hovered = id;
                Task::none()
            }
        }
    }

    /// Todos in the list that can be selected: those of expanded groups, in
    /// screen order. The selection index counts these.
    pub(crate) fn selectable_todos(&self) -> impl Iterator<Item = &ListedTodo> {
        self.listing
            .groups
            .iter()
            .filter(|group| !self.state.collapsed_list_groups.contains(&self.list_group_key(group)))
            .flat_map(|group| group.todos.iter())
    }

    pub(crate) fn selected_todo(&self) -> Option<&ListedTodo> {
        self.selectable_todos().nth(self.selected?)
    }

    pub(crate) fn select_row(&mut self, index: usize) -> Task<Message> {
        let count = self.selectable_todos().count();
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

    /// Deletes a todo. Its note file is kept; if no other todo links to it
    /// any more, a toast says so.
    pub(crate) fn delete_todo(&mut self, target: Target) -> Task<Message> {
        let Target { file, todo } = target;
        let orphan = self
            .documents
            .get(&file)
            .and_then(|document| NoteService::orphaned_by_delete(document, &todo))
            .map(|link| match link {
                NoteLink::Folder(name) => format!("{}/{name}", NoteService::folder_name(&file)),
                NoteLink::External(path) => path,
            });
        let delete = self.change_file(file, move |service, path, options| service.delete(path, &todo, options));
        match orphan {
            Some(name) => Task::batch([delete, self.toast(trf("note_orphaned", &[("note", name.into())]))]),
            None => delete,
        }
    }

    /// Moves one todo to the done file of the file it is in, asking for a
    /// done file first when that file has none.
    fn archive_one(&mut self, target: Target) -> Task<Message> {
        self.row_menu = None;
        let Some(index) = self.state.files.iter().position(|entry| entry.path == target.file) else {
            return Task::none();
        };
        let Some(done_path) = self.state.files[index].done_path.clone() else {
            let notice = self.toast(tr("done_file_needed"));
            return Task::batch([
                notice,
                Task::done(Message::Files(files::Message::ChangeDoneFile(index))),
            ]);
        };
        let Target { file, todo } = target;
        let archive = self.change_file(file, move |service, path, options| {
            service.archive_one(path, &done_path, &todo, options)
        });
        Task::batch([archive, self.toast(tr("archived_one"))])
    }
}
