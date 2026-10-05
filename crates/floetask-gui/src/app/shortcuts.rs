//! Keyboard shortcuts. Only keys no widget consumed arrive here, so typing
//! in a text field never triggers list actions.

use iced::Task;
use iced::keyboard::{Key, Modifiers, key::Named};

use floetask_application::TodoRef;

use super::{Floetask, Message, ViewToggle};

impl Floetask {
    pub(super) fn on_key(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
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
            Key::Named(Named::Enter) => self.on_selected(Message::OpenTodo),
            Key::Named(Named::Space) => self.on_selected(Message::ToggleComplete),
            Key::Named(Named::Delete) => self.on_selected(Message::AskDelete),
            Key::Named(Named::Backspace) if modifiers.command() => self.on_selected(Message::AskDelete),
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
            ("n", false, false) => Message::NewTodo,
            ("f", false, false) => Message::FocusSearch,
            ("f", false, true) => Message::ToggleSavedFilters,
            ("h", false, false) => Message::ViewToggle(ViewToggle::Completed, !self.state.view.show_completed),
            ("h", true, false) => Message::ToggleFilesDrawer,
            ("0", false, false) => Message::ResetFilters,
            ("a", true, false) => Message::ArchiveCompleted,
            ("o", false, false) => Message::OpenFileDialog,
            (",", false, false) => Message::OpenSettings,
            ("b", false, false) => Message::ToggleDrawer,
            ("d", true, false) => Message::ToggleTheme,
            ("w", false, false) | ("q", false, false) => Message::Quit,
            (digit, false, false) => {
                let number: usize = digit.parse().ok().filter(|n| (1..=9).contains(n))?;
                Message::SelectFile(number - 1)
            }
            _ => return None,
        })
    }

    fn escape(&mut self) -> Task<Message> {
        if let Some(super::Dialog::Editor(editor)) = &self.dialog
            && editor.calendar.is_some()
        {
            return self.update(Message::EditorCloseCalendar);
        }
        if self.dialog.is_some() {
            return self.close_dialog();
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
            return self.update(Message::ToggleDrawer);
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
