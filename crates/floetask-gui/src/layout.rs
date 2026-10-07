//! Which side panels fit next to a view that needs a minimum width.
//!
//! The main view (list, board or calendar) must never get narrower than
//! [`MAIN_MIN_WIDTH`]. When the window is too small for it and every open
//! panel, panels are
//! hidden for as long as it stays too small, in a fixed order: the filter
//! drawer first, then the file drawer, then the undated panel. Their open
//! state is kept, so they come back when the window grows again.
//!
//! Kept free of widgets so it can be unit-tested.

/// Narrowest the main view gets: the list, the board, or the calendar's
/// week with all seven columns.
pub const MAIN_MIN_WIDTH: f32 = 760.0;
/// Left and right padding of the page.
pub const PAGE_PADDING: f32 = 32.0;
/// Space between the main area and a panel next to it.
pub const PANEL_SPACING: f32 = 12.0;
pub const FILES_DRAWER_WIDTH: f32 = 248.0;
pub const FILTER_DRAWER_WIDTH: f32 = 300.0;
pub const UNDATED_PANEL_WIDTH: f32 = 300.0;

/// A side panel the user can open over the main view when there is no room
/// for it beside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Files,
    Filters,
}

/// Which panels the user has open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Panels {
    pub files: bool,
    pub filters: bool,
    pub undated: bool,
}

/// The open panels that fit next to a view needing `min_main` pixels in a
/// window `width` pixels wide (logical, after zoom).
pub fn fit(panels: Panels, width: f32, min_main: f32) -> Panels {
    let needed = |p: Panels| {
        let side = |open: bool, size: f32| if open { size + PANEL_SPACING } else { 0.0 };
        PAGE_PADDING
            + min_main
            + side(p.files, FILES_DRAWER_WIDTH)
            + side(p.filters, FILTER_DRAWER_WIDTH)
            + side(p.undated, UNDATED_PANEL_WIDTH)
    };
    let mut shown = panels;
    for hide in [
        |p: &mut Panels| p.filters = false,
        |p: &mut Panels| p.files = false,
        |p: &mut Panels| p.undated = false,
    ] {
        if needed(shown) <= width {
            break;
        }
        hide(&mut shown);
    }
    shown
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: Panels = Panels {
        files: true,
        filters: true,
        undated: true,
    };

    #[test]
    fn wide_windows_keep_every_panel() {
        // 32 + 760 + 260 + 312 + 312
        assert_eq!(fit(ALL, 1676.0, MAIN_MIN_WIDTH), ALL);
    }

    #[test]
    fn panels_close_filter_first_then_files_then_undated() {
        let shown = fit(ALL, 1500.0, MAIN_MIN_WIDTH);
        assert_eq!((shown.filters, shown.files, shown.undated), (false, true, true));
        let shown = fit(ALL, 1200.0, MAIN_MIN_WIDTH);
        assert_eq!((shown.filters, shown.files, shown.undated), (false, false, true));
        let shown = fit(ALL, 900.0, MAIN_MIN_WIDTH);
        assert_eq!((shown.filters, shown.files, shown.undated), (false, false, false));
    }

    #[test]
    fn closed_panels_stay_closed_and_nothing_hides_without_need() {
        let files_only = Panels {
            files: true,
            filters: false,
            undated: false,
        };
        assert_eq!(fit(files_only, 1100.0, MAIN_MIN_WIDTH), files_only);
        assert!(!fit(files_only, 1000.0, MAIN_MIN_WIDTH).files);
    }
}
