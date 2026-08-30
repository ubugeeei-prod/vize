//! `ui.bind` value admission for emitter-only JS edge cases.

use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::BindOp;

use super::js::RawJs;
use super::props_bind;
use super::{EmitCx, EmitError};

pub(super) enum BindValue<'a> {
    Js(&'a JsExpr<'a>),
    RawJs(RawJs<'a>),
}

impl BindValue<'_> {
    pub(super) fn emit(&self, cx: &mut EmitCx<'_>) {
        match self {
            Self::Js(js) => cx.buf.push(js.source),
            Self::RawJs(source) => cx.buf.push(source.as_str()),
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
        Some(expr) => {
            if let Some(raw) = super::js::parse_rejected_raw_js(&expr, true) {
                Ok(BindValue::RawJs(raw))
            } else {
                props_bind::js_value(bind).map(BindValue::Js)
            }
        }
        None => props_bind::js_value(bind).map(BindValue::Js),
    }
}
