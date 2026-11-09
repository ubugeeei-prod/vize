//! Style languages: one profile per `<style lang>`, one dialect module each.
//!
//! Preprocessors themselves stay upstream. These dialects only read the
//! authored source: where rules, comments and nesting are, and which class
//! names the rules declare.

use super::AuthoredClasses;
use super::structure::{Lexis, StyleSheet};

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

    /// Whether `//` starts a line comment: in every preprocessor, not in CSS.
    #[must_use]
    pub const fn has_line_comments(self) -> bool {
        !matches!(self, Self::Css)
    }

    /// Local class names the block's rules declare, with nesting resolved.
    ///
    /// Rules whose selector cannot be known statically (interpolation, mixin,
    /// function and control-flow bodies) add nothing and leave the result
    /// incomplete; `None` means the source cannot be read at all (unbalanced,
    /// unterminated string or comment). Imports are not followed.
    #[must_use]
    pub fn authored_classes(self, source: &str) -> Option<AuthoredClasses> {
        let sheet = self.structure(source, 0);
        match self {
            Self::Css => css::authored_classes(&sheet),
            Self::Scss => nested::authored_classes(&sheet, nested::Dialect::Scss),
            Self::Less => nested::authored_classes(&sheet, nested::Dialect::Less),
            Self::Sass => indented::authored_classes(&sheet, indented::Syntax::Sass),
            Self::Stylus => indented::authored_classes(&sheet, indented::Syntax::Stylus),
        }
    }

    /// The block's rules, declarations and comments. `source` starts at file
    /// offset `base`, so every span is file-absolute.
    #[must_use]
    pub fn structure(self, source: &str, base: u32) -> StyleSheet<'_> {
        StyleSheet::read(source, base, &self.lexis(source))
    }

    fn lexis(self, source: &str) -> Lexis {
        match self {
            Self::Css => css::LEXIS,
            Self::Scss => nested::SCSS,
            Self::Less => nested::LESS,
            Self::Sass => indented::SASS,
            Self::Stylus => indented::stylus(source),
        }
    }
}

#[cfg(test)]
mod tests;
