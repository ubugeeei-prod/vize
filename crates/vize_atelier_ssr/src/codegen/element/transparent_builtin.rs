//! Built-ins the server renders as their children.

use vize_atelier_core::{ElementNode, ExpressionNode, PropNode, TemplateChildNode};

use crate::codegen::SsrCodegenContext;
use crate::codegen::css_vars::RootCssVars;
use crate::codegen::helpers::single_fallthrough_child_index;

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
        if matches!(el.tag, "Transition" | "transition") {
            self.process_transition(el, inherit_attrs);
            return;
        }
        self.process_children_with_fallthrough_attrs(
            &el.children,
            false,
            false,
            false,
            inherit_attrs,
        );
    }

    /// Vue's `ssrProcessTransition`: the client's `BaseTransition` renders its
    /// first non-comment child, so the server drops direct comments
    /// (vuejs/core#5351) and nested fragments, and an `appear` transition
    /// hydrates from a `<template>`.
    fn process_transition(&mut self, el: &ElementNode<'a>, inherit_attrs: bool) {
        let appear = has_static_or_bound_prop(el, "appear");
        if appear {
            self.push_string_part_static("<template>");
        }
        let fallthrough = if inherit_attrs {
            single_fallthrough_child_index(&el.children)
        } else {
            None
        };
        for (index, child) in vize_atelier_core::walk_probe::ssr_children(&el.children).enumerate()
        {
            if matches!(child, TemplateChildNode::Comment(_)) {
                continue;
            }
            self.process_child(
                child,
                true,
                false,
                fallthrough == Some(index),
                RootCssVars::default(),
            );
        }
        if appear {
            self.push_string_part_static("</template>");
        }
    }
}

/// Vue's `findProp(node, name, false, true)`: a static attribute or a
/// static-name `v-bind`, whatever its value.
fn has_static_or_bound_prop(el: &ElementNode<'_>, name: &str) -> bool {
    el.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == name,
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name)
        }
    })
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
