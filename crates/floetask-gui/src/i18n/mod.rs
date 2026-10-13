//! User-facing strings in the user's language.
//!
//! Every label goes through [`tr`] or [`trf`]. They look the key up in the
//! current language's Fluent file (`locales/<code>/floetask.ftl`, embedded
//! at compile time) and fall back to English for anything a translation
//! lacks, so a missing translation shows English, never its key.

mod dates;
#[cfg(test)]
mod tests;

pub use dates::{date_range, long_date, month_year, weekday_abbr};

use std::collections::HashMap;
use std::sync::LazyLock;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use fluent_syntax::ast::Entry;

use floetask_application::AppError;
use floetask_domain::{DocumentError, NoteError};

/// A language of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    German,
    French,
    Spanish,
    Italian,
    /// `en-XA`: English lengthened by 40 % and bracketed, to find clipped
    /// or hard-coded text. Offered in debug builds only.
    Pseudo,
}

impl Language {
    /// Every translated language, in the order Settings lists them.
    pub const SHIPPED: [Language; 5] = [
        Language::English,
        Language::German,
        Language::French,
        Language::Spanish,
        Language::Italian,
    ];

    /// The code in `config.toml` and the name of the locale folder.
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::Italian => "it",
            Language::Pseudo => "en-XA",
        }
    }

    /// The language's name in itself, so users find theirs whatever
    /// language the interface is in.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::German => "Deutsch",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::Italian => "Italiano",
            Language::Pseudo => "Pseudo (en-XA)",
        }
    }

    /// The language a `language` setting names, if floetask has it.
    pub fn from_code(code: &str) -> Option<Self> {
        if code == Language::Pseudo.code() {
            return cfg!(debug_assertions).then_some(Language::Pseudo);
        }
        Self::SHIPPED.into_iter().find(|language| language.code() == code)
    }

    /// The language of an OS locale such as `de-CH` or `fr_FR.UTF-8`: the
    /// region is dropped, so `de-CH` falls back to `de`.
    pub fn from_locale(locale: &str) -> Option<Self> {
        let language = locale.split(['-', '_', '.', '@']).next()?.to_ascii_lowercase();
        Self::SHIPPED.into_iter().find(|shipped| shipped.code() == language)
    }

    fn source(self) -> &'static str {
        match self {
            Language::English | Language::Pseudo => include_str!("../../locales/en/floetask.ftl"),
            Language::German => include_str!("../../locales/de/floetask.ftl"),
            Language::French => include_str!("../../locales/fr/floetask.ftl"),
            Language::Spanish => include_str!("../../locales/es/floetask.ftl"),
            Language::Italian => include_str!("../../locales/it/floetask.ftl"),
        }
    }
}

/// The values of the Language setting: `system`, then every language code.
pub fn language_choices() -> Vec<&'static str> {
    let pseudo = cfg!(debug_assertions).then_some(Language::Pseudo);
    std::iter::once("system")
        .chain(Language::SHIPPED.into_iter().chain(pseudo).map(Language::code))
        .collect()
}

/// What a `language` setting means: the language with that code, otherwise
/// (`system`, or a value floetask does not know) the OS language when
/// floetask has it, otherwise English.
pub fn resolve(setting: &str, system_locale: Option<&str>) -> Language {
    Language::from_code(setting)
        .or_else(|| system_locale.and_then(Language::from_locale))
        .unwrap_or(Language::English)
}

// Views render on the main thread but notification texts are written on
// background tasks, so the language is process-wide. Tests run in parallel,
// some in German, and keep it per thread instead.
#[cfg(not(test))]
static CURRENT: std::sync::RwLock<Language> = std::sync::RwLock::new(Language::English);

#[cfg(test)]
thread_local! {
    static CURRENT: std::cell::Cell<Language> = const { std::cell::Cell::new(Language::English) };
}

/// The language strings are currently shown in.
#[cfg(not(test))]
pub fn language() -> Language {
    *CURRENT.read().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Shows every string in `language` from the next frame on.
#[cfg(not(test))]
pub fn set_language(language: Language) {
    *CURRENT.write().unwrap_or_else(std::sync::PoisonError::into_inner) = language;
}

#[cfg(test)]
pub fn language() -> Language {
    CURRENT.get()
}

#[cfg(test)]
pub fn set_language(language: Language) {
    CURRENT.set(language);
}

/// Looks a string up. A key not even English has is returned as it is.
pub fn tr(key: &str) -> &str {
    tr_known(key).unwrap_or(key)
}

/// Looks a string up and fills its named placeholders, as in
/// `trf("calendar_more", &[("count", 3.into())])`. Numbers pick the plural
/// form, so pass counts as numbers, not text.
pub fn trf(key: &str, args: &[(&str, FluentValue<'_>)]) -> String {
    let mut fluent = FluentArgs::with_capacity(args.len());
    for (name, value) in args {
        fluent.set(*name, value.clone());
    }
    let language = language();
    catalog(language)
        .format(key, Some(&fluent))
        .or_else(|| catalog(Language::English).format(key, Some(&fluent)))
        .map(|text| match language {
            Language::Pseudo => pseudo(&text),
            _ => text,
        })
        .unwrap_or_else(|| key.to_owned())
}

/// A string if English has `key`: for names that are translated when
/// floetask knows them and shown as written otherwise, like statuses.
pub fn tr_known(key: &str) -> Option<&'static str> {
    catalog(language())
        .text(key)
        .or_else(|| catalog(Language::English).text(key))
}

/// An error in the user's language. Details from the OS, such as "access
/// denied", stay as the OS wrote them.
pub fn error_text(error: &AppError) -> String {
    match error {
        AppError::Io { path, message } => trf(
            "error_io",
            &[
                ("path", path.display().to_string().into()),
                ("message", message.as_str().into()),
            ],
        ),
        AppError::Document(DocumentError::StaleLine { line }) => trf("error_stale_line", &[("line", line.into())]),
        AppError::Document(DocumentError::MissingLine { line }) => trf("error_missing_line", &[("line", line.into())]),
        AppError::Note(NoteError::Empty) => tr("error_note_empty").to_owned(),
        AppError::Note(NoteError::InvalidCharacter(character)) => {
            trf("error_note_character", &[("character", character.to_string().into())])
        }
        AppError::Note(NoteError::OutsideNotesFolder) => tr("error_note_outside").to_owned(),
        AppError::NoDoneFile(path) => trf("error_no_done_file", &[("file", path.display().to_string().into())]),
        AppError::EmptyTodo => tr("error_empty_todo").to_owned(),
        AppError::Config(details) => trf("error_config", &[("details", details.as_str().into())]),
        AppError::Other(message) => message.clone(),
    }
}

/// One language's strings: the Fluent bundle, for strings with
/// placeholders, and every string formatted once, so [`tr`] can lend them
/// out without formatting on every frame.
struct Catalog {
    bundle: FluentBundle<FluentResource>,
    texts: HashMap<String, String>,
}

impl Catalog {
    fn load(language: Language) -> Self {
        // A syntax error drops only the broken entry; the tests catch it.
        let resource = FluentResource::try_new(language.source().to_owned()).unwrap_or_else(|(resource, _)| resource);
        let ids: Vec<String> = resource
            .entries()
            .filter_map(|entry| match entry {
                Entry::Message(message) => Some(message.id.name.to_owned()),
                _ => None,
            })
            .collect();
        let locale = language.code().parse().expect("language codes are valid identifiers");
        let mut bundle = FluentBundle::new_concurrent(vec![locale]);
        // Fluent wraps placeables in Unicode isolation marks for mixed text
        // directions. iced would draw them, and every shipped language runs
        // left to right.
        bundle.set_use_isolating(false);
        // A duplicate id keeps the first; the tests catch it.
        let _ = bundle.add_resource(resource);
        let mut catalog = Catalog {
            bundle,
            texts: HashMap::new(),
        };
        for id in ids {
            if let Some(text) = catalog.format(&id, None) {
                let text = match language {
                    Language::Pseudo => pseudo(&text),
                    _ => text,
                };
                catalog.texts.insert(id, text);
            }
        }
        catalog
    }

    fn text(&self, key: &str) -> Option<&str> {
        self.texts.get(key).map(String::as_str)
    }

    fn format(&self, key: &str, args: Option<&FluentArgs>) -> Option<String> {
        let pattern = self.bundle.get_message(key)?.value()?;
        let mut errors = Vec::new();
        Some(self.bundle.format_pattern(pattern, args, &mut errors).into_owned())
    }
}

fn catalog(language: Language) -> &'static Catalog {
    static CATALOGS: LazyLock<Vec<(Language, Catalog)>> = LazyLock::new(|| {
        Language::SHIPPED
            .into_iter()
            .chain([Language::Pseudo])
            .map(|language| (language, Catalog::load(language)))
            .collect()
    });
    let (_, catalog) = CATALOGS
        .iter()
        .find(|(loaded, _)| *loaded == language)
        .expect("every language has a catalog");
    catalog
}

/// English made 40 % longer and bracketed: text that is clipped, or not
/// going through [`tr`], stands out.
fn pseudo(text: &str) -> String {
    let padding = (text.chars().count() * 2).div_ceil(5);
    format!("[{text}{}]", "~".repeat(padding))
}
