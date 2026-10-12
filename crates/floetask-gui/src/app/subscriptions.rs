//! Event sources: keyboard, window, system theme, file watcher and a
//! periodic tick for notifications and the date rollover.

use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use iced::futures::{SinkExt, StreamExt};
use iced::{Subscription, event, keyboard, window};

use floetask_application::WatcherOptions;
use floetask_application::ports::FileWatcher;

use super::{Dialog, Floetask, Message};

const TICK: Duration = Duration::from_secs(60);

pub(super) fn all(app: &Floetask) -> Subscription<Message> {
    Subscription::batch([
        // Keys no widget took, with the window they came from, since the
        // main window and the note window handle them differently.
        event::listen_with(|event, status, id| match (event, status) {
            (iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }), event::Status::Ignored) => {
                Some(Message::KeyPressed(id, key, modifiers))
            }
            (iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)), event::Status::Ignored) => {
                Some(Message::ModifiersChanged(modifiers))
            }
            (
                iced::Event::Window(
                    event @ (window::Event::FileDropped(_)
                    | window::Event::Resized(_)
                    | window::Event::Moved(_)
                    | window::Event::CloseRequested),
                ),
                _,
            ) => Some(Message::Window(id, event)),
            _ => None,
        }),
        iced::system::theme_changes().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark)),
        iced::time::every(TICK).map(|_| Message::Tick),
        watch(app),
        note_editors(app),
    ])
}

/// The notes editors' caret blink, and `Escape` from a focused one: the
/// editor takes the key, so the key listener above never sees it, yet in the
/// dialog it should still close the dialog.
fn note_editors(app: &Floetask) -> Subscription<Message> {
    let dialog = match &app.dialog {
        Some(Dialog::Editor(editor)) => Some(editor.note.state()),
        _ => None,
    };
    let window = app.note_window.as_ref().map(|window| window.note.state());
    let mut subscriptions = Vec::new();
    if let Some(state) = dialog {
        subscriptions.push(state.subscription().map(Message::EditorNoteAction));
    }
    if let Some(state) = window {
        subscriptions.push(state.subscription().map(Message::NoteWindowAction));
    }
    if dialog.is_some_and(|state| state.is_focused()) {
        subscriptions.push(event::listen_with(|event, status, id| match (event, status) {
            (
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }),
                event::Status::Captured,
            ) => Some(Message::NoteEditorEscape(id)),
            _ => None,
        }));
    }
    Subscription::batch(subscriptions)
}

/// What to watch. Hashed by generation and paths, so the subscription is
/// restarted whenever the set of files changes.
#[derive(Clone)]
struct WatchRequest {
    generation: u64,
    paths: Vec<PathBuf>,
    options: WatcherOptions,
    watcher: Arc<dyn FileWatcher>,
}

impl Hash for WatchRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.generation.hash(state);
        self.paths.hash(state);
        self.options.debounce_ms.hash(state);
        self.options.polling.hash(state);
        self.options.poll_interval_ms.hash(state);
    }
}

fn watch(app: &Floetask) -> Subscription<Message> {
    let mut paths: Vec<PathBuf> = app.state.files.iter().map(|entry| entry.path.clone()).collect();
    paths.push(app.services.ports.saved_filters.location());
    let request = WatchRequest {
        generation: app.watch_generation,
        paths,
        options: app.settings.watcher.clone(),
        watcher: app.services.ports.watcher.clone(),
    };
    Subscription::run_with(request, |request| {
        let request = request.clone();
        iced::stream::channel(32, async move |mut output| {
            let (sender, mut changes) = iced::futures::channel::mpsc::unbounded::<PathBuf>();
            let guard = request.watcher.watch(
                &request.paths,
                &request.options,
                Box::new(move |path| {
                    let _ = sender.unbounded_send(path);
                }),
            );
            let _guard = match guard {
                Ok(guard) => guard,
                Err(error) => {
                    let _ = output.send(Message::Failed(error)).await;
                    return;
                }
            };
            while let Some(path) = changes.next().await {
                if output.send(Message::FileChanged(path)).await.is_err() {
                    break;
                }
            }
        })
    })
}
