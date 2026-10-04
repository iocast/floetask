//! Colours and widget styles.
//!
//! floetask has a light and a dark (near-black) theme. Both are built from
//! [`Colors`], which the user can override per mode in the colour file. All
//! widget styles live here so views stay declarative.

use iced::widget::{button, checkbox, container, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector, border};

use floetask_application::{ColorOverrides, PaletteOverrides};
use floetask_domain::Priority;

/// The colours of one theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colors {
    pub dark: bool,
    /// Window background, behind cards.
    pub background: Color,
    /// Cards, panels, dialogs.
    pub surface: Color,
    pub border: Color,
    pub hover: Color,
    pub text: Color,
    pub muted: Color,
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub priority_a: Color,
    pub priority_b: Color,
    pub priority_c: Color,
    pub priority_other: Color,
}

impl Colors {
    pub fn new(dark: bool, overrides: &ColorOverrides) -> Self {
        let defaults = if dark { Self::DARK } else { Self::LIGHT };
        let custom = if dark { &overrides.dark } else { &overrides.light };
        defaults.with_overrides(custom)
    }

    const LIGHT: Self = Self {
        dark: false,
        background: rgb(0xf4f5f7),
        surface: rgb(0xffffff),
        border: rgb(0xe3e5ea),
        hover: rgb(0xeceef2),
        text: rgb(0x1a1d23),
        muted: rgb(0x6b7280),
        primary: rgb(0x4f46e5),
        success: rgb(0x16a34a),
        warning: rgb(0xd97706),
        danger: rgb(0xe5484d),
        priority_a: rgb(0xef4444),
        priority_b: rgb(0xf59e0b),
        priority_c: rgb(0x10b981),
        priority_other: rgb(0x94a3b8),
    };

    const DARK: Self = Self {
        dark: true,
        background: rgb(0x0b0c0f),
        surface: rgb(0x15171c),
        border: rgb(0x262930),
        hover: rgb(0x1e2128),
        text: rgb(0xe8eaed),
        muted: rgb(0x8b919c),
        primary: rgb(0x818cf8),
        success: rgb(0x4ade80),
        warning: rgb(0xfbbf24),
        danger: rgb(0xf87171),
        priority_a: rgb(0xf87171),
        priority_b: rgb(0xfbbf24),
        priority_c: rgb(0x34d399),
        priority_other: rgb(0x64748b),
    };

    fn with_overrides(mut self, custom: &PaletteOverrides) -> Self {
        let apply = |target: &mut Color, value: &Option<String>| {
            if let Some(color) = value.as_deref().and_then(parse_hex) {
                *target = color;
            }
        };
        apply(&mut self.background, &custom.background);
        apply(&mut self.text, &custom.text);
        apply(&mut self.primary, &custom.primary);
        apply(&mut self.success, &custom.success);
        apply(&mut self.warning, &custom.warning);
        apply(&mut self.danger, &custom.danger);
        apply(&mut self.surface, &custom.navigation);
        apply(&mut self.priority_a, &custom.priority_a);
        apply(&mut self.priority_b, &custom.priority_b);
        apply(&mut self.priority_c, &custom.priority_c);
        apply(&mut self.priority_other, &custom.priority_other);
        self
    }

    pub fn theme(&self) -> Theme {
        Theme::custom(
            if self.dark { "floetask dark" } else { "floetask light" },
            iced::theme::Palette {
                background: self.surface,
                text: self.text,
                primary: self.primary,
                success: self.success,
                warning: self.warning,
                danger: self.danger,
            },
        )
    }

    /// A–C get their own colour, D–Z share one.
    pub fn priority(&self, priority: Priority) -> Color {
        match priority.letter() {
            'A' => self.priority_a,
            'B' => self.priority_b,
            'C' => self.priority_c,
            _ => self.priority_other,
        }
    }

    /// `color` faded onto the surface, for tinted backgrounds.
    pub fn tint(&self, color: Color, amount: f32) -> Color {
        mix(self.surface, color, amount)
    }
}

const fn rgb(hex: u32) -> Color {
    Color::from_rgb8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// Parses `#rrggbb` or `rrggbb`.
pub fn parse_hex(text: &str) -> Option<Color> {
    let hex = text.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    u32::from_str_radix(hex, 16).ok().map(rgb)
}

fn mix(a: Color, b: Color, amount: f32) -> Color {
    Color::from_rgb(
        a.r + (b.r - a.r) * amount,
        a.g + (b.g - a.g) * amount,
        a.b + (b.b - a.b) * amount,
    )
}

fn filled(color: Color) -> Option<Background> {
    Some(Background::Color(color))
}

// Containers

pub fn app(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(colors.background),
        text_color: Some(colors.text),
        ..container::Style::default()
    }
}

/// A floating panel: the drawer, menus, the search bar.
pub fn panel(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(colors.surface),
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: colors.border,
        },
        ..container::Style::default()
    }
}

/// A todo row. Hovered and selected rows are lifted a little.
pub fn row(colors: Colors, hovered: bool, selected: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(if hovered { colors.hover } else { colors.surface }),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: if selected { colors.primary } else { colors.border },
        },
        ..container::Style::default()
    }
}

/// The thin coloured bar marking a todo's priority.
pub fn accent(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(color),
        border: border::rounded(2),
        ..container::Style::default()
    }
}

pub fn card(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(colors.surface),
        text_color: Some(colors.text),
        border: Border {
            radius: 16.0.into(),
            width: 1.0,
            color: colors.border,
        },
        shadow: Shadow {
            color: Color::BLACK.scale_alpha(if colors.dark { 0.6 } else { 0.18 }),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 32.0,
        },
        ..container::Style::default()
    }
}

pub fn backdrop(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(Color::BLACK.scale_alpha(if colors.dark { 0.6 } else { 0.3 })),
        ..container::Style::default()
    }
}

pub fn toast(colors: Colors, error: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(if error { colors.danger } else { colors.text }),
        text_color: Some(colors.surface),
        border: border::rounded(10),
        shadow: Shadow {
            color: Color::BLACK.scale_alpha(0.25),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    }
}

pub fn tooltip(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(colors.text),
        text_color: Some(colors.surface),
        border: border::rounded(6),
        ..container::Style::default()
    }
}

/// Boxes showing raw todo.txt text.
pub fn code(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: filled(colors.background),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: colors.border,
        },
        ..container::Style::default()
    }
}

// Buttons

/// Borderless icon or text button; `active` tints it with the primary colour.
pub fn ghost(colors: Colors, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let background = match (active, status) {
            (true, _) => Some(colors.tint(colors.primary, 0.14)),
            (false, button::Status::Hovered | button::Status::Pressed) => Some(colors.hover),
            _ => None,
        };
        button::Style {
            background: background.map(Background::Color),
            text_color: if active { colors.primary } else { colors.text },
            border: border::rounded(8),
            ..button::Style::default()
        }
    }
}

/// Window controls; the close button turns red on hover.
pub fn window_control(colors: Colors, close: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let background = match (hovered, close) {
            (true, true) => Some(rgb(0xe81123)),
            (true, false) => Some(colors.hover),
            _ => None,
        };
        button::Style {
            background: background.map(Background::Color),
            text_color: if hovered && close { Color::WHITE } else { colors.text },
            ..button::Style::default()
        }
    }
}

pub fn primary(colors: Colors) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let base = colors.primary;
        let background = match status {
            button::Status::Hovered => mix(base, Color::WHITE, 0.12),
            button::Status::Pressed => mix(base, Color::BLACK, 0.1),
            button::Status::Disabled => base.scale_alpha(0.5),
            button::Status::Active => base,
        };
        button::Style {
            background: filled(background),
            text_color: if colors.dark { rgb(0x0b0c0f) } else { Color::WHITE },
            border: border::rounded(8),
            ..button::Style::default()
        }
    }
}

pub fn secondary(colors: Colors) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: filled(match status {
            button::Status::Hovered | button::Status::Pressed => colors.hover,
            _ => colors.surface,
        }),
        text_color: colors.text,
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: colors.border,
        },
        ..button::Style::default()
    }
}

pub fn danger(colors: Colors) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| button::Style {
        background: filled(match status {
            button::Status::Hovered | button::Status::Pressed => mix(colors.danger, Color::BLACK, 0.1),
            _ => colors.danger,
        }),
        text_color: Color::WHITE,
        border: border::rounded(8),
        ..button::Style::default()
    }
}

/// A file tab in the title bar.
pub fn tab(colors: Colors, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let background = match (active, status) {
            (true, _) => Some(colors.surface),
            (false, button::Status::Hovered | button::Status::Pressed) => Some(colors.hover),
            _ => None,
        };
        button::Style {
            background: background.map(Background::Color),
            text_color: if active { colors.text } else { colors.muted },
            border: Border {
                radius: 8.0.into(),
                width: if active { 1.0 } else { 0.0 },
                color: colors.border,
            },
            ..button::Style::default()
        }
    }
}

/// A clickable attribute chip; `active` when it is a filter in use.
pub fn chip(color: Color, colors: Colors, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let background = match (active, hovered) {
            (true, _) => color,
            (false, true) => colors.tint(color, 0.22),
            (false, false) => colors.tint(color, 0.12),
        };
        button::Style {
            background: filled(background),
            text_color: if active { colors.surface } else { color },
            border: border::rounded(20),
            ..button::Style::default()
        }
    }
}

// Scrolling

pub fn thin_scrollbar() -> scrollable::Direction {
    scrollable::Direction::Vertical(scrollable::Scrollbar::new().width(6).scroller_width(6).margin(2))
}

/// A slim scroller with no visible rail.
pub fn slim_scroller(colors: Colors) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |theme, status| {
        let mut style = scrollable::default(theme, status);
        let hovered = !matches!(status, scrollable::Status::Active { .. });
        let rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(if hovered { colors.muted } else { colors.border }),
                border: border::rounded(3),
            },
        };
        style.vertical_rail = rail;
        style.horizontal_rail = rail;
        style
    }
}

// Inputs

/// Round checkbox, filled with the primary colour when done.
pub fn round_checkbox(colors: Colors) -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style {
    move |_, status| {
        let (checked, hovered) = match status {
            checkbox::Status::Active { is_checked } | checkbox::Status::Disabled { is_checked } => (is_checked, false),
            checkbox::Status::Hovered { is_checked } => (is_checked, true),
        };
        checkbox::Style {
            background: Background::Color(if checked {
                colors.primary
            } else if hovered {
                colors.hover
            } else {
                colors.surface
            }),
            icon_color: colors.surface,
            border: Border {
                radius: 20.0.into(),
                width: 1.5,
                color: if checked || hovered {
                    colors.primary
                } else {
                    colors.muted
                },
            },
            text_color: None,
        }
    }
}

pub fn input(colors: Colors) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: filled(colors.surface).unwrap(),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: if focused { colors.primary } else { colors.border },
            },
            icon: colors.muted,
            placeholder: colors.muted,
            value: colors.text,
            selection: colors.tint(colors.primary, 0.3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colours() {
        assert_eq!(parse_hex("#ff0000"), Some(Color::from_rgb8(255, 0, 0)));
        assert_eq!(parse_hex("00ff00"), Some(Color::from_rgb8(0, 255, 0)));
        assert_eq!(parse_hex("#fff"), None);
    }

    #[test]
    fn overrides_apply_per_mode() {
        let overrides = ColorOverrides {
            dark: PaletteOverrides {
                primary: Some("#010203".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(Colors::new(true, &overrides).primary, Color::from_rgb8(1, 2, 3));
        assert_eq!(Colors::new(false, &overrides).primary, Colors::LIGHT.primary);
    }
}
