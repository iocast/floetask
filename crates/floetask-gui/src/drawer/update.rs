//! Filters, view toggles, sorting, and renaming or removing a value.

use std::path::PathBuf;

use iced::Task;

use floetask_application::AppError;
use floetask_domain::TodoDocument;
use floetask_domain::listing::Attribute;

use super::ViewToggle;
use crate::app::{Dialog, Floetask, Message, Pending};
use crate::i18n::trf;
use crate::{drawer, files};

impl Floetask {
    pub(crate) fn update_drawer(&mut self, message: drawer::Message) -> Task<Message> {
        match message {
            drawer::Message::Toggle => return self.toggle_panel(crate::layout::Panel::Filters),
            drawer::Message::Tab(tab) => {
                self.state.drawer_tab = tab;
                return self.persist_state();
            }
            drawer::Message::ToggleSection(attribute) => {
                self.state.toggle_section(attribute);
                return self.persist_state();
            }
            drawer::Message::AskRename(attribute, value) => {
                self.dialog = Some(Dialog::Rename {
                    attribute,
                    input: value.clone(),
                    from: value,
                });
                return Task::none();
            }
            drawer::Message::RenameInput(text) => {
                if let Some(Dialog::Rename { input, .. }) = &mut self.dialog {
                    *input = text;
                }
                return Task::none();
            }
            drawer::Message::AskRemoveValue(attribute, value) => {
                self.dialog = Some(Dialog::Confirm {
                    message: trf(
                        if self.state.shows_overview() {
                            "remove_value_confirm_all"
                        } else {
                            "remove_value_confirm"
                        },
                        &[("value", tag(attribute, &value).into())],
                    ),
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
            drawer::Message::FilterValue(attribute, value, exclude) => view.toggle_filter(attribute, &value, exclude),
            drawer::Message::ToggleCategory(attribute) => view.toggle_hidden_category(attribute),
            drawer::Message::ViewToggle(toggle, on) => match toggle {
                ViewToggle::Completed => view.show_completed = on,
                ViewToggle::Hidden => view.show_hidden = on,
                ViewToggle::HiddenAttributes => view.show_hidden_attributes = on,
                ViewToggle::HiddenStatuses => view.show_hidden_statuses = on,
                ViewToggle::FutureThreshold => view.show_future_threshold = on,
                ViewToggle::FutureDue => view.show_future_due = on,
            },
            drawer::Message::MoveSort(index, step) => sorting.move_criterion(index, step),
            drawer::Message::InvertSort(index) => sorting.toggle_direction(index),
            drawer::Message::FileOrder(on) => sorting.file_order = on,
            drawer::Message::CompletedLast(on) => sorting.completed_last = on,
            drawer::Message::ResetFilters => {
                view.clear_filters();
                self.search.input.clear();
                self.search.query = floetask_domain::search::Query::Empty;
            }
            _ => return Task::none(),
        }
        self.refresh();
        self.persist_state()
    }

    /// Renames a value in every shown file: the active one, or all of the
    /// overview's.
    pub(crate) fn rename_value(&mut self, attribute: Attribute, from: String, to: String) -> Task<Message> {
        self.rename_filters(attribute, &from, Some(to.trim()));
        Task::batch(self.shown_paths().into_iter().map(|path| {
            let service = self.services.todo_files.clone();
            let options = self.file_options();
            let (from, to) = (from.clone(), to.clone());
            Task::perform(
                async move {
                    service
                        .rename_attribute(&path, attribute, &from, &to, &options)
                        .map(|r| (path, r))
                },
                saved_count,
            )
        }))
    }

    /// Removes a value from every shown file, like `rename_value`.
    pub(crate) fn remove_value(&mut self, attribute: Attribute, value: String) -> Task<Message> {
        self.rename_filters(attribute, &value, None);
        Task::batch(self.shown_paths().into_iter().map(|path| {
            let service = self.services.todo_files.clone();
            let options = self.file_options();
            let value = value.clone();
            Task::perform(
                async move {
                    service
                        .remove_attribute(&path, attribute, &value, &options)
                        .map(|r| (path, r))
                },
                saved_count,
            )
        }))
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

/// Reports a rename or removal in one file.
fn saved_count(result: Result<(PathBuf, (TodoDocument, usize)), AppError>) -> Message {
    match result {
        Ok((path, outcome)) => Message::Files(files::Message::SavedCount(path, Ok(outcome), "renamed")),
        Err(error) => Message::Failed(error),
    }
}

/// `+name` or `@name`, as the user writes it.
fn tag(attribute: Attribute, value: &str) -> String {
    match attribute {
        Attribute::Projects => format!("+{value}"),
        Attribute::Contexts => format!("@{value}"),
        _ => value.to_owned(),
    }
}
