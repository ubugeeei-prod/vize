//! Complete original Component custody after consuming its expression owners.

use alloc::vec::Vec as OwnedVec;
use vize_l0::{SourceBlock, Vec};

use super::super::text::{RetainedTextBinding, TextBoundary};
use super::ComponentParse;
use crate::dialect::LegacyVueVersion;
use crate::parse::SurfaceError;
use crate::surface::SurfaceTree;

mod body;
pub use body::{RetainedTextChild, RetainedTextChildren, RetainedTextRefusal, RetainedTextView};

/// The complete original body and every original callback observation.
/// All syntax/diagnostic owners use ordinary Vec and run their destructors.
/// This is a consuming ownership handoff, not a new body parse or admission.
#[derive(Debug)]
pub struct ComponentExpressionPool<'a> {
    block: SourceBlock<'a>,
    tree: SurfaceTree<'a>,
    authored: Option<SurfaceTree<'a>>,
    errors: Vec<'a, SurfaceError>,
    bindings: OwnedVec<RetainedTextBinding<'a>>,
    unsupported: OwnedVec<TextBoundary>,
}

impl<'a> ComponentExpressionPool<'a> {
    pub const fn version(&self) -> LegacyVueVersion {
        LegacyVueVersion::V2
    }
    pub const fn block(&self) -> SourceBlock<'a> {
        self.block
    }
    pub fn tree(&self) -> &SurfaceTree<'a> {
        &self.tree
    }
    pub fn authored(&self) -> Option<&SurfaceTree<'a>> {
        self.authored.as_ref()
    }
    pub fn errors(&self) -> &[SurfaceError] {
        &self.errors
    }
    pub fn bindings(&self) -> &[RetainedTextBinding<'a>] {
        &self.bindings
    }
    pub fn unsupported(&self) -> &[TextBoundary] {
        &self.unsupported
    }
}

impl<'a> ComponentParse<'a> {
    /// Consume this actual original body, retaining all callbacks and refusals.
    /// Successful roots survive normal pool drop while source and arena live;
    /// original non-Expr artifacts remain owner-borrowed. A refused slot keeps
    /// its complete owner beside all earlier and later slots, without rollback.
    pub fn into_expression_pool(self) -> ComponentExpressionPool<'a> {
        let Self {
            block,
            tree,
            authored,
            errors,
            bindings,
            unsupported,
        } = self;
        ComponentExpressionPool {
            block,
            tree,
            authored,
            errors,
            bindings: bindings
                .into_iter()
                .map(super::super::text::retain_binding)
                .collect(),
            unsupported,
        }
    }
}

#[cfg(test)]
mod tests;
