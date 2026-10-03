//! Genuine file-owned DOM entry: literals need no framework binding access.

use vize_l0::id::NodeId;
use vize_l2::{expr::ExprRef, file::FileArtifact, resolution::Occurrence};
use vize_l3::decision::{NativeFileAnalysis, dom::DomFacts, native::NativeTemplateDomAnalysis};

use super::expression::ExpressionWriter;
use super::{DomError, DomErrorKind, encode};
use crate::expr::{
    AccessError, AccessProvider, AccessSpelling, EmitError, EmitErrorKind, write_expression,
};
use crate::write::{LinkSink, Writer};

/// Emit a render declaration from the sole checked file owner.
///
/// Static structure and retained literals are supported. Declarations and
/// lexical visibility do not authorize Vue access, so every expression with
/// references is explicitly refused until its genuine runtime provider exists.
/// If/For and whole-SFC producer completeness remain separate prerequisites.
/// No partial writer is returned on any refusal. The immutable analysis keeps
/// the source, semantic diagnostics, factory scopes and original ASTs alive.
///
/// A caller cannot substitute a bare analysis or supply a context policy:
/// ```compile_fail
/// use vize_l3::decision::NativeAnalysis;
/// use vize_l4::{targets::dom::emit_file, write::NoLinks};
/// fn substitute(analysis: &NativeAnalysis<'_, '_>) { let _ = emit_file::<NoLinks>(analysis); }
/// ```
pub fn emit_file<L: LinkSink>(
    analysis: &NativeFileAnalysis<'_, '_>,
) -> Result<Writer<L>, DomError> {
    encode(
        analysis.artifact().source(),
        analysis.policy(),
        analysis.tables(),
        analysis.dom(),
        FileExpressions {
            file: analysis.file(),
            facts: analysis.dom(),
        },
    )
}

/// Emit the original selected template through its moved completion view.
///
/// The same owner supplies File, artifact and existing sole-walk DOM facts.
/// No neutral analysis is extracted or rebuilt. Ordinary static structure is
/// supported; File expressions retain the literal-only checks below. This
/// supplies no Vue read policy, script emission or whole-SFC completion.
///
/// A neutral File analysis cannot stand in for selected-root completion:
/// ```compile_fail
/// use vize_l3::decision::NativeFileAnalysis;
/// use vize_l4::{targets::dom::emit_template, write::NoLinks};
/// fn substitute(analysis: &NativeFileAnalysis<'_, '_>) {
///     let _ = emit_template::<NoLinks>(analysis);
/// }
/// ```
pub fn emit_template<L: LinkSink>(
    analysis: &NativeTemplateDomAnalysis<'_, '_>,
) -> Result<Writer<L>, DomError> {
    encode(
        analysis.artifact().source(),
        analysis.policy(),
        analysis.tables(),
        analysis.dom(),
        FileExpressions {
            file: analysis.file(),
            facts: analysis.dom(),
        },
    )
}

// Constructed only by the two typed owner entries above. Callers cannot pair
// an arbitrary File with DOM rows or discard the selected completion wrapper.
struct FileExpressions<'s, 'owner, 'arena> {
    file: &'owner FileArtifact<'arena>,
    facts: Option<&'s DomFacts<'owner, 'arena>>,
}

impl ExpressionWriter for FileExpressions<'_, '_, '_> {
    fn write<L: LinkSink>(
        &self,
        writer: &mut Writer<L>,
        node: NodeId,
        expression: ExprRef<'_>,
    ) -> Result<(), DomError> {
        let fail = |kind| DomError {
            node: Some(node),
            span: expression.span(),
            kind,
        };
        let ExprRef::Js(expression) = expression else {
            return Err(fail(DomErrorKind::UnsupportedExpression));
        };
        let row = self
            .facts
            .and_then(|facts| facts.file_expression(node))
            .ok_or_else(|| fail(DomErrorKind::MissingFileExpression))?;
        let resolution = row.resolution();
        if !core::ptr::eq(resolution.file(), self.file) || resolution.node() != node {
            return Err(fail(DomErrorKind::FileOwnerMismatch));
        }
        if !resolution.scope().is_some_and(|scope| {
            self.file
                .scopes()
                .get(scope.index() as usize)
                .is_some_and(|record| record.id == scope)
        }) {
            return Err(fail(DomErrorKind::MissingFileScope));
        }
        let table = resolution
            .table()
            .ok_or_else(|| fail(DomErrorKind::MissingFileExpression))?;
        let retained = table.expression();
        let coordinates = match (retained.coordinates, expression.coordinates) {
            (None, None) => true,
            (Some(left), Some(right)) => core::ptr::eq(left, right),
            _ => false,
        };
        if !core::ptr::eq(retained.ast, expression.ast)
            || !core::ptr::eq(retained.source, expression.source)
            || retained.span != expression.span
            || !coordinates
        {
            return Err(fail(DomErrorKind::Expression(EmitError {
                span: expression.span,
                kind: EmitErrorKind::SourceMismatch,
            })));
        }
        if !expression.ast.is_literal() || !table.occurrences().is_empty() {
            return Err(fail(DomErrorKind::RuntimeAccessUnavailable));
        }
        // This policy is private and rejects every access. The real retained
        // literal and complete empty table were checked above, so it supplies
        // no context/ref/prop fallback and is never consulted for a binding.
        write_expression(writer, self.file.artifact().source(), table, &NoReferences).map_err(
            |error| DomError {
                node: Some(node),
                span: error.span,
                kind: DomErrorKind::Expression(error),
            },
        )
    }
}

struct NoReferences;

impl AccessProvider for NoReferences {
    fn spelling(&self, _: &Occurrence<'_>) -> Result<AccessSpelling<'_>, AccessError> {
        Err(AccessError::MissingBinding)
    }
}

#[cfg(test)]
mod tests;
