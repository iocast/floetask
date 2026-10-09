//! Due-date notifications.

use std::sync::Arc;

use floetask_domain::{Date, TodoDocument, WeekStart};

use crate::error::AppError;
use crate::ports::{Clock, NotificationLog, Notifier};
use crate::saved_filters::SavedFilter;

/// A todo that is due soon and was not notified about today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueNotification {
    /// De-duplication key: a hash of today's date and the todo text.
    pub key: String,
    pub body: String,
    pub due: Date,
    /// 0 = due today.
    pub days_left: i64,
}

#[derive(Clone)]
pub struct NotificationService {
    notifier: Arc<dyn Notifier>,
    log: Arc<dyn NotificationLog>,
    clock: Arc<dyn Clock>,
}

impl NotificationService {
    pub fn new(notifier: Arc<dyn Notifier>, log: Arc<dyn NotificationLog>, clock: Arc<dyn Clock>) -> Self {
        Self { notifier, log, clock }
    }

    /// Incomplete todos due from today up to `threshold_days` ahead, minus
    /// those matching a saved filter that suppresses notifications and those
    /// already notified today. Overdue todos are not included. `week_start`
    /// resolves filter phrases such as `due: end of week`.
    pub fn pending<'a>(
        &self,
        documents: impl IntoIterator<Item = &'a TodoDocument>,
        threshold_days: u8,
        saved_filters: &[SavedFilter],
        week_start: WeekStart,
    ) -> Vec<DueNotification> {
        let today = self.clock.today();
        let suppressing: Vec<&SavedFilter> = saved_filters.iter().filter(|f| f.suppress_notifications).collect();
        documents
            .into_iter()
            .flat_map(|document| document.todos())
            .filter_map(|entry| {
                let todo = entry.todo;
                let due = todo.due()?;
                let days_left = (due - today).num_days();
                let in_window = (0..=i64::from(threshold_days)).contains(&days_left);
                if !in_window || todo.is_complete() {
                    return None;
                }
                if suppressing.iter().any(|filter| filter.matches(todo, today, week_start)) {
                    return None;
                }
                let key = notification_key(today, todo.raw());
                if self.log.contains(&key) {
                    return None;
                }
                Some(DueNotification {
                    key,
                    body: todo.body().to_owned(),
                    due,
                    days_left,
                })
            })
            .collect()
    }

    /// Shows a notification and remembers it so it is not repeated today.
    pub fn send(&self, notification: &DueNotification, title: &str) -> Result<(), AppError> {
        self.notifier.notify(title, &notification.body)?;
        self.log.insert(&notification.key)
    }
}

/// FNV-1a: stable across Rust versions, unlike `DefaultHasher`.
fn notification_key(today: Date, raw: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in today.to_string().bytes().chain(raw.bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex;

    use super::*;
    use floetask_domain::date::parse_iso;

    struct FixedClock;
    impl Clock for FixedClock {
        fn today(&self) -> Date {
            parse_iso("2024-03-13").unwrap()
        }
    }

    #[derive(Default)]
    struct MemoryLog(Mutex<HashSet<String>>);
    impl NotificationLog for MemoryLog {
        fn contains(&self, key: &str) -> bool {
            self.0.lock().unwrap().contains(key)
        }
        fn insert(&self, key: &str) -> Result<(), AppError> {
            self.0.lock().unwrap().insert(key.to_owned());
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingNotifier(Mutex<Vec<String>>);
    impl Notifier for RecordingNotifier {
        fn notify(&self, title: &str, body: &str) -> Result<(), AppError> {
            self.0.lock().unwrap().push(format!("{title}: {body}"));
            Ok(())
        }
    }

    #[test]
    fn notifies_due_soon_once() {
        let notifier = Arc::new(RecordingNotifier::default());
        let service = NotificationService::new(notifier.clone(), Arc::new(MemoryLog::default()), Arc::new(FixedClock));
        let document = TodoDocument::parse(
            "today due:2024-03-13\nsoon due:2024-03-15\nlater due:2024-03-16\nlate due:2024-03-12\nx 2024-03-01 done due:2024-03-13\nquiet due:2024-03-13 +quiet\n",
            &[],
        );
        let filters = vec![SavedFilter {
            name: "q".into(),
            query: "+quiet".into(),
            suppress_notifications: true,
        }];
        let pending = service.pending([&document], 2, &filters, WeekStart::Monday);
        let bodies: Vec<_> = pending.iter().map(|n| n.body.as_str()).collect();
        assert_eq!(bodies, vec!["today", "soon"]);
        assert_eq!(pending[1].days_left, 2);

        for notification in &pending {
            service.send(notification, "Due").unwrap();
        }
        assert_eq!(notifier.0.lock().unwrap().len(), 2);
        assert!(service.pending([&document], 2, &filters, WeekStart::Monday).is_empty());
    }
}
