//! Static-name `ui.bind` accessors shared by props admission and object emit.

use vize_disegno::expr::{ExprRef, JsExpr};
use vize_disegno::op::{BindOp, DynamicName};

use super::EmitError;

pub(super) fn static_bind_name<'a>(bind: &'a BindOp<'a>) -> Result<&'a str, EmitError> {
    match bind.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => Err(EmitError::Unsupported),
    }
}

pub(super) fn js_value<'a>(bind: &'a BindOp<'a>) -> Result<&'a JsExpr<'a>, EmitError> {
    match bind.value {
        Some(ExprRef::Js(js)) => Ok(js),
        _ => Err(EmitError::Unsupported),
    }
}
