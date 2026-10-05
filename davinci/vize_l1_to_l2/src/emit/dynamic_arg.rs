//! Dynamic directive argument consumption in the existing DOM emit walk.

use vize_l0::String;
use vize_l2::expr::JsExpr;

use super::{EmitCx, prefix};

impl EmitCx<'_> {
    /// `emit_dynamic_directive_arg` under `prefix_identifiers`.
    pub(super) fn prefixed_dynamic_arg(&self, js: &JsExpr<'_>) -> String {
        prefix::prefix_dynamic_arg(&self.scope, js)
    }
}
