//! Derived element queries shared by every backend.

use super::MarkupElement;
use crate::markup::attribute::MarkupAttribute;
use crate::markup::binding::{MarkupBinding, MarkupBindingKind};
use crate::markup::element::MarkupElementInner;
use crate::markup::l2::L2ElementOp;
use crate::markup::l2::surface::SurfaceDirective;
use crate::markup::node::MarkupNode;
use vize_l0::String;
use vize_relief::ElementNode;

impl<'a> MarkupElement<'a> {
    /// Find the first binding matching a [`MarkupBindingKind`] and argument
    /// name, e.g. a `v-bind:key` / `:key` (`Bind` + `"key"`) or a `@click` /
    /// `onClick` (`On` + `"click"`).
    pub fn binding(&self, kind: MarkupBindingKind, arg: &str) -> Option<MarkupBinding<'a>> {
        let mut found = None;
        self.walk_bindings(&mut |binding| {
            if found.is_none() && binding.kind() == kind && binding.arg_name_eq(arg) {
                found = Some(binding);
            }
        });
        found
    }

    /// Whether this element has a `key` binding (`:key` / `key` / `key={…}`),
    /// the cross-backend version of the `v-for` key check.
    pub fn has_key_binding(&self) -> bool {
        let mut found = false;
        self.walk_bindings(&mut |binding| {
            if binding.is_key() {
                found = true;
            }
        });
        found
    }

    /// Check whether this element matches the given tag name.
    pub fn is_tag(&self, expected: &str) -> bool {
        self.tag().eq_ignore_ascii_case(expected)
    }

    /// Get a static attribute by name.
    pub fn static_attribute(&self, name: &str) -> Option<MarkupAttribute<'a>> {
        let mut matched = None;
        self.walk_attributes(&mut |attr| {
            if matched.is_none() && attr.name_eq(name) && !attr.is_dynamic() {
                matched = Some(attr);
            }
        });
        matched
    }

    /// Check if a named static attribute exists.
    pub fn has_static_attribute(&self, name: &str) -> bool {
        self.static_attribute(name).is_some()
    }

    /// Check if a directive with the given normalized name exists.
    pub fn has_directive(&self, name: &str) -> bool {
        let mut found = false;
        self.walk_directives(&mut |directive| {
            if directive.name_eq(name) {
                found = true;
            }
        });
        found
    }

    /// Check if this element contains a bound attribute for the given arg name.
    pub fn has_bound_attribute(&self, name: &str) -> bool {
        let mut found = false;
        match self.inner {
            MarkupElementInner::JsxElement { .. } => self.walk_attributes(&mut |attr| {
                if attr.name_eq(name) && attr.is_dynamic() {
                    found = true;
                }
            }),
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::Authored { .. }
            | MarkupElementInner::Relief(_)
            | MarkupElementInner::L2 { .. }
            | MarkupElementInner::L2Carrier { .. } => self.walk_directives(&mut |directive| {
                if directive.name_eq("bind") && directive.arg_name_eq(name) {
                    found = true;
                }
            }),
        }
        found
    }

    /// Concatenate direct text child nodes.
    pub fn direct_text_content(&self) -> String {
        let mut text = String::default();
        self.walk_children(&mut |child| {
            if let MarkupNode::Text(text_node) = child {
                text.push_str(text_node.content());
            }
        });
        text
    }

    /// The backing `vize_relief` element, when this element was projected from
    /// a Vue template. Lets a migrating rule fall back to concrete-node helpers
    /// for template-only cases while still sharing one entry point.
    pub fn as_relief(&self) -> Option<&'a ElementNode<'a>> {
        match self.inner {
            MarkupElementInner::Relief(node) => Some(node),
            _ => None,
        }
    }
}

/// Whether an L2 `template` op is a Vue special template (`v-slot` carrier, or
/// an authored `v-if` / `v-else-if` / `v-else` / `v-for` spelling the lowering
/// kept the op for) — Relief's `ElementType::Template` rule.
pub(super) fn l2_template_is_special(
    op: L2ElementOp<'_>,
    surface: Option<&vize_l1::Element<'_>>,
) -> bool {
    match surface {
        Some(element) => element.open.attrs.iter().any(|attr| {
            SurfaceDirective::parse(attr.name.text)
                .is_some_and(|directive| directive.is_structural() || directive.name == "slot")
        }),
        None => op
            .bindings()
            .iter()
            .any(|binding| matches!(binding, vize_l2::op::BindingOp::SlotContent(_))),
    }
}

/// The Vue parser's lint-mode component rule (no DOM `is_native_tag`): a core
/// built-in component or a tag starting with an uppercase letter.
pub(in crate::markup) fn is_lint_component(tag: &str) -> bool {
    matches!(
        tag,
        "Teleport" | "Suspense" | "KeepAlive" | "BaseTransition" | "Transition" | "TransitionGroup"
    ) || tag.chars().next().is_some_and(char::is_uppercase)
}
