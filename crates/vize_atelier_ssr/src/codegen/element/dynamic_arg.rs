//! Dynamic directive arguments (`:[key]`, `@[event]`, `v-model:[prop]`,
//! `#[slot]`, `v-dir:[arg]`) as the SSR walker spells them.

use super::props::is_simple_identifier;
use super::{ExpressionNode, RuntimeHelper, SsrCodegenContext, String, cstr};

impl SsrCodegenContext<'_> {
    /// Render a dynamic directive argument while preserving scoped slot locals.
    ///
    /// The transform rewrites a dynamic argument like any other expression,
    /// so a name it left bare is a local, a global or an inline setup binding,
    /// and stays bare. Only a name it never rewrote (identifier prefixing off)
    /// is read from `_ctx`.
    pub(super) fn dynamic_arg_to_string(&mut self, expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Simple(simple)
                if !simple.is_static
                    && !simple.is_ref_transformed
                    && is_simple_identifier(simple.content) =>
            {
                // A `v-for` alias or slot param is a local, not `_ctx.<key>`.
                self.strip_ctx_for_scoped_params(&cstr!("_ctx.{}", simple.content))
            }
            _ => self.expression_to_string(expr),
        }
    }

    /// A dynamic `v-bind` key under `.camel`, camelized at render time.
    pub(crate) fn camelized_dynamic_key(&mut self, key: &str) -> String {
        self.use_core_helper(RuntimeHelper::Camelize);
        cstr!("_camelize(({key}) || \"\")")
    }
}
