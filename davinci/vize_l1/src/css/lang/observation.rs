//! One original style structure for structural rules and class-name queries.

use super::{AuthoredClasses, StyleLanguage, StyleSheet, css, indented, nested};
use vize_l0::SourceBlock;

/// A style block shared by tools needing both its authored structure and its
/// class names. All spans use the original block's file coordinates.
pub struct StyleObservation<'a> {
    language: StyleLanguage,
    sheet: StyleSheet<'a>,
    classes: Option<AuthoredClasses>,
}

impl<'a> StyleObservation<'a> {
    /// Build the dialect's structure once. Its class reader uses that same
    /// structure, including the original dialect's recovery and nesting rules.
    #[must_use]
    pub fn read(language: StyleLanguage, block: SourceBlock<'a>) -> Self {
        let sheet = language.structure(block.source(), block.start());
        let classes = match language {
            StyleLanguage::Css => css::authored_classes(&sheet),
            StyleLanguage::Scss => nested::authored_classes(&sheet, nested::Dialect::Scss),
            StyleLanguage::Less => nested::authored_classes(&sheet, nested::Dialect::Less),
            StyleLanguage::Sass => indented::authored_classes(&sheet, indented::Syntax::Sass),
            StyleLanguage::Stylus => indented::authored_classes(&sheet, indented::Syntax::Stylus),
        };
        Self {
            language,
            sheet,
            classes,
        }
    }

    /// The dialect used to parse this original block.
    #[must_use]
    pub const fn language(&self) -> StyleLanguage {
        self.language
    }

    /// Move the computed class set to a consumer after the structural readers finish.
    #[must_use]
    pub fn into_authored_classes(self) -> Option<AuthoredClasses> {
        self.classes
    }

    /// Rules, declarations, comments and parser recovery over the original bytes.
    #[inline]
    #[must_use]
    pub const fn structure(&self) -> &StyleSheet<'a> {
        &self.sheet
    }

    /// The class set and its completeness. A dialect that refuses its input
    /// still leaves its parsed structure available for structural consumers.
    #[inline]
    #[must_use]
    pub const fn authored_classes(&self) -> Option<&AuthoredClasses> {
        self.classes.as_ref()
    }
}

#[cfg(test)]
mod tests;
