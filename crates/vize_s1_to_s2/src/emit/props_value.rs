//! `ui.bind` value admission for emitter-only JS edge cases.

use vize_s0::{Allocator, Span};
use vize_s2::expr::{ExprRef, JsExpr, OpaqueReason};
use vize_s2::op::BindOp;

use super::js::is_valid_js_identifier;
use super::{EmitCx, EmitError};
use super::{UnsupportedReason as Reason, props_bind};

pub(super) enum BindValue<'a> {
    Js(&'a JsExpr<'a>),
    RawJs(&'a str),
}

impl BindValue<'_> {
    pub(super) fn emit(&self, cx: &mut EmitCx<'_>) {
        match self {
            Self::Js(js) => cx.buf.push(js.source),
            Self::RawJs(source) => cx.buf.push(source),
        }
    }

    pub(super) const fn js(&self) -> Option<&JsExpr<'_>> {
        match self {
            Self::Js(js) => Some(js),
            Self::RawJs(_) => None,
        }
    }
}

pub(super) fn bind_value<'a>(bind: &'a BindOp<'a>) -> Result<BindValue<'a>, EmitError> {
    match bind.value {
        Some(ExprRef::Js(js)) => Ok(BindValue::Js(js)),
        Some(ExprRef::Opaque(opaque)) if opaque.reason == OpaqueReason::ParseRejected => {
            if is_valid_js_identifier(opaque.source) {
                return Ok(BindValue::RawJs(opaque.source));
            }
            if trailing_block_comment_value_is_js(opaque.source, opaque.span) {
                return Ok(BindValue::RawJs(opaque.source));
            }
            Err(EmitError::unsupported_at(
                Reason::BindValueNotJs,
                opaque.span,
            ))
        }
        Some(_) | None => props_bind::js_value(bind).map(BindValue::Js),
    }
}

fn trailing_block_comment_value_is_js(source: &str, span: Span) -> bool {
    let Some(stripped) = strip_trailing_block_comments(source) else {
        return false;
    };
    let allocator = Allocator::new();
    JsExpr::parse_in(
        &allocator,
        stripped,
        Span::new(span.start, span.start + stripped.len() as u32),
    )
    .is_ok()
}

fn strip_trailing_block_comments(mut source: &str) -> Option<&str> {
    let original = source;
    loop {
        source = source.trim_end();
        if let Some(end) = source.strip_suffix("*/")
            && let Some(start) = end.rfind("/*")
        {
            source = &end[..start];
            continue;
        }
        break;
    }
    let stripped = source.trim_end();
    (stripped.len() < original.len() && !stripped.trim().is_empty()).then_some(stripped)
}
