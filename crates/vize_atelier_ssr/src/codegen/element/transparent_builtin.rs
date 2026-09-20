//! Built-ins the server renders as their children.

use vize_atelier_core::ElementNode;

use crate::codegen::SsrCodegenContext;

impl<'a> SsrCodegenContext<'a> {
    /// `<Transition>` and `<KeepAlive>` are client-side concerns: the server
    /// renders their children in place, and a single child is still the root
    /// that inherits the component's fallthrough attrs. Resolving them as user
    /// components instead renders nothing and warns at runtime.
    pub(super) fn process_transparent_builtin(
        &mut self,
        el: &ElementNode<'a>,
        inherit_attrs: bool,
    ) {
        self.process_children_with_fallthrough_attrs(
            &el.children,
            false,
            false,
            false,
            inherit_attrs,
        );
    }
}

pub(super) fn is_server_transparent_builtin(tag: &str) -> bool {
    matches!(
        tag,
        "Transition"
            | "transition"
            | "BaseTransition"
            | "base-transition"
            | "KeepAlive"
            | "keep-alive"
    )
}
