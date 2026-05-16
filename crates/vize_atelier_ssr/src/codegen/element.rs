//! Element, component, and slot processing for SSR code generation.
//!
//! The entry module owns dispatch plus shared data structures. Specialized
//! submodules keep SSR element generation small enough to audit independently.

mod component;
mod plain;
mod props;
mod slot;
mod vnode;

use vize_atelier_core::ast::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper,
    TemplateChildNode,
};
use vize_carton::{FxHashSet, String, ToCompactString};

use super::{helpers::escape_html_attr, helpers::extract_destructure_params, SsrCodegenContext};
use vize_carton::cstr;

/// One JavaScript property emitted into a generated SSR prop object.
#[derive(Clone, Debug)]
pub(super) struct VNodePropEntry {
    key: String,
    value: String,
    dynamic: bool,
}

/// Borrowed or collected children used by SSR component slot codegen.
pub(super) enum ComponentSlotChildren<'node, 'a> {
    Slice(&'node [TemplateChildNode<'a>]),
    Refs(std::vec::Vec<&'node TemplateChildNode<'a>>),
}

/// A `<template v-slot>` payload normalized before slot function emission.
pub(super) struct ComponentTemplateSlot<'node, 'a> {
    name: String,
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
    ) {
        match el.tag_type {
            ElementType::Element => {
                self.process_plain_element(el, inherit_attrs);
            }
            ElementType::Component => {
                self.process_component(el, disable_nested_fragments, inherit_attrs);
            }
            ElementType::Slot => {
                self.process_slot_outlet(el);
            }
            ElementType::Template => {
                // Process template children directly
                self.process_children(&el.children, false, disable_nested_fragments, false);
            }
        }
    }
}
