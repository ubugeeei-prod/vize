//! Whole native JavaScript JSX modules from the retained original File owner.
//!
//! Only genuine completed L3 decisions admit replacement spans. Everything
//! outside those spans, including module declarations and comments, is copied
//! from the original source. TS erasure, slots and normalized JSX text remain
//! explicit refusals. This does not replace a legacy product route.

use vize_l0::Span;
use vize_l2::lang::js::JsxNode;
use vize_l3::jsx::{JsxDecisionKind, NativeJsxAnalysis};

use crate::write::{Emitted, LinkSink, Writer};

mod admission;
mod helpers;
mod write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxEmitErrorKind {
    InvalidDecision,
    SourceWindow,
    Typescript,
    DirectiveOrHashbang,
    MemberTag,
    UnknownIntrinsic,
    ComponentChildren,
    TextNormalization,
    AttributeNormalization,
    DuplicateAttribute,
    RuntimeHelper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsxEmitError {
    pub kind: JsxEmitErrorKind,
    pub span: Span,
}

impl JsxEmitError {
    fn at(kind: JsxEmitErrorKind, node: JsxNode<'_, '_>) -> Self {
        Self {
            kind,
            span: node.span().unwrap_or(Span::new(0, 0)),
        }
    }
}

/// Emit one complete JS module, with the same bytes for either link sink.
///
/// A TSX owner is retained but refused; copying type syntax is not TS erasure.
pub fn emit_js_module<L: LinkSink>(
    analysis: &NativeJsxAnalysis<'_>,
) -> Result<Emitted<L>, JsxEmitError> {
    admission::check(analysis)?;
    let helper = helpers::selected(analysis)?;
    let source = analysis.owner().file().artifact().source();
    let mut writer = Writer::with_capacity(source.len());
    let mut cursor = 0;
    for root in analysis
        .decisions()
        .filter(|decision| decision.kind() == JsxDecisionKind::Root)
    {
        let node = root.node();
        let span = node.span().ok_or_else(|| invalid(node))?;
        if span.start < cursor {
            return Err(invalid(node));
        }
        write::original(&mut writer, source, Span::new(cursor, span.start))?;
        let element = node.children().next().ok_or_else(|| invalid(node))?;
        write::element(&mut writer, analysis, element, &helper)?;
        cursor = span.end;
    }
    let end = u32::try_from(source.len()).map_err(|_| JsxEmitError {
        kind: JsxEmitErrorKind::SourceWindow,
        span: Span::new(0, cursor),
    })?;
    write::original(&mut writer, source, Span::new(cursor, end))?;
    let mut preamble = Writer::default();
    preamble.push("import { ");
    for (index, selected) in core::iter::once(&helper.node)
        .chain(helper.text.iter())
        .enumerate()
    {
        if index > 0 {
            preamble.push(", ");
        }
        preamble.push(selected.export);
        preamble.push(" as ");
        preamble.push(selected.alias.as_str());
    }
    preamble.push(" } from ");
    preamble.push(&serde_json::to_string(helper.node.module).unwrap_or_default());
    preamble.push(";\n");
    Ok(writer.finish_with_preamble(preamble))
}

fn invalid(node: JsxNode<'_, '_>) -> JsxEmitError {
    JsxEmitError::at(JsxEmitErrorKind::InvalidDecision, node)
}

fn kind<'a>(
    analysis: &NativeJsxAnalysis<'a>,
    node: JsxNode<'_, 'a>,
) -> Result<JsxDecisionKind<'a>, JsxEmitError> {
    analysis
        .decision_for(node)
        .map(|decision| decision.kind())
        .ok_or_else(|| invalid(node))
}
