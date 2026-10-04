//! The search bar and saved filters.

use iced::Task;
use iced::widget::operation;

use floetask_application::SavedFilter;
use floetask_domain::search::{Interpretation, Query};

use super::{Dialog, Floetask, Message, Pending};
use crate::i18n::trf;
use crate::view;

impl Floetask {
    pub(super) fn update_search(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleSearch => {
                self.state.search_visible = !self.state.search_visible || !self.search.input.is_empty();
                let focus = if self.state.search_visible {
                    operation::focus(view::SEARCH_ID)
                } else {
                    Task::none()
                };
                Task::batch([focus, self.persist_state()])
            }
            Message::SearchInput(input) => {
                self.set_search(input);
                Task::none()
            }
            Message::SearchSubmitAsTodo => {
                let text = self.search.input.trim().to_owned();
                if text.is_empty() {
                    return Task::none();
                }
                self.set_search(String::new());
                let input = self.input_options();
                self.change_active_file(move |service, path, options| service.add(path, &text, &input, options))
            }
            Message::ToggleSavedFilters => {
                self.search.saved_open = !self.search.saved_open;
                Task::none()
            }
            Message::AskSaveFilter => {
                self.dialog = Some(Dialog::SaveFilter { name: String::new() });
                Task::none()
            }
            Message::SaveFilterName(text) => {
                if let Some(Dialog::SaveFilter { name }) = &mut self.dialog {
                    *name = text;
                }
                Task::none()
            }
            Message::ApplySavedFilter(index) => {
                if let Some(filter) = self.saved_filters.get(index) {
                    let query = filter.query.clone();
                    self.state.search_visible = true;
                    self.search.saved_open = false;
                    self.set_search(query);
                }
                Task::none()
            }
            Message::AskDeleteSavedFilter(index) => {
                if let Some(filter) = self.saved_filters.get(index) {
                    self.dialog = Some(Dialog::Confirm {
                        message: trf("delete_filter_confirm", &[&filter.name]),
                        detail: Some(filter.query.clone()),
                        pending: Pending::DeleteSavedFilter(index),
                    });
                }
                Task::none()
            }
            Message::ToggleSuppress(index) => {
                if let Some(filter) = self.saved_filters.get_mut(index) {
                    filter.suppress_notifications = !filter.suppress_notifications;
                }
                self.store_saved_filters()
            }
            _ => Task::none(),
        }
    }

    /// Updates the search text. An unfinished expression keeps the previous
    /// results on screen instead of flashing an empty list.
    pub(crate) fn set_search(&mut self, input: String) {
        if let Interpretation::Ready(query) = Query::interpret(&input) {
            self.search.query = query;
        }
        self.search.input = input;
        self.selected = None;
        self.refresh();
    }

    pub(crate) fn clear_or_hide_search(&mut self) -> Task<Message> {
        if self.search.input.is_empty() {
            self.state.search_visible = false;
            self.search.saved_open = false;
            self.persist_state()
        } else {
            self.set_search(String::new());
            Task::none()
        }
    }

    pub(super) fn save_filter(&mut self, name: String) -> Task<Message> {
        let name = name.trim().to_owned();
        if name.is_empty() || self.search.input.trim().is_empty() {
            return Task::none();
        }
        let suppress = self
            .saved_filters
            .iter()
            .find(|f| f.name == name)
            .is_some_and(|f| f.suppress_notifications);
        floetask_application::saved_filters::upsert(
            &mut self.saved_filters,
            SavedFilter {
                name,
                query: self.search.input.trim().to_owned(),
                suppress_notifications: suppress,
            },
        );
        self.store_saved_filters()
    }

    pub(super) fn delete_saved_filter(&mut self, index: usize) -> Task<Message> {
        if index < self.saved_filters.len() {
            self.saved_filters.remove(index);
        }
        self.store_saved_filters()
    }

    fn store_saved_filters(&self) -> Task<Message> {
        let store = self.services.ports.saved_filters.clone();
        let filters = self.saved_filters.clone();
        Task::perform(async move { store.save(&filters) }, Message::from_result)
    }
}
