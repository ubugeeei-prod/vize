//! Suspense root-child emission, shared with ordinary SSR roots.

use super::{ElementNode, RuntimeHelper, SsrCodegenContext};

impl<'a> SsrCodegenContext<'a> {
    /// Process Vue's built-in <Suspense> component.
    ///
    /// The SSR renderer has a dedicated helper for Suspense. Rendering it through
    /// `ssrRenderComponent(resolveComponent("Suspense"))` makes Vue attempt a
    /// runtime component lookup and leaves Nuxt root components empty.
    pub(super) fn process_suspense(&mut self, el: &ElementNode<'a>) {
        self.flush_push();
        self.use_ssr_helper(RuntimeHelper::SsrRenderSuspense);

        self.push_indent();
        self.push("_ssrRenderSuspense(_push, {\n");
        self.indent_level += 1;
        self.push_indent();
        self.push("default: () => {\n");
        self.indent_level += 1;

        let old_parts = std::mem::take(&mut self.current_template_parts);
        self.process_children(&el.children, false, false, false);
        self.flush_push();
        self.current_template_parts = old_parts;

        self.indent_level -= 1;
        self.push_indent();
        self.push("},\n");
        self.push_indent();
        self.push("_: 1\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("})\n");
    }
}
