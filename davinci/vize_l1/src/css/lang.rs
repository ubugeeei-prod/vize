//! Style languages: one profile per `<style lang>`, one dialect module each.
//!
//! Preprocessors themselves stay upstream. These dialects only read the
//! authored source: where rules, comments and nesting are, and which class
//! names the rules declare.

use alloc::collections::BTreeSet;
use vize_l0::String;

mod css;
mod indented;
mod nested;

/// The language of one style block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleLanguage {
    /// Absent `lang`, `css` or `postcss`.
    Css,
    /// `scss`.
    Scss,
    /// `sass`, the indented syntax.
    Sass,
    /// `less`.
    Less,
    /// `stylus` or `styl`.
    Stylus,
}

impl StyleLanguage {
    /// The language a block's `lang` value names, ASCII case-insensitively.
    /// Any other value has no dialect and stays opaque.
    #[must_use]
    pub fn from_lang(lang: Option<&str>) -> Option<Self> {
        let Some(lang) = lang else {
            return Some(Self::Css);
        };
        [
            ("css", Self::Css),
            ("postcss", Self::Css),
            ("scss", Self::Scss),
            ("sass", Self::Sass),
            ("less", Self::Less),
            ("stylus", Self::Stylus),
            ("styl", Self::Stylus),
        ]
        .into_iter()
        .find_map(|(name, language)| lang.eq_ignore_ascii_case(name).then_some(language))
    }

    /// Local class names the block's rules declare, with nesting resolved.
    ///
    /// Rules whose selector cannot be known statically (interpolation, mixin,
    /// function and control-flow bodies) add nothing; `None` means the source
    /// cannot be read at all (unbalanced, unterminated string or comment).
    /// Imports are not followed.
    #[must_use]
    pub fn authored_local_classes(self, source: &str) -> Option<BTreeSet<String>> {
        match self {
            Self::Css => css::authored_local_classes(source),
            Self::Scss => nested::authored_local_classes(source, nested::Dialect::Scss),
            Self::Less => nested::authored_local_classes(source, nested::Dialect::Less),
            Self::Sass => indented::authored_local_classes(source, indented::Syntax::Sass),
            Self::Stylus => indented::authored_local_classes(source, indented::Syntax::Stylus),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StyleLanguage;

    #[test]
    fn lang_values_name_one_dialect_each() {
        for (lang, expected) in [
            (None, Some(StyleLanguage::Css)),
            (Some("css"), Some(StyleLanguage::Css)),
            (Some("postcss"), Some(StyleLanguage::Css)),
            (Some("scss"), Some(StyleLanguage::Scss)),
            (Some("SCSS"), Some(StyleLanguage::Scss)),
            (Some("sass"), Some(StyleLanguage::Sass)),
            (Some("less"), Some(StyleLanguage::Less)),
            (Some("stylus"), Some(StyleLanguage::Stylus)),
            (Some("styl"), Some(StyleLanguage::Stylus)),
            (Some(""), None),
        ] {
            assert_eq!(StyleLanguage::from_lang(lang), expected, "{lang:?}");
        }
    }
}
