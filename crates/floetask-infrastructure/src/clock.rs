use floetask_application::ports::Clock;
use floetask_domain::Date;

/// Today in the local time zone.
pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> Date {
        chrono::Local::now().date_naive()
    }
}
