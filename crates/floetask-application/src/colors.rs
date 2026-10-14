//! User colour overrides (the colour file).

use serde::Deserialize;

/// Colours for one theme. Each value is a `#rrggbb` string; `None` keeps the
/// built-in colour. An unknown name is an error, so a typo is reported
/// instead of ignored.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PaletteOverrides {
    pub background: Option<String>,
    pub text: Option<String>,
    pub primary: Option<String>,
    pub project: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub danger: Option<String>,
    pub navigation: Option<String>,
    pub priority_a: Option<String>,
    pub priority_b: Option<String>,
    pub priority_c: Option<String>,
    pub priority_other: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default)]
pub struct ColorOverrides {
    pub light: PaletteOverrides,
    pub dark: PaletteOverrides,
}
