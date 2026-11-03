//! Spell original sealed SSR setup reads through Vue's actual sixth argument.

use super::{SsrError, SsrErrorKind, encode};
use crate::expr::{AccessError, AccessProvider, AccessSpelling, write_expression};
use crate::runtime::{Runtime, vocabulary};
use crate::write::{LinkSink, Writer};
use core::cell::Cell;
use vize_l0::id::NodeId;
use vize_l2::{op::InterpolationOp, resolution::Occurrence};
use vize_l3::decision::ssr::{NativeSelectedSetupSsrAnalysis, SsrSetupExpression};

/// Only genuine original setup/template SSR analysis selects this entry.
/// A caller cannot substitute DOM facts or choose a context access policy.
/// The whole unsupported view returns no writer or partial SSR function.
/// ```compile_fail
/// use vize_l3::decision::native::NativeSelectedSetupDomAnalysis;
/// use vize_l4::{targets::ssr::emit_selected_setup_template, write::NoLinks};
/// fn substitute(dom: &NativeSelectedSetupDomAnalysis<'_, '_, '_>) {
///     let _ = emit_selected_setup_template::<NoLinks>(dom);
/// }
/// ```
pub fn emit_selected_setup_template<L: LinkSink>(
    analysis: &NativeSelectedSetupSsrAnalysis<'_, '_, '_>,
) -> Result<Writer<L>, SsrError> {
    encode(
        analysis.artifact().source(),
        analysis.ssr(),
        None,
        Some(analysis),
    )
}

pub(super) fn write<L: LinkSink>(
    writer: &mut Writer<L>,
    analysis: &NativeSelectedSetupSsrAnalysis<'_, '_, '_>,
    node: NodeId,
    interpolation: &InterpolationOp<'_>,
) -> Result<(), SsrError> {
    let fail = |kind| SsrError {
        node: Some(node),
        span: interpolation.span,
        kind,
    };
    let row = analysis
        .expression(node)
        .ok_or_else(|| fail(SsrErrorKind::MissingAnalysis))?;
    let resolution = row.resolution();
    if !core::ptr::eq(resolution.file(), analysis.file()) || resolution.node() != node {
        return Err(fail(SsrErrorKind::MissingAnalysis));
    }
    let table = resolution
        .table()
        .ok_or_else(|| fail(SsrErrorKind::MissingAnalysis))?;
    if row.reads().len() != table.occurrences().len() {
        return Err(fail(SsrErrorKind::MissingAnalysis));
    }
    let helper = vocabulary(Runtime::VueServerRenderer)
        .helper("ssrInterpolate")
        .ok_or_else(|| fail(SsrErrorKind::MissingRuntimeHelper))?;
    writer.use_helper(helper);
    writer.push("${_ssrInterpolate(");
    write_expression(
        writer,
        analysis.artifact().source(),
        table,
        &SetupReads {
            row,
            index: Cell::new(0),
        },
    )
    .map_err(|error| SsrError {
        node: Some(node),
        span: error.span,
        kind: SsrErrorKind::Expression(error),
    })?;
    writer.push(")}");
    Ok(())
}

struct SetupReads<'read, 'owner, 'arena> {
    row: &'read SsrSetupExpression<'owner, 'arena>,
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
        Ok(AccessSpelling::Rewrite {
            prefix: "$setup.",
            replacement: Some(read.occurrence().name),
            suffix: "",
            helper: None,
        })
    }
}
