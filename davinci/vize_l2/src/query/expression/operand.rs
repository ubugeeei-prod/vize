//! Expression-producing template operations and their original lexical resolution.
use super::{ExpressionReferenceRef, Model, TemplateQueryError, check_template_offset};
use crate::{ir::build::ScopedResolution, query::ScopeRef};
use vize_l0::{Span, id::NodeId};

/// Operation that attached an expression resolution to the canonical File.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateOperandKind {
    Interpolation,
    Bind,
    Condition,
    Model,
    /// A handler that reads a template binding as a method reference.
    EventReference,
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
        let table = &self.row.table;
        let file = self.scope.l2ir();
        for occurrence in table.occurrences() {
            let span = table
                .expression()
                .authored_span(occurrence.span)
                .ok_or(TemplateQueryError::Projection)?;
            let binding = file
                .binding(occurrence.binding)
                .ok_or(TemplateQueryError::OriginalMembership)?;
            visit(ExpressionReferenceRef {
                binding,
                node: self.node,
                span,
                scope: self.scope.scope().id,
                occurrence,
            });
        }
        Ok(())
    }
}

impl<'f, 'a> Model<'f, 'a> {
    /// Number of original expression operands in this completed File.
    pub fn resolved_template_operand_count(self) -> Result<usize, TemplateQueryError> {
        check_template_offset(self.l2ir, 0)?;
        Ok(self.l2ir.facts.expressions.len())
    }

    /// Read the operand minted for this node in this File's own ID space.
    pub fn resolved_template_operand(
        self,
        node: NodeId,
    ) -> Result<Option<ResolvedTemplateOperand<'f, 'a>>, TemplateQueryError> {
        check_template_offset(self.l2ir, 0)?;
        self.l2ir
            .facts
            .expressions
            .get(node)
            .map(|row| self.operand(node, row))
            .transpose()
    }

    fn operand(
        self,
        node: NodeId,
        row: &'f ScopedResolution<'a>,
    ) -> Result<ResolvedTemplateOperand<'f, 'a>, TemplateQueryError> {
        let scope = ScopeRef::from_id(self.l2ir, row.scope)
            .ok_or(TemplateQueryError::OriginalMembership)?;
        Ok(ResolvedTemplateOperand { node, scope, row })
    }

    /// Visit each resolved expression once in unspecified order. The operation
    /// that minted its node recorded the role alongside the resolution. A late
    /// refusal invalidates the whole visit.
    pub fn for_each_resolved_template_operand(
        self,
        mut visit: impl FnMut(ResolvedTemplateOperand<'f, 'a>),
    ) -> Result<(), TemplateQueryError> {
        check_template_offset(self.l2ir, 0)?;
        for (node, row) in self.l2ir.facts.expressions.iter() {
            visit(self.operand(node, row)?);
        }
        Ok(())
    }
}
