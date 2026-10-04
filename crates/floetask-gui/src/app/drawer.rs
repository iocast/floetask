//! The drawer: attribute filters, view toggles and sorting.

use iced::Task;

use floetask_domain::listing::Attribute;

use super::{Dialog, Floetask, Message, Pending, ViewToggle};
use crate::i18n::trf;

impl Floetask {
    pub(super) fn update_drawer(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleDrawer => {
                self.state.drawer_open = !self.state.drawer_open;
                return self.persist_state();
            }
            Message::DrawerTab(tab) => {
                self.state.drawer_tab = tab;
                return self.persist_state();
            }
            Message::ToggleSection(attribute) => {
                self.state.toggle_section(attribute);
                return self.persist_state();
            }
            Message::AskRename(attribute, value) => {
                self.dialog = Some(Dialog::Rename {
                    attribute,
                    input: value.clone(),
                    from: value,
                });
                return Task::none();
            }
            Message::RenameInput(text) => {
                if let Some(Dialog::Rename { input, .. }) = &mut self.dialog {
                    *input = text;
                }
                return Task::none();
            }
            Message::AskRemoveValue(attribute, value) => {
                self.dialog = Some(Dialog::Confirm {
                    message: trf("remove_value_confirm", &[&tag(attribute, &value)]),
                    detail: None,
                    pending: Pending::RemoveValue(attribute, value),
                });
                return Task::none();
            }
            _ => {}
        }

        // Everything below changes what is visible.
        let view = &mut self.state.view;
        let sorting = &mut self.state.sorting;
        match message {
            Message::FilterValue(attribute, value, exclude) => view.toggle_filter(attribute, &value, exclude),
            Message::ToggleCategory(attribute) => view.toggle_hidden_category(attribute),
            Message::ViewToggle(toggle, on) => match toggle {
                ViewToggle::Completed => view.show_completed = on,
                ViewToggle::Hidden => view.show_hidden = on,
                ViewToggle::HiddenAttributes => view.show_hidden_attributes = on,
                ViewToggle::FutureThreshold => view.show_future_threshold = on,
                ViewToggle::FutureDue => view.show_future_due = on,
            },
            Message::MoveSort(index, step) => sorting.move_criterion(index, step),
            Message::InvertSort(index) => sorting.toggle_direction(index),
            Message::FileOrder(on) => sorting.file_order = on,
            Message::CompletedLast(on) => sorting.completed_last = on,
            Message::ResetFilters => {
                view.clear_filters();
                self.search.input.clear();
                self.search.query = floetask_domain::search::Query::Empty;
            }
            _ => return Task::none(),
        }
        self.refresh();
        self.persist_state()
    }

    pub(super) fn rename_value(&mut self, attribute: Attribute, from: String, to: String) -> Task<Message> {
        let Some(path) = self.active_path() else {
            return Task::none();
        };
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        self.rename_filters(attribute, &from, Some(to.trim()));
        Task::perform(
            async move {
                service
                    .rename_attribute(&path, attribute, &from, &to, &options)
                    .map(|r| (path, r))
            },
            |result| match result {
                Ok((path, outcome)) => Message::SavedCount(path, Ok(outcome), "renamed"),
                Err(error) => Message::Failed(error),
            },
        )
    }

    pub(super) fn remove_value(&mut self, attribute: Attribute, value: String) -> Task<Message> {
        let Some(path) = self.active_path() else {
            return Task::none();
        };
        let service = self.services.todo_files.clone();
        let options = self.file_options();
        self.rename_filters(attribute, &value, None);
        Task::perform(
            async move {
                service
                    .remove_attribute(&path, attribute, &value, &options)
                    .map(|r| (path, r))
            },
            |result| match result {
                Ok((path, outcome)) => Message::SavedCount(path, Ok(outcome), "renamed"),
                Err(error) => Message::Failed(error),
            },
        )
    }

    /// Keeps active drawer filters pointing at a renamed or removed value.
    fn rename_filters(&mut self, attribute: Attribute, from: &str, to: Option<&str>) {
        let filters = &mut self.state.view.filters;
        match to {
            Some(to) => filters
                .iter_mut()
                .filter(|f| f.attribute == attribute && f.value == from)
                .for_each(|f| f.value = to.to_owned()),
            None => filters.retain(|f| !(f.attribute == attribute && f.value == from)),
        }
    }
}

/// `+name` or `@name`, as the user writes it.
pub(crate) fn tag(attribute: Attribute, value: &str) -> String {
    match attribute {
        Attribute::Projects => format!("+{value}"),
        Attribute::Contexts => format!("@{value}"),
        _ => value.to_owned(),
    }
}
