//! Encode sealed same-file setup reads without a caller runtime policy.

use core::cell::Cell;

use vize_l0::id::NodeId;
use vize_l2::file::FileArtifact;
use vize_l2::{expr::ExprRef, resolution::Occurrence};
use vize_l3::decision::{
    dom::{
        DomFacts,
        vue::{NativeVueRenderAnalysis, VueReadKind, VueRenderExpression},
    },
    native::NativeSelectedSetupDomAnalysis,
};

use super::expression::ExpressionWriter;
use super::{DomError, DomErrorKind, encode};
use crate::expr::{
    AccessError, AccessProvider, AccessSpelling, EmitError, EmitErrorKind, write_expression,
};
use crate::write::{LinkSink, Writer};

/// Encode the external-function read decisions of one authenticated Vue owner.
///
/// The first spelling slice admits retained literals and normalized direct
/// identifier roots. Comments remain in their checked source windows. Compound
/// roots refuse until the resolver supplies genuine complete spelling facts.
/// Script emission and original native template custody belong to the frontend;
/// declaration membership alone does not certify a whole original SFC.
/// No partial writer is returned. The separate literal-only file entry keeps
/// refusing references, and a caller cannot supply an accessor policy here.
///
/// ```compile_fail
/// use vize_l3::decision::NativeFileAnalysis;
/// use vize_l4::{targets::dom::emit_vue, write::NoLinks};
/// fn substitute(analysis: &NativeFileAnalysis<'_, '_>) {
///     let _ = emit_vue::<NoLinks>(analysis);
/// }
/// ```
pub fn emit_vue<L: LinkSink>(
    analysis: &NativeVueRenderAnalysis<'_, '_, '_, '_, '_>,
) -> Result<Writer<L>, DomError> {
    encode(
        analysis.artifact().source(),
        analysis.policy(),
        analysis.tables(),
        analysis.dom(),
        VueExpressions {
            analysis: VueRows::Ordinary(analysis),
        },
    )
}

/// Consume only genuine original setup/template read decisions.
/// No neutral File analysis or caller accessor policy selects this entry.
/// ```compile_fail
/// use vize_l3::decision::NativeFileAnalysis;
/// use vize_l4::{targets::dom::emit_selected_setup_template, write::NoLinks};
/// fn promote(file: &NativeFileAnalysis<'_, '_>) {
///     let _ = emit_selected_setup_template::<NoLinks>(file);
/// }
/// ```
pub fn emit_selected_setup_template<L: LinkSink>(
    analysis: &NativeSelectedSetupDomAnalysis<'_, '_, '_>,
) -> Result<Writer<L>, DomError> {
    encode(
        analysis.artifact().source(),
        analysis.policy(),
        analysis.tables(),
        analysis.dom(),
        VueExpressions {
            analysis: VueRows::Selected(analysis),
        },
    )
}

// This private union preserves both sealed owners without allocating or
// accepting an external policy. The original expression encoder stays shared.
enum VueRows<'read, 'view, 'owner, 'descriptor, 'program, 'arena> {
    Ordinary(&'read NativeVueRenderAnalysis<'view, 'owner, 'descriptor, 'program, 'arena>),
    Selected(&'read NativeSelectedSetupDomAnalysis<'view, 'owner, 'arena>),
}
impl<'owner, 'arena> VueRows<'_, '_, 'owner, '_, '_, 'arena> {
    fn file(&self) -> &'owner FileArtifact<'arena> {
        match self {
            Self::Ordinary(a) => a.file(),
            Self::Selected(a) => a.file(),
        }
    }
    fn dom(&self) -> Option<&DomFacts<'owner, 'arena>> {
        match self {
            Self::Ordinary(a) => a.dom(),
            Self::Selected(a) => a.dom(),
        }
    }
    fn expression(&self, node: NodeId) -> Option<&VueRenderExpression<'owner, 'arena>> {
        match self {
            Self::Ordinary(a) => a.expression(node),
            Self::Selected(a) => a.expression(node),
        }
    }
}
struct VueExpressions<'read, 'view, 'owner, 'descriptor, 'program, 'arena> {
    analysis: VueRows<'read, 'view, 'owner, 'descriptor, 'program, 'arena>,
}
impl ExpressionWriter for VueExpressions<'_, '_, '_, '_, '_, '_> {
    fn write_handler<L: LinkSink>(
        &self,
        writer: &mut Writer<L>,
        node: NodeId,
        on: &vize_l2::op::OnOp<'_>,
    ) -> Result<(), DomError> {
        super::handler::write(writer, self.analysis.file(), self.analysis.dom(), node, on)
    }
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
            .analysis
            .expression(node)
            .ok_or_else(|| fail(DomErrorKind::MissingFileExpression))?;
        let resolution = row.resolution();
        if !core::ptr::eq(resolution.file(), self.analysis.file()) || resolution.node() != node {
            return Err(fail(DomErrorKind::FileOwnerMismatch));
        }
        if !resolution.scope().is_some_and(|scope| {
            self.analysis
                .file()
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
        if row.reads().len() != table.occurrences().len() {
            return Err(fail(DomErrorKind::FileOwnerMismatch));
        }
        // These are encoding-shape checks, not binding classification. A
        // direct identifier has no hidden member/key descendants to inspect.
        let certified = if expression.ast.is_literal() {
            table.occurrences().is_empty()
        } else if expression.ast.is_identifier_reference() && table.occurrences().len() == 1 {
            table.occurrences().first().is_some_and(|occurrence| {
                expression
                    .ast
                    .get_identifier_reference()
                    .is_some_and(|identifier| {
                        occurrence.name == identifier.name.as_str()
                            && expression.ast_span_to_source(identifier.span)
                                == Some(occurrence.span)
                            && expression
                                .source
                                .get(occurrence.span.start as usize..occurrence.span.end as usize)
                                == Some(occurrence.name)
                    })
            })
        } else {
            false
        };
        if !certified {
            return Err(fail(DomErrorKind::UncertifiedExpressionSpelling));
        }
        write_expression(
            writer,
            self.analysis.file().artifact().source(),
            table,
            &SetupReads {
                row,
                index: Cell::new(0),
            },
        )
        .map_err(|error| DomError {
            node: Some(node),
            span: error.span,
            kind: DomErrorKind::Expression(error),
        })
    }
}

struct SetupReads<'read, 'owner, 'arena> {
    row: &'read VueRenderExpression<'owner, 'arena>,
    index: Cell<usize>,
}

impl AccessProvider for SetupReads<'_, '_, '_> {
    fn spelling(&self, occurrence: &Occurrence<'_>) -> Result<AccessSpelling<'_>, AccessError> {
        let index = self.index.get();
        let read = self
            .row
            .reads()
            .get(index)
            .ok_or(AccessError::MissingBinding)?;
        let binding = self
            .row
            .resolution()
            .binding(occurrence.binding)
            .ok_or(AccessError::MissingBinding)?;
        if !core::ptr::eq(read.occurrence(), occurrence)
            || read.binding().id() != occurrence.binding
            || !read.binding().same_owner(binding)
        {
            return Err(AccessError::MissingBinding);
        }
        self.index.set(index + 1);
        match read.kind() {
            VueReadKind::SetupLet | VueReadKind::SetupConst => Ok(AccessSpelling::Rewrite {
                prefix: "$setup.",
                replacement: Some(read.occurrence().name),
                suffix: "",
                helper: None,
            }),
        }
    }
}
