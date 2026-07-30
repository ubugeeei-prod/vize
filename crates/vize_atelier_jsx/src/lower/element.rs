//! Lowering JSX elements and fragments into [`ElementNode`]s.

use oxc_ast::ast::{JSXElement, JSXElementName, JSXFragment};
use vize_relief::ElementNode;
use vize_relief::ElementType;

use super::{Lowerer, name};

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Lower a JSX element into an [`ElementNode`] (tag, kind, props, children).
    pub(crate) fn lower_element_node(&mut self, element: &JSXElement<'_>) -> ElementNode<'a> {
        let opening = &element.opening_element;
        let tag = name::element_tag(&opening.name);
        let loc = self.mapper().location(element.span);
        let mut node = ElementNode::new(self.bump(), tag, loc);
        node.tag_type = element_type(&opening.name);
        node.is_self_closing = element.closing_element.is_none();
        // `v-models` and `v-slots` are component-only. A dashed lowercase tag is
        // classified as an intrinsic element here, but the DOM backend still
        // resolves it with `resolveComponent`, so custom elements count as
        // components for those checks just as they do for
        // `@vue/babel-plugin-jsx`.
        let on_component = node.tag_type == ElementType::Component || node.tag.contains('-');
        node.props = self.lower_attributes(&opening.attributes, on_component);
        // Components route through slot synthesis (object/render-prop children
        // become `<template v-slot>`s); intrinsic elements lower children
        // directly.
        node.children = if node.tag_type == ElementType::Component {
            self.lower_component_children(&element.children)
        } else {
            self.lower_children(&element.children)
        };
        // `v-slots` contributes slot templates, appended after the element's own
        // children so those still become the `default` slot when the slots object
        // does not name one (#3418).
        self.apply_v_slots(&mut node, &opening.attributes, on_component);
        node
    }

    /// Lower a JSX fragment (`<>...</>`) used as a child into an [`ElementNode`]
    /// tagged `Fragment`, matching `@vue/babel-plugin-jsx` semantics.
    pub(crate) fn lower_fragment_node(&mut self, fragment: &JSXFragment<'_>) -> ElementNode<'a> {
        let loc = self.mapper().location(fragment.span);
        let mut node = ElementNode::new(self.bump(), "Fragment", loc);
        node.tag_type = ElementType::Component;
        node.children = self.lower_children(&fragment.children);
        node
    }
}

fn element_type(name: &JSXElementName<'_>) -> ElementType {
    if name::is_component(name) {
        ElementType::Component
    } else {
        ElementType::Element
    }
}
