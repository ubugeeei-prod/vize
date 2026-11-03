//! Rewrite only the sealed original ordinary empty-default statement prefix.

use super::setup::COMPONENT_BINDING;
use crate::write::{LinkSink, Writer};
use vize_l0::Span;
use vize_l2::lang::js::VueOrdinaryEmpty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryEmitErrorKind {
    OriginalRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinaryEmitError {
    pub span: Span,
    pub kind: OrdinaryEmitErrorKind,
}

/// Prepare the genuine ordinary script for the existing component assembler.
/// Original leading source and object-through-tail bytes are linked once.
/// The certified export/default prefix, including intervening comments, is
/// replaced with the shared generated component declaration without source links.
/// The original parser observation still retains every authored comment.
/// Range failures return before a writer or partial fragment is created.
///
/// A setup capability cannot substitute for this ordinary certificate:
/// ```compile_fail
/// use vize_l2::lang::js::VueSetup;
/// use vize_l4::{module::ordinary::emit_ordinary_empty, write::NoLinks};
/// fn substitute(setup: &VueSetup<'_, '_, '_, '_>) {
///     let _ = emit_ordinary_empty::<NoLinks>(setup);
/// }
/// ```
pub fn emit_ordinary_empty<L: LinkSink>(
    ordinary: &VueOrdinaryEmpty<'_, '_, '_, '_>,
) -> Result<Writer<L>, OrdinaryEmitError> {
    let fail = |span| OrdinaryEmitError {
        span,
        kind: OrdinaryEmitErrorKind::OriginalRange,
    };
    let source = ordinary.source();
    let statement = ordinary.statement_span();
    let export = ordinary.export_keyword_span();
    let default = ordinary.default_keyword_span();
    let object = ordinary.object_span();
    if !source.contains_block_span(statement)
        || !source.contains_block_span(export)
        || !source.contains_block_span(default)
        || !source.contains_block_span(object)
        || statement.start != export.start
        || export.end.checked_sub(export.start) != Some(6)
        || export.end > default.start
        || default.end.checked_sub(default.start) != Some(7)
        || default.end > object.start
        || object.start >= object.end
        || object.end > statement.end
    {
        return Err(fail(statement));
    }
    let leading = source
        .source()
        .get(..(statement.start - source.start()) as usize)
        .ok_or_else(|| fail(statement))?;
    let tail = source
        .source()
        .get((object.start - source.start()) as usize..)
        .ok_or_else(|| fail(object))?;

    let mut writer = Writer::default();
    if !leading.is_empty() {
        writer.push_linked(leading, Span::new(source.start(), statement.start));
    }
    writer.push("const ");
    writer.push(COMPONENT_BINDING);
    writer.push(" = ");
    writer.push_linked(tail, Span::new(object.start, source.end()));
    Ok(writer)
}
