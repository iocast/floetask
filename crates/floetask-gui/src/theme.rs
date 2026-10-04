//! Colours and widget styles.
//!
//! floetask builds an iced theme from its own palette, which the user can
//! override per light and dark mode in the colour file.

use iced::widget::{button, container};
use iced::{Background, Border, Color, Theme, border};

use floetask_application::{ColorOverrides, PaletteOverrides};
use floetask_domain::Priority;

/// floetask's colours for one mode, beyond what iced's palette holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colors {
    pub dark: bool,
    pub background: Color,
    pub text: Color,
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub navigation: Color,
    pub muted: Color,
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
        background: rgb(0xffffff),
        text: rgb(0x1f2328),
        primary: rgb(0x2563eb),
        success: rgb(0x16a34a),
        warning: rgb(0xd97706),
        danger: rgb(0xdc2626),
        navigation: rgb(0xeef0f3),
        muted: rgb(0x6b7280),
        priority_a: rgb(0xe5484d),
        priority_b: rgb(0xf59e0b),
        priority_c: rgb(0x30a46c),
        priority_other: rgb(0x8b8d98),
    };

    const DARK: Self = Self {
        dark: true,
        background: rgb(0x1b1d22),
        text: rgb(0xe6e6e6),
        primary: rgb(0x60a5fa),
        success: rgb(0x4ade80),
        warning: rgb(0xfbbf24),
        danger: rgb(0xf87171),
        navigation: rgb(0x15171b),
        muted: rgb(0x9ca3af),
        priority_a: rgb(0xf06a6e),
        priority_b: rgb(0xf5b84a),
        priority_c: rgb(0x4cc38a),
        priority_other: rgb(0x8b8d98),
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
        apply(&mut self.navigation, &custom.navigation);
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
                background: self.background,
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

    /// A slightly raised surface (rows, chips) on the background.
    pub fn surface(&self, strength: f32) -> Color {
        mix(self.background, self.text, strength)
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

// Styles. Each takes the colours explicitly so views stay declarative.

pub fn navigation(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(colors.navigation)),
        ..container::Style::default()
    }
}

pub fn row(colors: Colors, selected: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: selected.then(|| Background::Color(colors.surface(0.08))),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// A clickable attribute chip; `active` when it is a filter in use.
pub fn chip(color: Color, colors: Colors, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let background = if active {
            color
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => colors.surface(0.16),
                _ => colors.surface(0.08),
            }
        };
        button::Style {
            background: Some(Background::Color(background)),
            text_color: if active { colors.background } else { color },
            border: border::rounded(10),
            ..button::Style::default()
        }
    }
}

pub fn badge(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        text_color: Some(Color::WHITE),
        border: border::rounded(4),
        ..container::Style::default()
    }
}

pub fn card(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(colors.background)),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: colors.surface(0.15),
        },
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(0.3),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    }
}

pub fn backdrop(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::BLACK.scale_alpha(0.45))),
        ..container::Style::default()
    }
}

pub fn toast(colors: Colors, error: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if error {
            colors.danger
        } else {
            colors.surface(0.85)
        })),
        text_color: Some(colors.background),
        border: border::rounded(8),
        ..container::Style::default()
    }
}

pub fn group_header(colors: Colors) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        text_color: Some(colors.muted),
        ..container::Style::default()
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
