//! Internationalization (i18n) foundation for Vize.
//!
//! High-performance, zero-cost abstractions for multi-language support.
//!
//! ## Design Principles
//!
//! - **Zero-cost where possible**: Static strings, inline functions
//! - **Fast lookups**: FxHashMap with perfect hash fallback
//! - **Minimal allocations**: Only allocate when variable substitution needed
//! - **Great DX**: Simple API, fallback to English, clear error handling
//!
//! ## Supported Locales
//!
//! - `en` - English (default, always available)
//! - `ja` - Japanese
//! - `zh` - Chinese (Simplified)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use vize_carton::i18n::{Locale, Translator};
//!
//! let translator = Translator::new();
//!
//! // Simple lookup (returns &'static str for static messages)
//! let msg = translator.get(Locale::Ja, "error.parse_failed");
//!
//! // With variable substitution (returns String)
//! let msg = translator.format(Locale::Ja, "error.unexpected_token", &[("token", "}")]);
//! ```

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::str::FromStr;

use vize_l0::diag::MessageLookup;

mod catalog;
mod load;
pub use catalog::LocaleMessages;
use load::load_json;

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Locale {
    /// English (default)
    #[default]
    En = 0,
    /// Japanese
    Ja = 1,
    /// Chinese (Simplified)
    Zh = 2,
}

/// Error type for parsing Locale from string
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLocaleError;

impl std::fmt::Display for ParseLocaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid locale string")
    }
}

impl std::error::Error for ParseLocaleError {}

impl FromStr for Locale {
    type Err = ParseLocaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "en" | "en-us" | "en-gb" | "english" => Ok(Self::En),
            "ja" | "ja-jp" | "japanese" => Ok(Self::Ja),
            "zh" | "zh-cn" | "zh-hans" | "chinese" => Ok(Self::Zh),
            _ => Err(ParseLocaleError),
        }
    }
}

impl Locale {
    /// All available locales
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::Ja, Locale::Zh];

    /// Try to parse locale from string (case-insensitive)
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Get locale code (BCP 47 format)
    #[inline]
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ja => "ja",
            Self::Zh => "zh",
        }
    }

    /// Get locale display name (in that language)
    #[inline]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Ja => "日本語",
            Self::Zh => "中文",
        }
    }

    /// Get locale as array index
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Message domain for organizing translations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// Lint rule messages (vize_patina)
    Lint,
    /// Compiler messages (vize_atelier)
    Compiler,
    /// CLI messages (vize)
    Cli,
    /// General/shared messages
    General,
}

impl Domain {
    /// Get domain prefix for message keys
    #[inline]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Lint => "lint",
            Self::Compiler => "compiler",
            Self::Cli => "cli",
            Self::General => "general",
        }
    }
}

/// Translation entry - either static or owned
#[derive(Debug, Clone)]
pub enum Message {
    /// Static string (zero-cost)
    Static(&'static str),
    /// Owned string (for dynamic content)
    Owned(String),
}

impl Message {
    /// Get message as string slice
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Message::Static(s) => s,
            Message::Owned(s) => s,
        }
    }
}

impl From<&'static str> for Message {
    #[inline]
    fn from(s: &'static str) -> Self {
        Message::Static(s)
    }
}

impl From<String> for Message {
    #[inline]
    fn from(s: String) -> Self {
        Message::Owned(s)
    }
}

impl AsRef<str> for Message {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// High-performance translator
///
/// Uses a two-level lookup:
/// 1. Array lookup by locale index (O(1))
/// 2. HashMap lookup by key (O(1) average)
pub struct Translator {
    /// Messages indexed by [locale][key]
    messages: [FxHashMap<&'static str, &'static str>; 3],
}

impl Translator {
    /// Create a new translator with embedded messages
    #[inline]
    pub fn new() -> &'static Self {
        &GLOBAL_TRANSLATOR
    }

    /// Get a message without variable substitution
    ///
    /// Returns the English fallback if the key is not found in the requested locale.
    /// Returns the key itself if not found in any locale.
    #[inline]
    pub fn get(&self, locale: Locale, key: &str) -> Cow<'static, str> {
        self.for_locale(locale).lookup(key)
    }

    /// Get a message with variable substitution
    ///
    /// Variables are specified as `{name}` in the message template.
    #[inline]
    pub fn format(&self, locale: Locale, key: &str, vars: &[(&str, &str)]) -> String {
        let template = self.get(locale, key);

        if vars.is_empty() {
            return template.into_owned();
        }

        let mut result = template.into_owned();
        for (name, value) in vars {
            let placeholder = crate::cstr!("{{{}}}", name);
            result = result.replace(placeholder.as_str(), value);
        }
        result
    }

    /// Check if a key exists for a locale (without fallback)
    #[inline]
    pub fn has_key(&self, locale: Locale, key: &str) -> bool {
        self.locale_messages(locale)
            .is_some_and(|messages| messages.contains_key(key))
    }

    /// Get all keys for a locale
    pub fn keys(&self, locale: Locale) -> impl Iterator<Item = &'static str> + '_ {
        self.locale_messages(locale)
            .into_iter()
            .flat_map(|messages| messages.keys().copied())
    }

    /// The message table of `locale` (one per locale, by its index).
    fn locale_messages(&self, locale: Locale) -> Option<&FxHashMap<&'static str, &'static str>> {
        self.messages.get(locale.index())
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new().clone()
    }
}

impl Clone for Translator {
    fn clone(&self) -> Self {
        Self {
            messages: self.messages.clone(),
        }
    }
}

// Global translator instance (initialized once, lives forever)
static GLOBAL_TRANSLATOR: Lazy<Translator> = Lazy::new(|| {
    let mut messages: [FxHashMap<&'static str, &'static str>; 3] = [
        FxHashMap::default(),
        FxHashMap::default(),
        FxHashMap::default(),
    ];

    // Load embedded translations (JSON + Rust-side supplemental entries)
    load_json(&mut messages[0], include_str!("i18n/en.json"));
    load_json(&mut messages[1], include_str!("i18n/ja.json"));
    load_json(&mut messages[2], include_str!("i18n/zh.json"));
    crate::i18n_supplemental::register(&mut messages);
    Translator { messages }
});

/// Convenience function to get the global translator
#[inline]
pub fn translator() -> &'static Translator {
    &GLOBAL_TRANSLATOR
}

/// Convenience function to translate a message
#[inline]
pub fn t(locale: Locale, key: &str) -> Cow<'static, str> {
    translator().get(locale, key)
}

/// Convenience function to translate with variables
#[inline]
pub fn t_fmt(locale: Locale, key: &str, vars: &[(&str, &str)]) -> String {
    translator().format(locale, key, vars)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod compiler_error_tests;
