use serde::{Deserialize, Serialize};

use super::HyphenationStyle;
use crate::String;

/// The original style string or a style with literal event-name exemptions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VOnEventHyphenationConfig {
    Style(HyphenationStyle),
    Options(VOnEventHyphenationOptions),
}

impl VOnEventHyphenationConfig {
    pub fn style(&self) -> HyphenationStyle {
        match self {
            Self::Style(style) => *style,
            Self::Options(options) => options.style,
        }
    }

    pub fn options(&self) -> VOnEventHyphenationOptions {
        match self {
            Self::Style(_) => VOnEventHyphenationOptions {
                style: self.style(),
                ..Default::default()
            },
            Self::Options(options) => options.clone(),
        }
    }
}

/// `ignore` entries are case-sensitive substrings, as in eslint-plugin-vue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawOptions")]
pub struct VOnEventHyphenationOptions {
    pub style: HyphenationStyle,
    pub ignore: Vec<String>,
}

impl Default for VOnEventHyphenationOptions {
    fn default() -> Self {
        Self {
            style: HyphenationStyle::Always,
            ignore: Vec::new(),
        }
    }
}

impl VOnEventHyphenationOptions {
    /// Apply the same validation to JSON and typed native callers.
    pub fn validate(&self) -> Result<(), &'static str> {
        for (index, entry) in self.ignore.iter().enumerate() {
            if entry.chars().all(is_ecmascript_whitespace) {
                return Err("ignore entries must not be blank");
            }
            if entry.ends_with(":exit") {
                return Err("ignore entries must not end with :exit");
            }
            if self.ignore.iter().take(index).any(|prior| prior == entry) {
                return Err("ignore entries must be unique");
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawOptions {
    style: HyphenationStyle,
    ignore: Vec<String>,
}

impl Default for RawOptions {
    fn default() -> Self {
        Self {
            style: HyphenationStyle::Always,
            ignore: Vec::new(),
        }
    }
}

impl TryFrom<RawOptions> for VOnEventHyphenationOptions {
    type Error = &'static str;

    fn try_from(raw: RawOptions) -> Result<Self, Self::Error> {
        let options = Self {
            style: raw.style,
            ignore: raw.ignore,
        };
        options.validate()?;
        Ok(options)
    }
}

// Rust's Unicode whitespace includes NEL and excludes BOM. The upstream
// JSON-schema /^\s*$/ uses ECMAScript whitespace, so neither trim() nor
// char::is_whitespace has the required boundary semantics.
fn is_ecmascript_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000d}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    )
}

#[cfg(test)]
mod tests;
