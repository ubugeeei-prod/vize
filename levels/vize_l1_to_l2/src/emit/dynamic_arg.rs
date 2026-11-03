//! Dynamic directive argument consumption in the existing DOM emit walk.

use vize_l0::{String, cstr};
use vize_l2::expr::{ExprRef, JsExpr};

use super::{EmitCx, EmitError, prefix};

impl EmitCx<'_> {
    /// `emit_dynamic_directive_arg` under `prefix_identifiers`.
    pub(super) fn prefixed_dynamic_arg(&self, js: &JsExpr<'_>) -> Result<String, EmitError> {
        if !self.scope.is_script_setup() {
            return Ok(prefix::prefix_dynamic_arg(&self.scope, js));
        }
        if let Some(local) = js.source.strip_prefix("_ctx.")
            && self.scope.is_slot_param(local)
        {
            return Ok(String::from(local));
        }
        if self.scope.is_slot_param(js.source) {
            return Ok(String::from(js.source));
        }
        let text = self.prefixed_trimmed_expr(&ExprRef::Js(js), prefix::Site::Expression)?;
        Ok(if js.source.starts_with('`') {
            cstr!("({})", text.as_str())
        } else {
            text
        })
    }
}
