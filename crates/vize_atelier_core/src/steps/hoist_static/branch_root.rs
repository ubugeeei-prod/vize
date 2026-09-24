//! The element a `v-if` branch renders as its block root.

use crate::{ElementNode, ElementType, TemplateChildNode};

/// The element a `v-if` branch renders as its block root. A `<template v-if>`
/// wrapping one element renders that element directly, so it carries the
/// branch key and must stay inline exactly like an element branch does;
/// hoisting it would leave the branch a keyed fragment around a static vnode.
///
/// Returns that wrapped element, or `None` when `el` is itself the root.
pub(super) fn wrapped_branch_root<'b, 'a>(
    el: &'b mut ElementNode<'a>,
) -> Option<&'b mut ElementNode<'a>> {
    if el.tag_type != ElementType::Template {
        return None;
    }
    match el.children.as_mut_slice() {
        [TemplateChildNode::Element(inner)] if inner.tag_type != ElementType::Template => {
            Some(inner)
        }
        _ => None,
    }
}
