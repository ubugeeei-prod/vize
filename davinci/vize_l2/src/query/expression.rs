//! Borrowed queries over the checked expression tables of one File.
use super::{Model, PositionQueryError, TemplateQueryError, template::check_template_offset};
use crate::{
    file::{BindingRef, FileArtifact, ScopeId},
    resolution::Occurrence,
};
use vize_l0::{Span, id::NodeId};

/// A use in a resolved expression, its captured lexical scope and the original
/// file-absolute spelling. The occurrence retains its decoded expression-relative range.
#[derive(Clone, Copy)]
pub struct ExpressionReferenceRef<'f, 'a> {
    binding: BindingRef<'f, 'a>,
    scope: ScopeId,
    node: NodeId,
    span: Span,
    occurrence: &'f Occurrence<'a>,
}

impl<'f, 'a> ExpressionReferenceRef<'f, 'a> {
    #[must_use]
    pub fn binding(self) -> BindingRef<'f, 'a> {
        self.binding
    }
    #[must_use]
    pub fn scope(self) -> ScopeId {
        self.scope
    }
    #[must_use]
    pub fn node(self) -> NodeId {
        self.node
    }
    #[must_use]
    pub fn span(self) -> Span {
        self.span
    }
    #[must_use]
    pub fn occurrence(self) -> &'f Occurrence<'a> {
        self.occurrence
    }
}

impl<'f, 'a> Model<'f, 'a> {
    /// Visit exactly the uses in this File's sealed expression tables, in
    /// unspecified order. A late projection refusal invalidates the whole visit.
    pub fn for_each_expression_reference(
        self,
        mut visit: impl FnMut(ExpressionReferenceRef<'f, 'a>),
    ) -> Result<(), TemplateQueryError> {
        check_template_offset(self.file, 0)?;
        visit_rows(self.file, |_, site| {
            visit(site);
            Ok(())
        })
    }

    /// Visit uses of a binding from this same File, including template aliases.
    pub fn for_each_expression_reference_to(
        self,
        binding: BindingRef<'_, '_>,
        mut visit: impl FnMut(ExpressionReferenceRef<'f, 'a>),
    ) -> Result<(), TemplateQueryError> {
        if !core::ptr::eq(self.file, binding.file()) {
            return Err(TemplateQueryError::ForeignSymbol);
        }
        self.for_each_expression_reference(|site| {
            if site.binding.id() == binding.id() {
                visit(site);
            }
        })
    }

    /// Find the unique expression use covering a file-absolute UTF-8 boundary.
    pub fn expression_reference_at_offset(
        self,
        offset: u32,
    ) -> Result<Option<ExpressionReferenceRef<'f, 'a>>, TemplateQueryError> {
        check_template_offset(self.file, offset)?;
        scan_at(self.file, offset)
    }
}

fn scan_at<'f, 'a>(
    file: &'f FileArtifact<'a>,
    offset: u32,
) -> Result<Option<ExpressionReferenceRef<'f, 'a>>, TemplateQueryError> {
    let mut found = None;
    let mut ambiguous = false;
    visit_rows(file, |_, site| {
        if site.span.start <= offset && offset < site.span.end {
            ambiguous |= found.replace(site).is_some();
        }
        Ok(())
    })?;
    if ambiguous {
        return Err(TemplateQueryError::Position(
            PositionQueryError::AmbiguousSite,
        ));
    }
    Ok(found)
}

fn visit_rows<'f, 'a>(
    file: &'f FileArtifact<'a>,
    mut visit: impl FnMut(usize, ExpressionReferenceRef<'f, 'a>) -> Result<(), TemplateQueryError>,
) -> Result<(), TemplateQueryError> {
    for (node, resolved) in file.facts.expressions.iter() {
        // The expression factory checked this table against the File's source
        // before attaching it. Project only the original occurrence range.
        let expression = resolved.table.expression();
        for (index, occurrence) in resolved.table.occurrences().iter().enumerate() {
            let span = expression
                .authored_span(occurrence.span)
                .ok_or(TemplateQueryError::Projection)?;
            let binding = file
                .binding(occurrence.binding)
                .ok_or(TemplateQueryError::OriginalMembership)?;
            visit(
                index,
                ExpressionReferenceRef {
                    binding,
                    scope: resolved.scope,
                    node,
                    span,
                    occurrence,
                },
            )?;
        }
    }
    Ok(())
}

#[cfg(feature = "position-index")]
mod index;
#[cfg(feature = "position-index")]
pub use index::ExpressionPositionIndex;

#[cfg(feature = "template-operand-queries")]
mod operand;
#[cfg(feature = "template-operand-queries")]
pub use operand::{ResolvedTemplateOperand, TemplateOperandKind};
