//! Builtin writers over the original SSR string-plan regions.

use vize_atelier_core::RuntimeHelper;

use super::{Emitter, Flags, Result};

impl Emitter<'_, '_, '_, '_, '_, '_> {
    /// `_ssrRenderSuspense(_push, { default: () => { ... }, _: 1 })`.
    pub(super) fn suspense(&mut self, flags: Flags) -> Result<()> {
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderSuspense);
        self.ctx.push_indent();
        self.ctx.push("_ssrRenderSuspense(_push, {\n");
        self.ctx.indent_level += 1;
        self.ctx.push_indent();
        self.ctx.push("default: () => {\n");
        self.ctx.indent_level += 1;
        self.children(flags)?;
        self.ctx.flush_push();
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("},\n");
        self.ctx.push_indent();
        self.ctx.push("_: 1\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("})\n");
        Ok(())
    }
}
