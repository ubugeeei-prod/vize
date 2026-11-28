//! Original syntax capability of a source-factory JSX root.

use super::{L2Markup, L2Origin};
use vize_atelier_jsx::{SyntaxRoot, l2::JsxL2Root};

impl<'a> L2Markup<'a> {
    /// View one JSX/TSX render root's P2-16 L2 projection.
    pub fn from_projected_root(root: &'a JsxL2Root<'a>) -> Self {
        Self {
            root: &root.root,
            source: root.source,
            op_count: root.op_count,
            origin: L2Origin::ProjectedJsx,
            texts: None,
            text_index: texts::EMPTY,
            allocator: None,
            surface: None,
            authored: None,
            surface_index: None,
            authored_index: None,
            jsx_syntax: None,
        }
    }

    /// View a source-factory root with its immutable original JSX syntax.
    /// A refused root has no partially qualified L2 view.
    pub fn from_syntax_root(root: SyntaxRoot<'a, 'a>) -> Option<Self> {
        if !root.has_syntax() {
            return None;
        }
        let mut markup = Self::from_projected_root(root.lowered().l2.as_ref().ok()?);
        markup.jsx_syntax = Some(root);
        Some(markup)
    }

    pub(in crate::markup) fn has_original_element_syntax(&self, source: &str) -> bool {
        if !core::ptr::eq(self.source, source) {
            return false;
        }
        match self.origin {
            L2Origin::Template => false,
            L2Origin::ProjectedJsx => self.jsx_syntax.is_some(),
            L2Origin::Embedded => false,
            L2Origin::Tsrx => true,
        }
    }

    /// Only the original source/prepared-module driver grants TSRX syntax
    /// custody. A caller-created generic artifact keeps no such capability.
    pub(crate) fn from_tsrx_artifact(artifact: &'a vize_l2::artifact::L2Template<'a>) -> Self {
        let mut markup = Self::from_artifact(artifact);
        markup.origin = L2Origin::Tsrx;
        markup
    }
}
