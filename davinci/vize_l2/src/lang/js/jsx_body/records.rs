use super::JsxFile;
use crate::file::{Reference, ScopeId, ScriptUnitId};
use crate::resolution::SyntaxKind;
use core::ops::Range;
use vize_l0::Span;

pub(super) struct Record<'a> {
    pub kind: SyntaxKind<'a>,
    pub span: Span,
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub parent: Option<usize>,
    pub subtree_end: usize,
    pub references: Range<usize>,
}

/// A node query always borrows the actual owning source, File and syntax.
///
/// ```compile_fail
/// use vize_l2::lang::js::{JsxFile, JsxNode};
/// fn forge<'f, 'a>(owner: &'f JsxFile<'a>) { let _ = JsxNode { owner, index: 0 }; }
/// ```
#[derive(Clone, Copy)]
pub struct JsxNode<'f, 'a> {
    owner: &'f JsxFile<'a>,
    index: usize,
}

impl<'f, 'a> JsxNode<'f, 'a> {
    pub(super) fn new(owner: &'f JsxFile<'a>, index: usize) -> Self {
        Self { owner, index }
    }
    fn record(self) -> Option<&'f Record<'a>> {
        self.owner.records.get(self.index)
    }
    #[must_use]
    pub fn owner(self) -> &'f JsxFile<'a> {
        self.owner
    }
    /// Preorder position inside this owner; never a cross-owner admission key.
    #[must_use]
    pub fn index(self) -> usize {
        self.index
    }
    #[must_use]
    pub fn kind(self) -> Option<SyntaxKind<'a>> {
        Some(self.record()?.kind)
    }
    #[must_use]
    pub fn span(self) -> Option<Span> {
        Some(self.record()?.span)
    }
    #[must_use]
    pub fn unit(self) -> Option<ScriptUnitId> {
        Some(self.record()?.unit)
    }
    #[must_use]
    pub fn scope(self) -> Option<ScopeId> {
        Some(self.record()?.scope)
    }
    #[must_use]
    pub fn parent(self) -> Option<Self> {
        self.owner.node(self.record()?.parent?)
    }
    #[must_use]
    pub fn source(self) -> Option<&'a str> {
        let span = self.record()?.span;
        self.owner
            .file
            .artifact()
            .source()
            .get(span.start as usize..span.end as usize)
    }
    #[must_use]
    pub fn references(self) -> Option<&'f [Reference]> {
        self.owner
            .file
            .references()
            .get(self.record()?.references.clone())
    }
    #[must_use]
    pub fn children(self) -> JsxChildren<'f, 'a> {
        JsxChildren {
            owner: self.owner,
            next: self.index + 1,
            end: self.record().map_or(self.index + 1, |row| row.subtree_end),
        }
    }
    #[must_use]
    pub fn same_owner(self, other: JsxNode<'_, '_>) -> bool {
        core::ptr::eq(self.owner, other.owner)
    }
}

/// Preorder subtree endpoints permit direct-child iteration without an AST pass.
pub struct JsxChildren<'f, 'a> {
    owner: &'f JsxFile<'a>,
    next: usize,
    end: usize,
}
impl<'f, 'a> Iterator for JsxChildren<'f, 'a> {
    type Item = JsxNode<'f, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            return None;
        }
        let node = self.owner.node(self.next)?;
        self.next = node.record()?.subtree_end;
        Some(node)
    }
}
