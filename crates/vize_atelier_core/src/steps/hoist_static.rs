//! Static hoisting transform.
//!
//! Hoists static nodes to reduce runtime overhead.

mod props;
mod static_type;
#[cfg(test)]
mod tests;

use props::{
    create_props_expression, has_static_props, hoist_element_props, is_hoistable_static_prop,
};
pub use static_type::{StaticType, get_static_type, is_static_node};
use static_type::{has_only_native_element_descendants, has_only_static_nested_children};

use vize_s0::{Allocator, Box, String, Vec, ensure_sufficient_stack};

use crate::lane::TransformContext;
use crate::{
    AttributeNode, ElementNode, ElementType, JsChildNode, Namespace, PropNode, RuntimeHelper,
    SourceLocation, TemplateChildNode, TemplateTextChildNode, TextNode, VNodeCall, VNodeChildren,
    VNodeTag,
};

/// Hoist static nodes in the tree
pub fn hoist_static<'a>(
    ctx: &mut TransformContext<'a>,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
) {
    hoist_static_inner(ctx, children, true, false)
}

/// Inner implementation with is_root flag
fn hoist_static_inner<'a>(
    ctx: &mut TransformContext<'a>,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    is_root: bool,
    hoist_static_vnodes: bool,
) {
    if !ctx.options.hoist_static {
        return;
    }

    let allocator = ctx.allocator;
    let mut i = 0;

    while i < children.len() {
        let static_type = get_static_type(&children[i]);

        match static_type {
            StaticType::FullyStatic => {
                // Root elements should NOT be fully hoisted as VNodes
                // They must use createElementBlock for proper block tracking
                // Only hoist their props instead
                if is_root
                    && let TemplateChildNode::Element(el) = &mut children[i]
                    && should_hoist_props(ctx, el)
                {
                    hoist_element_props(ctx, el, allocator);
                } else if hoist_static_vnodes
                    && let TemplateChildNode::Element(el) = &mut children[i]
                {
                    // Scope ids are computed once per compile and repeat on
                    // every hoisted element, so they land in the arena as atoms.
                    let scope_id = ctx
                        .hoisted_scope_id
                        .as_deref()
                        .or(ctx.options.scope_id.as_deref())
                        .map(|id| allocator.alloc_str(id));
                    let vnode_call = create_vnode_call_from_element(allocator, el, scope_id);
                    let hoist_index = ctx.hoist(vnode_call);
                    children[i] = TemplateChildNode::Hoisted(hoist_index);
                    ctx.helper(RuntimeHelper::CreateElementVNode);
                }
            }
            StaticType::HasDynamicText => {
                // Root elements with dynamic text still benefit from hoisted
                // static props while preserving block tracking.
                if is_root
                    && let TemplateChildNode::Element(el) = &mut children[i]
                    && should_hoist_props(ctx, el)
                {
                    hoist_element_props(ctx, el, allocator);
                }
            }
            StaticType::NotStatic => {
                // Cannot hoist, but check children recursively (not as root)
                match &mut children[i] {
                    TemplateChildNode::Element(el) => {
                        if should_hoist_props(ctx, el)
                            && ((is_root
                                && ctx.options.inline
                                && has_only_native_element_descendants(el))
                                || el.ns != Namespace::Html
                                || has_only_static_nested_children(el))
                        {
                            hoist_element_props(ctx, el, allocator);
                        }
                        let child_hoist_static_vnodes = hoist_static_vnodes
                            || has_directives(el)
                            || el.tag_type != ElementType::Element;
                        // Guarded: one frame per nesting level.
                        ensure_sufficient_stack(|| {
                            hoist_static_inner(
                                ctx,
                                &mut el.children,
                                false,
                                child_hoist_static_vnodes,
                            );
                        });
                    }
                    TemplateChildNode::If(if_node) => {
                        // For v-if branches, only hoist nested children, not the branch root
                        // The branch root needs a key and must be created inline
                        for branch in if_node.branches.iter_mut() {
                            for child in branch.children.iter_mut() {
                                if let TemplateChildNode::Element(el) = child {
                                    // Only hoist inside the branch root's children
                                    ensure_sufficient_stack(|| {
                                        hoist_static_inner(ctx, &mut el.children, false, true);
                                    });
                                }
                            }
                        }
                    }
                    TemplateChildNode::For(for_node) => {
                        hoist_for_children(ctx, &mut for_node.children, allocator);
                    }
                    _ => {}
                }
            }
        }
        i += 1;
    }
}

fn hoist_for_children<'a>(
    ctx: &mut TransformContext<'a>,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    allocator: &'a Allocator,
) {
    match children.as_mut_slice() {
        [TemplateChildNode::Element(el)] if el.tag_type == ElementType::Template => {
            hoist_for_children(ctx, &mut el.children, allocator);
        }
        [TemplateChildNode::Element(el)] => {
            // A single v-for child is the loop item's block root, so the VNode
            // itself must stay inline. Static props and nested static children
            // can still be hoisted/cached independently.
            if should_hoist_props(ctx, el) {
                hoist_element_props(ctx, el, allocator);
            }
            ensure_sufficient_stack(|| {
                hoist_static_inner(ctx, &mut el.children, false, true);
            });
        }
        _ => {
            ensure_sufficient_stack(|| {
                hoist_static_inner(ctx, children, false, true);
            });
        }
    }
}

/// Create a VNodeCall from an ElementNode for hoisting.
///
/// Takes the element by `&mut` because the caller replaces it with a
/// `Hoisted` reference immediately afterwards; this lets the nested-children
/// case move the (already fully-static) child subtree into the VNodeCall
/// instead of deep-cloning it.
fn create_vnode_call_from_element<'a>(
    allocator: &'a Allocator,
    el: &mut ElementNode<'a>,
    scope_id: Option<&'a str>,
) -> JsChildNode<'a> {
    let tag = VNodeTag::String(el.tag);
    let props = create_props_expression(allocator, &el.props, scope_id, false);
    let children = create_children_expression(allocator, &mut el.children, scope_id);

    let vnode_call = VNodeCall {
        tag,
        props,
        children,
        patch_flag: None,
        dynamic_props: None,
        directives: None,
        is_block: false,
        disable_tracking: false,
        is_component: false,
        loc: el.loc.clone(),
    };

    JsChildNode::VNodeCall(Box::new_in(vnode_call, &allocator))
}

/// Create children expression from template children.
///
/// `scope_id` is threaded so nested hoisted elements carry the same scoped-CSS
/// attribute their parent does.
fn create_children_expression<'a>(
    allocator: &'a Allocator,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    scope_id: Option<&'a str>,
) -> Option<VNodeChildren<'a>> {
    if children.is_empty() {
        return None;
    }

    // For a single text child, use Single variant with Text
    if children.len() == 1
        && let TemplateChildNode::Text(text) = &children[0]
    {
        let text_node = TextNode::new(text.content, text.loc.clone());
        return Some(VNodeChildren::Single(TemplateTextChildNode::Text(
            Box::new_in(text_node, &allocator),
        )));
    }

    // For all-text children, combine them into one text node.
    if children
        .iter()
        .all(|c| matches!(c, TemplateChildNode::Text(_)))
    {
        let mut text_content = String::default();
        for child in children.iter() {
            if let TemplateChildNode::Text(text) = child {
                text_content.push_str(text.content);
            }
        }
        if !text_content.is_empty() {
            let text_node = TextNode::new(allocator.alloc_str(&text_content), SourceLocation::STUB);
            return Some(VNodeChildren::Single(TemplateTextChildNode::Text(
                Box::new_in(text_node, &allocator),
            )));
        }
    }

    // Nested elements (and mixed element/text) children. Move the child nodes
    // into a `Multiple` so a fully-static subtree hoists into a single
    // recursive `createElementVNode(...)`, matching @vue/compiler-core. The
    // subtree is already known to be fully static (the caller only reaches this
    // path for `StaticType::FullyStatic` elements), so codegen serializes each
    // element child recursively as a nested `createElementVNode`. Moving rather
    // than cloning is sound because the caller replaces this element with a
    // `Hoisted` reference right after, so the original children are unreachable.
    let mut moved = std::mem::replace(children, Vec::new_in(&allocator));

    // Scoped CSS: every native element in the hoisted subtree needs the
    // `data-v-xxxxxxxx` attribute, so inject it into each nested element.
    if let Some(scope_id) = scope_id {
        for child in moved.iter_mut() {
            inject_scope_id(allocator, child, scope_id);
        }
    }

    Some(VNodeChildren::Multiple(moved))
}

/// Recursively add the scoped-CSS attribute to a static element subtree so
/// hoisted nested vnodes carry `data-v-xxxxxxxx` like their parent.
fn inject_scope_id<'a>(
    allocator: &'a Allocator,
    node: &mut TemplateChildNode<'a>,
    scope_id: &'a str,
) {
    if let TemplateChildNode::Element(el) = node {
        let already = el.props.iter().any(|p| match p {
            PropNode::Attribute(a) => a.name == scope_id,
            PropNode::Directive(_) => false,
        });
        if !already {
            el.props.push(PropNode::Attribute(Box::new_in(
                AttributeNode {
                    name: scope_id,
                    name_loc: SourceLocation::STUB,
                    value: None,
                    loc: SourceLocation::STUB,
                },
                &allocator,
            )));
        }
        // Guarded: one frame per nesting level of the hoisted subtree.
        ensure_sufficient_stack(|| {
            for child in el.children.iter_mut() {
                inject_scope_id(allocator, child, scope_id);
            }
        });
    }
}

fn has_directives(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(_)))
}

fn should_hoist_props(ctx: &TransformContext<'_>, el: &ElementNode<'_>) -> bool {
    has_static_props(el)
        && !(ctx.hoisted_scope_id.is_some() && el.tag_type == ElementType::Component)
}

/// Check if children should use a block
pub fn should_use_block(el: &ElementNode<'_>) -> bool {
    // Use block for elements with v-for, v-if, or components
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop
            && (dir.name == "for" || dir.name == "if")
        {
            return true;
        }
    }

    el.tag_type == ElementType::Component
}

/// Count dynamic children for optimization hints
pub fn count_dynamic_children(children: &[TemplateChildNode<'_>]) -> usize {
    let mut count = 0;

    for child in children {
        match child {
            TemplateChildNode::Interpolation(_) => count += 1,
            TemplateChildNode::Element(el) => {
                // Check for dynamic props
                for prop in el.props.iter() {
                    if let PropNode::Directive(_) = prop {
                        count += 1;
                        break;
                    }
                }
            }
            TemplateChildNode::If(_) | TemplateChildNode::For(_) => count += 1,
            _ => {}
        }
    }

    count
}
