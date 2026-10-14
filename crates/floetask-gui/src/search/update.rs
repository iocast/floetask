//! The search bar and saved filters.

use iced::Task;
use iced::widget::operation;

use floetask_application::SavedFilter;
use floetask_domain::search::{Interpretation, Query};

use crate::app::{Dialog, Floetask, Message, Pending};
use crate::i18n::trf;
use crate::{search, view};

impl Floetask {
    pub(crate) fn update_search(&mut self, message: search::Message) -> Task<Message> {
        match message {
            search::Message::Focus => operation::focus(view::SEARCH_ID),
            search::Message::Input(input) => {
                self.set_search(input);
                Task::none()
            }
            search::Message::SubmitAsTodo => {
                let text = self.search.input.trim().to_owned();
                if text.is_empty() {
                    return Task::none();
                }
                self.set_search(String::new());
                let input = self.input_options();
                self.change_active_file(move |service, path, options| service.add(path, &text, &input, options))
            }
            search::Message::ToggleSaved => {
                self.search.saved_open = !self.search.saved_open;
                Task::none()
            }
            search::Message::AskSaveFilter => {
                self.dialog = Some(Dialog::SaveFilter { name: String::new() });
                Task::none()
            }
            search::Message::SaveFilterName(text) => {
                if let Some(Dialog::SaveFilter { name }) = &mut self.dialog {
                    *name = text;
                }
                Task::none()
            }
            search::Message::ApplySaved(index) => {
                if let Some(filter) = self.saved_filters.get(index) {
                    let query = filter.query.clone();
                    self.search.saved_open = false;
                    self.set_search(query);
                }
                Task::none()
            }
            search::Message::AskDeleteSaved(index) => {
                if let Some(filter) = self.saved_filters.get(index) {
                    self.dialog = Some(Dialog::Confirm {
                        message: trf("delete_filter_confirm", &[("name", filter.name.as_str().into())]),
                        detail: Some(filter.query.clone()),
                        pending: Pending::DeleteSavedFilter(index),
                    });
                }
                Task::none()
            }
            search::Message::ToggleSuppress(index) => {
                if let Some(filter) = self.saved_filters.get_mut(index) {
                    filter.suppress_notifications = !filter.suppress_notifications;
                }
                self.store_saved_filters()
            }
            search::Message::SavedFiltersLoaded(result) => match result {
                Ok(filters) => {
                    self.saved_filters = filters;
                    Task::none()
                }
                Err(error) => self.report(&error),
            },
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

    /// Escape in the list clears the search.
    pub(crate) fn clear_search(&mut self) -> Task<Message> {
        self.search.saved_open = false;
        self.set_search(String::new());
        Task::none()
    }

    pub(crate) fn save_filter(&mut self, name: String) -> Task<Message> {
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

    pub(crate) fn delete_saved_filter(&mut self, index: usize) -> Task<Message> {
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
