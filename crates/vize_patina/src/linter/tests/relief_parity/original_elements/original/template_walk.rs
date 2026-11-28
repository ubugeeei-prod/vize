//! Depth walk shared by the configurable template rules.
//!
//! Depth starts at 1 for the children of `<template>`, because eslint-plugin-vue
//! counts the template element itself as depth 0. Only elements increment depth.

use vize_relief::{ElementNode, TemplateChildNode};

pub(super) fn walk(
    children: &[TemplateChildNode<'_>],
    depth: u32,
    on_element: &mut impl FnMut(&ElementNode<'_>, u32),
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                on_element(element, depth);
                walk(&element.children, depth + 1, on_element);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    walk(&branch.children, depth, on_element);
                }
            }
            TemplateChildNode::For(node) => walk(&node.children, depth, on_element),
            _ => {}
        }
    }
}
