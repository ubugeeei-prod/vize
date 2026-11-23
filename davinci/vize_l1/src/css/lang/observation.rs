//! One original style structure for structural rules and class-name queries.

use super::{AuthoredClasses, StyleLanguage, StyleSheet, css, indented, nested};
use vize_l0::SourceBlock;

/// A style block shared by tools needing both its authored structure and its
/// class names. All spans use the original block's file coordinates.
pub struct StyleObservation<'a> {
    sheet: StyleSheet<'a>,
    classes: Option<AuthoredClasses>,
}

/// The parsed stylesheet and the dialect that constructed it, lent to
/// structural consumers before the class-name analysis allocates its result.
#[derive(Clone, Copy)]
pub struct OriginalStyleStructure<'s, 'a> {
    language: StyleLanguage,
    block: SourceBlock<'a>,
    sheet: &'s StyleSheet<'a>,
}

impl<'s, 'a> OriginalStyleStructure<'s, 'a> {
    /// The exact original block whose grammar produced this tree.
    pub const fn block(self) -> SourceBlock<'a> {
        self.block
    }

    /// The original block's CSS dialect.
    pub const fn language(self) -> StyleLanguage {
        self.language
    }

    /// The shared tree in original file coordinates.
    pub const fn sheet(self) -> &'s StyleSheet<'a> {
        self.sheet
    }
}

impl<'a> StyleObservation<'a> {
    /// Build the dialect's structure once. Its class reader uses that same
    /// structure, including the original dialect's recovery and nesting rules.
    #[must_use]
    pub fn read(language: StyleLanguage, block: SourceBlock<'a>) -> Self {
        Self::read_with_structure(language, block, |_| {})
    }

    /// Visit the structure before collecting class names from that same tree.
    #[must_use]
    pub fn read_with_structure(
        language: StyleLanguage,
        block: SourceBlock<'a>,
        visit: impl FnOnce(OriginalStyleStructure<'_, 'a>),
    ) -> Self {
        Self::read_with_product(language, block, visit).0
    }

    /// Return a product of the same original tree beside the retained class facts.
    pub fn read_with_product<R>(
        language: StyleLanguage,
        block: SourceBlock<'a>,
        visit: impl FnOnce(OriginalStyleStructure<'_, 'a>) -> R,
    ) -> (Self, R) {
        let sheet = language.structure(block.source(), block.start());
        let product = visit(OriginalStyleStructure {
            language,
            block,
            sheet: &sheet,
        });
        let classes = match language {
            StyleLanguage::Css => css::authored_classes(&sheet),
            StyleLanguage::Scss => nested::authored_classes(&sheet, nested::Dialect::Scss),
            StyleLanguage::Less => nested::authored_classes(&sheet, nested::Dialect::Less),
            StyleLanguage::Sass => indented::authored_classes(&sheet, indented::Syntax::Sass),
            StyleLanguage::Stylus => indented::authored_classes(&sheet, indented::Syntax::Stylus),
        };
        (Self { sheet, classes }, product)
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

impl StyleLanguage {
    /// Run a structural consumer without eagerly collecting class names.
    pub fn with_original_structure<'a, R>(
        self,
        block: SourceBlock<'a>,
        visit: impl FnOnce(OriginalStyleStructure<'_, 'a>) -> R,
    ) -> R {
        let sheet = self.structure(block.source(), block.start());
        visit(OriginalStyleStructure {
            language: self,
            block,
            sheet: &sheet,
        })
    }
}
