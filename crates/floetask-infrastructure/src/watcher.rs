//! File watching with debouncing, so sync tools (Syncthing, Dropbox) that
//! write files in several steps cause one reload instead of flicker.
//!
//! Parent directories are watched rather than the files themselves: many
//! editors and sync tools replace a file by renaming a new one over it,
//! which would silently end a watch on the old file.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify_debouncer_full::notify::{self, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdCache, NoCache, RecommendedCache, new_debouncer_opt};

use floetask_application::ports::{FileWatcher, WatchGuard};
use floetask_application::{AppError, WatcherOptions};

pub struct NotifyFileWatcher;

impl FileWatcher for NotifyFileWatcher {
    fn watch(
        &self,
        paths: &[PathBuf],
        options: &WatcherOptions,
        on_change: Box<dyn Fn(PathBuf) + Send + Sync>,
    ) -> Result<WatchGuard, AppError> {
        // Normalised path -> path as the caller knows it.
        let watched: BTreeMap<PathBuf, PathBuf> = paths.iter().map(|p| (normalize(p), p.clone())).collect();
        let handler = {
            let watched = watched.clone();
            move |result: DebounceEventResult| {
                let Ok(events) = result else { return };
                let changed: BTreeSet<PathBuf> = events
                    .iter()
                    .flat_map(|event| event.paths.iter())
                    .map(|path| normalize(path))
                    .filter_map(|path| watched.get(&path).cloned())
                    .collect();
                for path in changed {
                    on_change(path);
                }
            }
        };
        let timeout = Duration::from_millis(options.debounce_ms.max(1));
        let directories: BTreeSet<PathBuf> =
            watched.keys().filter_map(|p| p.parent().map(Path::to_path_buf)).collect();

        if options.polling {
            let config = notify::Config::default()
                .with_poll_interval(Duration::from_millis(options.poll_interval_ms.max(50)));
            let debouncer = new_debouncer_opt::<_, PollWatcher, NoCache>(timeout, None, handler, NoCache, config)
                .map_err(watch_error)?;
            start(debouncer, &directories)
        } else {
            let debouncer = new_debouncer_opt::<_, RecommendedWatcher, RecommendedCache>(
                timeout,
                None,
                handler,
                RecommendedCache::new(),
                notify::Config::default(),
            )
            .map_err(watch_error)?;
            start(debouncer, &directories)
        }
    }
}

fn start<W, C>(mut debouncer: Debouncer<W, C>, directories: &BTreeSet<PathBuf>) -> Result<WatchGuard, AppError>
where
    W: Watcher + Send + 'static,
    C: FileIdCache + Send + 'static,
{
    for directory in directories {
        // A missing directory is not fatal: the other files are still watched.
        if directory.exists() {
            debouncer
                .watch(directory, RecursiveMode::NonRecursive)
                .map_err(|e| AppError::io(directory, e))?;
        }
    }
    Ok(WatchGuard(Box::new(debouncer)))
}

fn normalize(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn watch_error(error: notify::Error) -> AppError {
    AppError::Other(format!("file watcher: {error}"))
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn reports_changes_to_watched_files_only() {
        let dir = tempfile::tempdir().unwrap();
        let todo = dir.path().join("todo.txt");
        let other = dir.path().join("other.txt");
        std::fs::write(&todo, "a\n").unwrap();
        let (sender, receiver) = mpsc::channel();
        let sender = std::sync::Mutex::new(sender);
        let options = WatcherOptions { debounce_ms: 50, ..WatcherOptions::default() };
        let _guard = NotifyFileWatcher
            .watch(
                &[todo.clone()],
                &options,
                Box::new(move |path| {
                    let _ = sender.lock().unwrap().send(path);
                }),
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(100));
        std::fs::write(&other, "x\n").unwrap();
        std::fs::write(&todo, "b\n").unwrap();
        let changed = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(changed, todo);
    }
}
