//! The element a `v-if` branch renders as its block root.

use crate::{ElementNode, ElementType, TemplateChildNode};

/// The element a `v-if` branch renders as its block root. A `<template v-if>`
/// wrapping one element renders that element directly, so it carries the
/// branch key and must stay inline exactly like an element branch does;
/// hoisting it would leave the branch a keyed fragment around a static vnode.
pub(super) fn if_branch_root<'b, 'a>(el: &'b mut ElementNode<'a>) -> &'b mut ElementNode<'a> {
    let wraps_single_element = el.tag_type == ElementType::Template
        && el.children.len() == 1
        && matches!(
            &el.children[0],
            TemplateChildNode::Element(inner) if inner.tag_type != ElementType::Template
        );
    if !wraps_single_element {
        return el;
    }
    match &mut el.children[0] {
        TemplateChildNode::Element(inner) => inner,
        _ => unreachable!("checked above"),
    }
}
