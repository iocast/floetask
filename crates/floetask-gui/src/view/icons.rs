//! floetask's line icons: 24×24, 2px strokes, drawn for this app. They are
//! tinted at render time, so one shape serves every theme and state.

use std::sync::LazyLock;

use iced::widget::svg;
use iced::{Color, Element};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Plus,
    Search,
    Filter,
    Sun,
    Moon,
    Settings,
    Folder,
    Archive,
    Minimize,
    Maximize,
    Close,
    Calendar,
    Repeat,
    Timer,
    ChevronDown,
    ChevronUp,
    ChevronRight,
    ChevronLeft,
    Star,
    More,
    Bell,
    BellOff,
    Eye,
    EyeOff,
    Pencil,
    Copy,
    Trash,
    ArrowUp,
    ArrowDown,
    Link,
}

impl Icon {
    const ALL: [Icon; 30] = [
        Icon::Plus,
        Icon::Search,
        Icon::Filter,
        Icon::Sun,
        Icon::Moon,
        Icon::Settings,
        Icon::Folder,
        Icon::Archive,
        Icon::Minimize,
        Icon::Maximize,
        Icon::Close,
        Icon::Calendar,
        Icon::Repeat,
        Icon::Timer,
        Icon::ChevronDown,
        Icon::ChevronUp,
        Icon::ChevronRight,
        Icon::ChevronLeft,
        Icon::Star,
        Icon::More,
        Icon::Bell,
        Icon::BellOff,
        Icon::Eye,
        Icon::EyeOff,
        Icon::Pencil,
        Icon::Copy,
        Icon::Trash,
        Icon::ArrowUp,
        Icon::ArrowDown,
        Icon::Link,
    ];

    /// The SVG body (elements inside the 24×24 canvas).
    fn shape(self) -> &'static str {
        match self {
            Icon::Plus => r#"<path d="M12 5v14M5 12h14"/>"#,
            Icon::Search => r#"<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.6-3.6"/>"#,
            Icon::Filter => r#"<path d="M4 6h16M7 12h10M10 18h4"/>"#,
            Icon::Sun => {
                r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>"#
            }
            Icon::Moon => r#"<path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z"/>"#,
            Icon::Settings => {
                r#"<path d="M18.66 9.26 L21.27 9.93 L21.27 14.07 L18.66 14.74 L18.64 14.77 L20.02 17.09 L17.09 20.02 L14.77 18.64 L14.74 18.66 L14.07 21.27 L9.93 21.27 L9.26 18.66 L9.23 18.64 L6.91 20.02 L3.98 17.09 L5.36 14.77 L5.34 14.74 L2.73 14.07 L2.73 9.93 L5.34 9.26 L5.36 9.23 L3.98 6.91 L6.91 3.98 L9.23 5.36 L9.26 5.34 L9.93 2.73 L14.07 2.73 L14.74 5.34 L14.77 5.36 L17.09 3.98 L20.02 6.91 L18.64 9.23Z"/><circle cx="12" cy="12" r="3"/>"#
            }
            Icon::Folder => r#"<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>"#,
            Icon::Archive => {
                r#"<rect x="3" y="4" width="18" height="4" rx="1"/><path d="M5 8v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8M10 12h4"/>"#
            }
            Icon::Minimize => r#"<path d="M6 12h12"/>"#,
            Icon::Maximize => r#"<rect x="6" y="6" width="12" height="12" rx="1.5"/>"#,
            Icon::Close => r#"<path d="M6 6l12 12M18 6L6 18"/>"#,
            Icon::Calendar => r#"<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18M8 3v4M16 3v4"/>"#,
            Icon::Repeat => {
                r#"<path d="M17 2l4 4-4 4M3 11V9a3 3 0 0 1 3-3h15M7 22l-4-4 4-4M21 13v2a3 3 0 0 1-3 3H3"/>"#
            }
            Icon::Timer => r#"<circle cx="12" cy="13" r="8"/><path d="M12 9v4l2.5 1.5M10 2h4"/>"#,
            Icon::ChevronDown => r#"<path d="M6 9l6 6 6-6"/>"#,
            Icon::ChevronUp => r#"<path d="M6 15l6-6 6 6"/>"#,
            Icon::ChevronRight => r#"<path d="M9 6l6 6-6 6"/>"#,
            Icon::ChevronLeft => r#"<path d="M15 6l-6 6 6 6"/>"#,
            Icon::Star => r#"<path d="M12 3l2.8 5.8 6.2.8-4.6 4.3 1.2 6.2L12 17l-5.6 3.1 1.2-6.2L3 9.6l6.2-.8z"/>"#,
            Icon::More => {
                r#"<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>"#
            }
            Icon::Bell => r#"<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9M10.3 21a1.9 1.9 0 0 0 3.4 0"/>"#,
            Icon::BellOff => {
                r#"<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9M10.3 21a1.9 1.9 0 0 0 3.4 0M3 3l18 18"/>"#
            }
            Icon::Eye => r#"<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>"#,
            Icon::EyeOff => {
                r#"<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/><path d="M3 3l18 18"/>"#
            }
            Icon::Pencil => r#"<path d="M4 20h4L19 9l-4-4L4 16zM13.5 6.5l4 4"/>"#,
            Icon::Copy => r#"<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h8"/>"#,
            Icon::Trash => r#"<path d="M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3"/>"#,
            Icon::ArrowUp => r#"<path d="M12 19V5M6 11l6-6 6 6"/>"#,
            Icon::ArrowDown => r#"<path d="M12 5v14M6 13l6 6 6-6"/>"#,
            Icon::Link => r#"<path d="M14 4h6v6M20 4l-9 9M18 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4"/>"#,
        }
    }

    fn handle(self) -> &'static svg::Handle {
        static HANDLES: LazyLock<Vec<svg::Handle>> = LazyLock::new(|| {
            Icon::ALL
                .iter()
                .map(|icon| {
                    let document = format!(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
                        icon.shape()
                    );
                    svg::Handle::from_memory(document.into_bytes())
                })
                .collect()
        });
        let index = Icon::ALL
            .iter()
            .position(|icon| *icon == self)
            .expect("every icon is listed");
        &HANDLES[index]
    }
}

/// An icon at `size` pixels in `color`.
pub fn icon<'a, Message: 'a>(icon: Icon, size: f32, color: Color) -> Element<'a, Message> {
    svg(icon.handle().clone())
        .width(size)
        .height(size)
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}
