//! Expression-producing template operations and their original lexical resolution.
use super::{
    ExpressionReferenceRef, Model, TemplateQueryError, check_template_offset, visit_resolution,
};
use crate::{file::build::ScopedResolution, query::ScopeRef};
use vize_l0::{Span, id::NodeId};

/// Operation that attached an expression resolution to the canonical File.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateOperandKind {
    Interpolation,
    Bind,
    Condition,
    Model,
}

/// One expression, including constants with no references. The scope and all
/// occurrences borrow the same File. Handler and loop-head resolutions belong
/// to their separate template query families.
#[derive(Clone, Copy)]
pub struct ResolvedTemplateOperand<'f, 'a> {
    node: NodeId,
    scope: ScopeRef<'f, 'a>,
    row: &'f ScopedResolution<'a>,
}

impl<'f, 'a> ResolvedTemplateOperand<'f, 'a> {
    #[must_use]
    pub fn node(self) -> NodeId {
        self.node
    }

    #[must_use]
    pub fn kind(self) -> TemplateOperandKind {
        self.row.kind
    }

    /// File-absolute UTF-8 extent of the original expression spelling.
    #[must_use]
    pub fn span(self) -> Span {
        self.row.table.expression().span
    }

    /// Decoded expression text as passed to the original language parser.
    #[must_use]
    pub fn text(self) -> &'a str {
        self.row.table.expression().source
    }

    #[must_use]
    pub fn scope(self) -> ScopeRef<'f, 'a> {
        self.scope
    }

    /// Project just this operand's original occurrences into authored coordinates.
    pub fn for_each_reference(
        self,
        mut visit: impl FnMut(ExpressionReferenceRef<'f, 'a>),
    ) -> Result<(), TemplateQueryError> {
        visit_resolution(self.scope.file(), self.node, self.row, |_, reference| {
            visit(reference);
            Ok(())
        })
    }
}

impl<'f, 'a> Model<'f, 'a> {
    /// Visit each resolved expression once in unspecified order. The operation
    /// that minted its node recorded the role alongside the resolution. A late
    /// refusal invalidates the whole visit.
    pub fn for_each_resolved_template_operand(
        self,
        mut visit: impl FnMut(ResolvedTemplateOperand<'f, 'a>),
    ) -> Result<(), TemplateQueryError> {
        check_template_offset(self.file, 0)?;
        for (node, row) in self.file.facts.expressions.iter() {
            let scope = ScopeRef::from_id(self.file, row.scope)
                .ok_or(TemplateQueryError::OriginalMembership)?;
            visit(ResolvedTemplateOperand { node, scope, row });
        }
        Ok(())
    }
}
