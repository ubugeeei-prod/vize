//! Element, component, and slot processing for SSR code generation.
//!
//! The entry module owns dispatch plus shared data structures. Specialized
//! submodules keep SSR element generation small enough to audit independently.

mod component;
mod component_props;
mod css_root;
mod directive_props;
mod merged;
mod plain;
pub(crate) mod props;
mod slot;
mod slot_fn;
pub(crate) mod spanned_props;
mod transition_group;
mod transparent_builtin;
mod vnode;

use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, ForNode, IfNode, PropNode,
    RuntimeHelper, TemplateChildNode,
};
use vize_l0::{FxHashSet, String, ToCompactString};

use super::{
    SsrCodegenContext, css_vars::RootCssVars, helpers::escape_html_attr,
    helpers::extract_destructure_params,
};
use vize_l0::cstr;

/// One JavaScript property emitted into a generated SSR prop object.
#[derive(Clone, Debug)]
pub(crate) struct VNodePropEntry {
    key: String,
    value: String,
    dynamic: bool,
    /// Authored spans, present only for map-requesting compiles.
    spans: Option<Box<spanned_props::PropEntrySpans>>,
}

impl VNodePropEntry {
    /// Whether the key is a computed (`[key || ""]`) property.
    pub(crate) const fn dynamic(&self) -> bool {
        self.dynamic
    }
}

/// Borrowed or collected children used by SSR component slot codegen.
pub(super) enum ComponentSlotChildren<'node, 'a> {
    Slice(&'node [TemplateChildNode<'a>]),
    Refs(std::vec::Vec<&'node TemplateChildNode<'a>>),
}

/// Vue excludes direct comments from implicit default content beside templates.
fn normalize_implicit_slot_children(children: &mut Vec<&TemplateChildNode<'_>>) {
    children.retain(|child| !matches!(child, TemplateChildNode::Comment(_)));
    if children.iter().all(
        |child| matches!(child, TemplateChildNode::Text(text) if text.content.trim().is_empty()),
    ) {
        children.clear();
    }
}

/// Authored anchors of one slot: its name token and the element carrying it.
#[derive(Clone, Copy, Debug)]
pub(super) struct SlotAnchor {
    /// The `v-slot` argument, when the slot name is authored.
    name: Option<u32>,
    unit: u32,
}

/// A `<template v-slot>` payload normalized before slot function emission.
pub(super) struct ComponentTemplateSlot<'node, 'a> {
    name: String,
    anchor: SlotAnchor,
    props_pattern: Option<String>,
    params: FxHashSet<String>,
    children: &'node [TemplateChildNode<'a>],
}

impl<'a> SsrCodegenContext<'a> {
    /// Process an element node
    pub(crate) fn process_element_with_fallthrough_attrs(
        &mut self,
        el: &ElementNode<'a>,
        disable_nested_fragments: bool,
        inherit_attrs: bool,
        css_vars: RootCssVars,
    ) {
        match el.tag_type {
            ElementType::Element => {
                self.process_plain_element(el, inherit_attrs, css_vars.enabled && !inherit_attrs);
            }
            ElementType::Component => {
                if matches!(el.tag, "TransitionGroup" | "transition-group") {
                    self.process_transition_group(el, inherit_attrs, css_vars.enabled);
                    return;
                }
                self.process_component(
                    el,
                    disable_nested_fragments,
                    inherit_attrs,
                    css_vars.enabled,
                );
            }
            ElementType::Slot => {
                self.process_slot_outlet(el);
            }
            ElementType::Template => {
                // Process template children directly. A template renders no
                // node of its own, so a single child is still the root that
                // inherits the component's fallthrough attrs.
                self.process_children_with_fallthrough_attrs_and_css_vars(
                    &el.children,
                    false,
                    disable_nested_fragments,
                    false,
                    inherit_attrs,
                    RootCssVars {
                        enabled: css_vars.enabled && css_vars.template_wrapper,
                        template_wrapper: false,
                    },
                );
            }
        }
    }
}
