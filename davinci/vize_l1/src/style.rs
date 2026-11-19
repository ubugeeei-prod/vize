//! Style languages: one sibling per `<style lang>` over shared readers.
//!
//! Preprocessors themselves stay upstream. These modules only read the
//! authored source: where rules, declarations, comments and nesting are
//! ([`structure`]), and which class names the rules declare ([`classes`],
//! with selectors resolved by `nesting`). Each language module supplies its
//! lexis; [`css`] also owns the one production cssparser parse.

mod classes;
pub mod css;
mod less;
mod nesting;
mod sass;
mod scss;
mod structure;
mod stylus;

pub use classes::AuthoredClasses;
pub use structure::{StyleCode, StyleComment, StyleNode, StyleNodeKind, StyleSheet};

use structure::Lexis;

/// The language of one style block. A block's `lang` value is resolved to
/// one by the block-language registry, `crate::container::lang::style_lang`.
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
            Self::Scss => nesting::braced_classes(&sheet, nesting::Dialect::Scss),
            Self::Less => nesting::braced_classes(&sheet, nesting::Dialect::Less),
            Self::Sass => nesting::indented_classes(&sheet, nesting::Syntax::Sass),
            Self::Stylus => nesting::indented_classes(&sheet, nesting::Syntax::Stylus),
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
            Self::Scss => scss::LEXIS,
            Self::Less => less::LEXIS,
            Self::Sass => sass::LEXIS,
            Self::Stylus => stylus::lexis(source),
        }
    }
}

#[cfg(test)]
mod tests;
