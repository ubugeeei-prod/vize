use serde::{Deserialize, Serialize};

use super::HyphenationStyle;
use crate::String;

/// The original style string or a style with literal attribute-name exemptions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeHyphenationConfig {
    Style(HyphenationStyle),
    Options(AttributeHyphenationOptions),
}

impl AttributeHyphenationConfig {
    pub fn style(&self) -> HyphenationStyle {
        match self {
            Self::Style(style) => *style,
            Self::Options(options) => options.style,
        }
    }

    pub fn ignore(&self) -> Option<&[String]> {
        match self {
            Self::Style(_) => None,
            Self::Options(options) => Some(&options.ignore),
        }
    }

    pub fn ignore_tags(&self) -> Option<&[String]> {
        match self {
            Self::Style(_) => None,
            Self::Options(options) => Some(&options.ignore_tags),
        }
    }
}

/// Case-sensitive literal substrings appended to the rule's built-in exemptions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RawOptions")]
pub struct AttributeHyphenationOptions {
    pub style: HyphenationStyle,
    pub ignore: Vec<String>,
    /// Exact authored tag names; regular-expression policies are unsupported.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ignore_tags: Vec<String>,
}

impl Default for AttributeHyphenationOptions {
    fn default() -> Self {
        Self {
            style: HyphenationStyle::Always,
            ignore: Vec::new(),
            ignore_tags: Vec::new(),
        }
    }
}

impl AttributeHyphenationOptions {
    pub fn validate(&self) -> Result<(), &'static str> {
        for (index, entry) in self.ignore.iter().enumerate() {
            if entry.chars().all(is_ecmascript_whitespace) {
                return Err("ignore entries must not be blank");
            }
            if entry.ends_with(":exit") {
                return Err("ignore entries must not end with :exit");
            }
            if self
                .ignore
                .iter()
                .take(index)
                .any(|previous| previous == entry)
            {
                return Err("ignore entries must be unique");
            }
        }
        for (index, tag) in self.ignore_tags.iter().enumerate() {
            if is_regex_tag_policy(tag) {
                return Err(
                    "ignoreTags supports literal tag names only; regular-expression policies are unsupported",
                );
            }
            if self
                .ignore_tags
                .iter()
                .take(index)
                .any(|previous| previous == tag)
            {
                return Err("ignoreTags entries must be unique");
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct RawOptions {
    style: HyphenationStyle,
    ignore: Vec<String>,
    ignore_tags: Vec<String>,
}

impl Default for RawOptions {
    fn default() -> Self {
        Self {
            style: HyphenationStyle::Always,
            ignore: Vec::new(),
            ignore_tags: Vec::new(),
        }
    }
}

impl TryFrom<RawOptions> for AttributeHyphenationOptions {
    type Error = &'static str;

    fn try_from(raw: RawOptions) -> Result<Self, Self::Error> {
        let options = Self {
            style: raw.style,
            ignore: raw.ignore,
            ignore_tags: raw.ignore_tags,
        };
        options.validate()?;
        Ok(options)
    }
}

// Recognize the official /^\/(.+)\/(.*)$/u marker without evaluating a regex.
// Dot excludes these four ECMAScript line terminators; all other plain strings,
// including empty/blank/:exit values, remain valid nonmatching literal names.
fn is_regex_tag_policy(value: &str) -> bool {
    value.starts_with('/')
        && !value
            .chars()
            .any(|ch| matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
        && value.rfind('/').is_some_and(|index| index > 1)
}

// Match the official JSON-schema /^\s*$/ boundary: Rust whitespace also
// includes NEL and excludes BOM, so trim()/is_whitespace() would differ.
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
