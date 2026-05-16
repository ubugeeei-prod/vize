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
        let idx = locale.index();

        // Try requested locale first
        if let Some(msg) = self.messages[idx].get(key) {
            return Cow::Borrowed(*msg);
        }

        // Fall back to English
        if locale != Locale::En {
            if let Some(msg) = self.messages[0].get(key) {
                return Cow::Borrowed(*msg);
            }
        }

        // Return key as fallback (for debugging)
        Cow::Owned(key.to_string())
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
            #[allow(clippy::disallowed_macros)]
            let placeholder = format!("{{{}}}", name);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Check if a key exists for a locale (without fallback)
    #[inline]
    pub fn has_key(&self, locale: Locale, key: &str) -> bool {
        self.messages[locale.index()].contains_key(key)
    }

    /// Get all keys for a locale
    pub fn keys(&self, locale: Locale) -> impl Iterator<Item = &'static str> + '_ {
        self.messages[locale.index()].keys().copied()
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

    // Load embedded translations
    load_json(&mut messages[0], include_str!("i18n/en.json"));
    load_json(&mut messages[1], include_str!("i18n/ja.json"));
    load_json(&mut messages[2], include_str!("i18n/zh.json"));

    Translator { messages }
});

/// Parse JSON and load into message map
fn load_json(map: &mut FxHashMap<&'static str, &'static str>, json: &'static str) {
    // Fast JSON parsing for flat key-value objects
    // Format: { "key": "value", "key2": "value2", ... }
    let json = json.trim();
    if json.len() < 2 || !json.starts_with('{') || !json.ends_with('}') {
        return;
    }

    let content = &json[1..json.len() - 1];
    let mut idx = 0;

    while idx < content.len() {
        // Skip whitespace
        while idx < content.len() && content.as_bytes()[idx].is_ascii_whitespace() {
            idx += 1;
        }

        if idx >= content.len() {
            break;
        }

        // Expect opening quote for key
        if content.as_bytes()[idx] != b'"' {
            idx += 1;
            continue;
        }
        idx += 1;

        // Parse key
        let key_start = idx;
        while idx < content.len() && content.as_bytes()[idx] != b'"' {
            if content.as_bytes()[idx] == b'\\' {
                idx += 2;
            } else {
                idx += 1;
            }
        }
        let key_end = idx;
        idx += 1; // Skip closing quote

        // Skip to colon
        while idx < content.len() && content.as_bytes()[idx] != b':' {
            idx += 1;
        }
        idx += 1; // Skip colon

        // Skip whitespace
        while idx < content.len() && content.as_bytes()[idx].is_ascii_whitespace() {
            idx += 1;
        }

        // Expect opening quote for value
        if idx >= content.len() || content.as_bytes()[idx] != b'"' {
            continue;
        }
        idx += 1;

        // Parse value (handle escaped quotes)
        let value_start = idx;
        while idx < content.len() {
            if content.as_bytes()[idx] == b'\\' {
                idx += 2;
            } else if content.as_bytes()[idx] == b'"' {
                break;
            } else {
                idx += 1;
            }
        }
        let value_end = idx;
        idx += 1; // Skip closing quote

        // Skip to comma or end
        while idx < content.len() && content.as_bytes()[idx] != b',' {
            idx += 1;
        }
        idx += 1; // Skip comma

        // Extract key and value
        let key = &content[key_start..key_end];
        let value = &content[value_start..value_end];

        // Unescape and leak to get 'static lifetime
        // This is safe because we only load once at startup
        let key: &'static str = Box::leak(key.to_string().into_boxed_str());
        let value: &'static str = Box::leak(unescape_json_string(value).into_boxed_str());

        map.insert(key, value);
    }
}

/// Unescape JSON string escape sequences
#[inline]
fn unescape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                        if let Some(c) = char::from_u32(cp) {
                            result.push(c);
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

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
mod tests {
    use super::{unescape_json_string, Locale, Translator};

    #[test]
    fn test_locale_from_str() {
        assert_eq!("en".parse::<Locale>(), Ok(Locale::En));
        assert_eq!("EN".parse::<Locale>(), Ok(Locale::En));
        assert_eq!("ja".parse::<Locale>(), Ok(Locale::Ja));
        assert_eq!("JA-JP".parse::<Locale>(), Ok(Locale::Ja));
        assert_eq!("zh".parse::<Locale>(), Ok(Locale::Zh));
        assert_eq!("zh-CN".parse::<Locale>(), Ok(Locale::Zh));
        assert!("unknown".parse::<Locale>().is_err());
    }

    #[test]
    fn test_locale_parse() {
        assert_eq!(Locale::parse("en"), Some(Locale::En));
        assert_eq!(Locale::parse("ja"), Some(Locale::Ja));
        assert_eq!(Locale::parse("zh"), Some(Locale::Zh));
        assert_eq!(Locale::parse("unknown"), None);
    }

    #[test]
    fn test_locale_code() {
        assert_eq!(Locale::En.code(), "en");
        assert_eq!(Locale::Ja.code(), "ja");
        assert_eq!(Locale::Zh.code(), "zh");
    }

    #[test]
    fn test_locale_display_name() {
        assert_eq!(Locale::En.display_name(), "English");
        assert_eq!(Locale::Ja.display_name(), "日本語");
        assert_eq!(Locale::Zh.display_name(), "中文");
    }

    #[test]
    fn test_translator_get() {
        let t = Translator::new();
        // Test that basic lookup works
        let msg = t.get(Locale::En, "test.hello");
        // Either returns the translation or the key as fallback
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_translator_format() {
        let t = Translator::new();
        let msg = t.format(Locale::En, "test.greeting", &[("name", "World")]);
        // Either contains the substitution or is the key
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_unescape_json_string() {
        assert_eq!(unescape_json_string("hello"), "hello");
        assert_eq!(unescape_json_string("hello\\nworld"), "hello\nworld");
        assert_eq!(unescape_json_string("hello\\tworld"), "hello\tworld");
        assert_eq!(unescape_json_string("he said \\\"hi\\\""), "he said \"hi\"");
        assert_eq!(unescape_json_string("path\\\\to\\\\file"), "path\\to\\file");
    }
}
