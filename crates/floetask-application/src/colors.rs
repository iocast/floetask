//! User colour overrides (the colour file).

/// Colours for one theme. Each value is a `#rrggbb` string; `None` keeps the
/// built-in colour.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaletteOverrides {
    pub background: Option<String>,
    pub text: Option<String>,
    pub primary: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub danger: Option<String>,
    pub navigation: Option<String>,
    pub priority_a: Option<String>,
    pub priority_b: Option<String>,
    pub priority_c: Option<String>,
    pub priority_other: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColorOverrides {
    pub light: PaletteOverrides,
    pub dark: PaletteOverrides,
}
