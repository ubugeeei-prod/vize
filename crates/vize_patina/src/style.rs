//! Style lint document/view types.
//!
//! This abstracts CSS lint input away from SFC-specific style blocks and
//! centralizes traversal over nested CSS rules.

use crate::ir::ByteRange;
use lightningcss::{
    rules::{font_face::FontFaceRule, style::StyleRule, CssRule as LightningCssRule},
    stylesheet::StyleSheet,
};

/// Style syntax used for a lint document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleSyntax {
    /// Standard CSS.
    Css,
    /// Reserved for SCSS.
    Scss,
    /// Reserved for Sass.
    Sass,
    /// Reserved for Less.
    Less,
    /// Reserved for PostCSS-like syntaxes.
    PostCss,
}

/// Style source document.
#[derive(Debug, Clone, Copy)]
pub struct StyleDocument<'a> {
    source: &'a str,
    range: ByteRange,
    syntax: StyleSyntax,
}

impl<'a> StyleDocument<'a> {
    /// Create a style document.
    pub const fn new(source: &'a str, range: ByteRange, syntax: StyleSyntax) -> Self {
        Self {
            source,
            range,
            syntax,
        }
    }

    /// Create a plain CSS document.
    pub fn css(source: &'a str, offset: u32) -> Self {
        Self::new(
            source,
            ByteRange::new(offset, offset + source.len() as u32),
            StyleSyntax::Css,
        )
    }

    /// Raw source.
    pub const fn source(&self) -> &'a str {
        self.source
    }

    /// Source range within the parent document.
    pub const fn range(&self) -> ByteRange {
        self.range
    }

    /// Start offset in parent document.
    pub const fn offset(&self) -> usize {
        self.range.start as usize
    }

    /// Style syntax.
    pub const fn syntax(&self) -> StyleSyntax {
        self.syntax
    }
}

/// Parsed stylesheet view used by CSS rules.
#[derive(Clone, Copy)]
pub struct ParsedStyleSheet<'a, 'i> {
    document: &'a StyleDocument<'i>,
    stylesheet: &'a StyleSheet<'i, 'i>,
}

impl<'a, 'i> ParsedStyleSheet<'a, 'i> {
    /// Create a parsed stylesheet view.
    pub const fn new(document: &'a StyleDocument<'i>, stylesheet: &'a StyleSheet<'i, 'i>) -> Self {
        Self {
            document,
            stylesheet,
        }
    }

    /// Raw source.
    pub const fn source(&self) -> &'i str {
        self.document.source()
    }

    /// Start offset in parent document.
    pub const fn offset(&self) -> usize {
        self.document.offset()
    }

    /// Style syntax.
    pub const fn syntax(&self) -> StyleSyntax {
        self.document.syntax()
    }

    /// Access to the underlying parsed stylesheet.
    pub const fn raw(&self) -> &'a StyleSheet<'i, 'i> {
        self.stylesheet
    }

    /// Visit all qualified style rules, descending through nested blocks.
    pub fn for_each_style_rule(&self, visitor: &mut impl FnMut(&StyleRule<'i>)) {
        walk_rules(&self.stylesheet.rules.0, &mut |rule| {
            if let LightningCssRule::Style(style_rule) = rule {
                visitor(style_rule);
            }
        });
    }

    /// Visit all `@font-face` rules, descending through nested blocks.
    pub fn for_each_font_face_rule(&self, visitor: &mut impl FnMut(&FontFaceRule<'i>)) {
        walk_rules(&self.stylesheet.rules.0, &mut |rule| {
            if let LightningCssRule::FontFace(font_face) = rule {
                visitor(font_face);
            }
        });
    }
}

fn walk_rules<'i>(rules: &[LightningCssRule<'i>], visitor: &mut impl FnMut(&LightningCssRule<'i>)) {
    for rule in rules {
        visitor(rule);

        match rule {
            LightningCssRule::Media(media) => walk_rules(&media.rules.0, visitor),
            LightningCssRule::Supports(supports) => walk_rules(&supports.rules.0, visitor),
            LightningCssRule::LayerBlock(layer) => walk_rules(&layer.rules.0, visitor),
            _ => {}
        }
    }
}
