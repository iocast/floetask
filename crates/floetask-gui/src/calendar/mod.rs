//! The calendar view: todos on a day, week or month by due date, dragged to
//! another day to move their `due:`.

mod update;
mod view;

use floetask_domain::Date;
use floetask_domain::calendar::CalendarMode;

use crate::app::Target;

pub(crate) use view::view;

#[derive(Debug, Clone)]
pub enum Message {
    Mode(CalendarMode),
    /// Moves the calendar by this many days, weeks or months.
    Step(i64),
    Today,
    /// Opens one day in Day mode.
    OpenDay(Date),
    /// A todo is pressed on a day, or in the undated panel (`None`): the
    /// start of a drag or a click.
    Press(Target, Option<Date>),
    /// Shows or hides the panel of todos without a due date.
    ToggleUndated,
    /// The cursor entered or left the undated panel.
    HoverUndated(bool),
    Hover(Date, bool),
    /// The cursor left a todo on the calendar: a press there became a drag.
    CardLeft,
    Release,
}
