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

use super::{Floetask, Message};

const TICK: Duration = Duration::from_secs(60);

pub(super) fn all(app: &Floetask) -> Subscription<Message> {
    Subscription::batch([
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => Some(Message::KeyPressed(key, modifiers)),
            keyboard::Event::ModifiersChanged(modifiers) => Some(Message::ModifiersChanged(modifiers)),
            _ => None,
        }),
        event::listen_with(|event, _status, id| match event {
            iced::Event::Window(
                event @ (window::Event::FileDropped(_)
                | window::Event::Resized(_)
                | window::Event::Moved(_)
                | window::Event::CloseRequested),
            ) => Some(Message::Window(id, event)),
            _ => None,
        }),
        iced::system::theme_changes().map(|mode| Message::SystemDark(mode == iced::theme::Mode::Dark)),
        iced::time::every(TICK).map(|_| Message::Tick),
        watch(app),
    ])
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
