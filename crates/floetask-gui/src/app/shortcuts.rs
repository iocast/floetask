//! Keyboard shortcuts. Only keys no widget consumed arrive here, so typing
//! in a text field never triggers list actions.

use iced::Task;
use iced::keyboard::{Key, Modifiers, key::Named};

use floetask_application::{MainView, TodoRef};

use super::{Floetask, Message};
use crate::drawer::ViewToggle;
use crate::{drawer, editor, files, list, note_window, search, settings};

impl Floetask {
    pub(super) fn on_key(&mut self, window: iced::window::Id, key: Key, modifiers: Modifiers) -> Task<Message> {
        // The note window has one shortcut of its own; the list's would act
        // on a window the user is not looking at.
        if self.note_window.as_ref().is_some_and(|note| note.id == window) {
            return match key.as_ref() {
                Key::Character("s") if modifiers.command() => {
                    self.update(Message::NoteWindow(note_window::Message::Save))
                }
                _ => Task::none(),
            };
        }
        if let Some(message) = self.shortcut(&key, modifiers) {
            return self.update(message);
        }
        match key.as_ref() {
            Key::Named(Named::Escape) => self.escape(),
            _ if self.dialog.is_some() => match key.as_ref() {
                Key::Named(Named::Enter) if !matches!(self.dialog, Some(super::Dialog::Editor(_))) => {
                    self.update(Message::ConfirmDialog)
                }
                _ => Task::none(),
            },
            Key::Named(Named::ArrowDown) => self.move_selection(1),
            Key::Named(Named::ArrowUp) => self.move_selection(-1),
            Key::Named(Named::Enter) => self.on_selected(|target| Message::List(list::Message::OpenTodo(target))),
            Key::Named(Named::Space) => self.on_selected(|target| Message::List(list::Message::ToggleComplete(target))),
            Key::Named(Named::Delete) => self.on_selected(|target| Message::List(list::Message::AskDelete(target))),
            Key::Named(Named::Backspace) if modifiers.command() => {
                self.on_selected(|target| Message::List(list::Message::AskDelete(target)))
            }
            _ => Task::none(),
        }
    }

    /// Global shortcuts with the command key (Ctrl, or Cmd on macOS).
    fn shortcut(&self, key: &Key, modifiers: Modifiers) -> Option<Message> {
        if !modifiers.command() {
            return None;
        }
        let character = match key.as_ref() {
            Key::Character(c) => c.to_lowercase(),
            Key::Named(Named::Enter) => return self.dialog.is_some().then_some(Message::ConfirmDialog),
            _ => return None,
        };
        let alt = modifiers.alt();
        let shift = modifiers.shift();
        Some(match (character.as_str(), alt, shift) {
            ("n", false, false) => Message::Editor(editor::Message::New),
            ("f", false, false) => Message::Search(search::Message::Focus),
            ("f", false, true) => Message::Search(search::Message::ToggleSaved),
            ("h", false, false) => Message::Drawer(drawer::Message::ViewToggle(
                ViewToggle::Completed,
                !self.state.view.show_completed,
            )),
            ("h", true, false) => Message::Files(files::Message::ToggleDrawer),
            ("0", false, false) => Message::Drawer(drawer::Message::ResetFilters),
            ("a", true, false) => Message::Files(files::Message::ArchiveCompleted),
            ("o", false, false) => Message::Files(files::Message::OpenDialog),
            (",", false, false) => Message::Settings(settings::Message::Open),
            ("b", false, false) => Message::Drawer(drawer::Message::Toggle),
            ("b", true, false) => Message::ToggleMainView,
            ("c", true, false) => Message::ShowMainView(if self.state.main_view == MainView::Calendar {
                MainView::List
            } else {
                MainView::Calendar
            }),
            ("d", true, false) => Message::Settings(settings::Message::ToggleTheme),
            ("w", false, false) | ("q", false, false) => Message::Quit,
            (digit, false, false) => {
                let number: usize = digit.parse().ok().filter(|n| (1..=9).contains(n))?;
                Message::Files(files::Message::Select(*self.state.drawer_order().get(number - 1)?))
            }
            _ => return None,
        })
    }

    fn escape(&mut self) -> Task<Message> {
        if self.drag.take().is_some() || self.file_drag.take().is_some() {
            return Task::none();
        }
        if let Some(super::Dialog::Editor(editor)) = &self.dialog
            && editor.calendar.is_some()
        {
            return self.update(Message::Editor(editor::Message::CloseDatePicker));
        }
        if self.dialog.is_some() {
            return self.close_dialog();
        }
        if self.panel_overlay.take().is_some() {
            return Task::none();
        }
        if self.row_menu.is_some() || self.file_menu.is_some() {
            self.row_menu = None;
            self.file_menu = None;
            return Task::none();
        }
        if self.search.saved_open {
            self.search.saved_open = false;
            return Task::none();
        }
        if self.state.drawer_open {
            return self.update(Message::Drawer(drawer::Message::Toggle));
        }
        if !self.search.input.is_empty() {
            return self.clear_search();
        }
        Task::none()
    }

    fn on_selected(&mut self, action: fn(TodoRef) -> Message) -> Task<Message> {
        match self.selected_todo() {
            Some(entry) => {
                let target = TodoRef::new(entry.line, &entry.todo);
                self.update(action(target))
            }
            None => Task::none(),
        }
    }
}
