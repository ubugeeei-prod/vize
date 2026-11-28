//! AST traversal functions for template transformation.

mod removal;

use crate::steps::v_slot::{get_slot_name, get_slot_prop_names, get_slot_props_string};
use crate::{ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode};
use vize_l0::{Vec, ensure_sufficient_stack, profile};

use super::element::{transform_element, transform_interpolation};
use super::structural::{
    MATCH_SCOPE_RAW_NAME, StructuralDirectiveKind, take_structural_directive, transform_for_scope,
    transform_v_if_with_directive,
};
use super::{ExitFns, TransformContext};

fn enter_v_slot_scope_if_needed<'a>(ctx: &mut TransformContext<'a>, el: &ElementNode<'a>) -> bool {
    if el.children.is_empty() || (el.tag_type != ElementType::Component && el.tag != "template") {
        return false;
    }

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            if dir.name != "slot" {
                continue;
            }

            let prop_names = get_slot_prop_names(dir, ctx.source);
            if prop_names.is_empty() {
                return false;
            }

            let slot_name = get_slot_name(dir, ctx.source);
            let props_pattern = get_slot_props_string(dir, ctx.source);
            ctx.enter_v_slot_scope(
                slot_name.as_str(),
                props_pattern.as_ref().map(|pattern| pattern.as_str()),
                &prop_names,
                dir.loc.span.start,
                el.loc.span.end,
            );
            return true;
        }
    }

    false
}

/// Traverse the children of a parent node.
///
/// Each child is lent to [`traverse_node`] together with the siblings before
/// it, split from the same vector, so a `v-else` can join the `v-if` before it
/// while the borrow checker keeps the two borrows apart.
pub fn traverse_children<'a>(
    ctx: &mut TransformContext<'a>,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
) {
    let mut i = 0;

    while i < children.len() {
        ctx.child_index = i;
        ctx.reset_node_removed();

        if let Some((before, rest)) = children.split_at_mut_checked(i)
            && let Some(node) = rest.first_mut()
        {
            traverse_node(ctx, before, node);
        }

        let node_removed_from = ctx.node_removed_from;
        ctx.reset_node_removed();

        if let Some(from) = node_removed_from {
            if i < children.len() {
                children.remove(i);
                removal::remove_branch_whitespace(children, &mut i, from);
            }
        } else {
            i += 1;
        }
    }
}

/// Traverse a single node
///
/// This is the transform lane's recursion step: `traverse_children` calls it per
/// child and it calls `traverse_children` back for that child's subtree, so the
/// call depth here follows template nesting depth. The guard moves the descent
/// onto a fresh stack before the thread stack runs out, because a stack overflow
/// aborts the process instead of producing a diagnostic
/// (`vize_l0::recursion`).
/// `before` holds the siblings that precede `node` in its parent.
pub fn traverse_node<'a>(
    ctx: &mut TransformContext<'a>,
    before: &mut [TemplateChildNode<'a>],
    node: &mut TemplateChildNode<'a>,
) {
    crate::walk_probe::record_visit(crate::walk_probe::WalkStage::Transform);
    ensure_sufficient_stack(|| traverse_node_guarded(ctx, before, node));
}

fn traverse_node_guarded<'a>(
    ctx: &mut TransformContext<'a>,
    before: &mut [TemplateChildNode<'a>],
    node: &mut TemplateChildNode<'a>,
) {
    // Leaves cannot own structural directives, children, or exit callbacks. Keep
    // their expression/scope work, but avoid constructing and dropping an empty
    // callback list for every interpolation, text node, and comment.
    match node {
        TemplateChildNode::Interpolation(interp) => {
            profile!(
                "atelier.transform.interpolation",
                transform_interpolation(ctx, interp)
            );
            return;
        }
        TemplateChildNode::Text(_) => {
            ctx.helper(RuntimeHelper::CreateText);
            return;
        }
        TemplateChildNode::Comment(_) => {
            ctx.helper(RuntimeHelper::CreateComment);
            return;
        }
        _ => {}
    }

    // Collect exit functions from transforms
    let mut exit_fns = ExitFns::new();

    // Read structural directives first so the `Element` borrow ends before a
    // structural transform replaces the slot.
    let match_scope = matches!(node, TemplateChildNode::Element(el) if el.props.iter().any(
        |prop| matches!(prop, PropNode::Directive(dir) if dir.raw_name == Some(MATCH_SCOPE_RAW_NAME))
    ));
    let structural_result = if let TemplateChildNode::Element(el) = node {
        profile!(
            "atelier.transform.check_structural",
            take_structural_directive(el, ctx.source)
        )
    } else {
        None
    };

    if let Some((directive_kind, exp, directive_loc)) = structural_result {
        // Handle the structural directive
        match directive_kind {
            StructuralDirectiveKind::If => {
                if let Some(exits) = profile!(
                    "atelier.transform.v_if",
                    transform_v_if_with_directive(
                        ctx,
                        directive_kind,
                        exp.as_ref(),
                        &directive_loc,
                        None,
                        node
                    )
                ) {
                    exit_fns.extend(exits);
                }
            }
            StructuralDirectiveKind::ElseIf | StructuralDirectiveKind::Else => {
                if let Some(exits) = profile!(
                    "atelier.transform.v_if",
                    transform_v_if_with_directive(
                        ctx,
                        directive_kind,
                        exp.as_ref(),
                        &directive_loc,
                        Some(before),
                        node
                    )
                ) {
                    exit_fns.extend(exits);
                }
            }
            StructuralDirectiveKind::For => {
                if let Some(exits) = profile!(
                    "atelier.transform.v_for",
                    transform_for_scope(ctx, exp.as_ref(), match_scope, node)
                ) {
                    exit_fns.extend(exits);
                }
            }
        }

        // A `v-else` joined the `v-if` before it: its slot is removed.
        if ctx.was_node_removed() {
            return;
        }
        // The transform may have replaced the slot (`v-if`, `v-for`), so
        // traverse the node it now holds.
        match node {
            TemplateChildNode::If(if_node) => {
                // Traverse if branches that were just created
                for branch in if_node.branches.iter_mut() {
                    profile!(
                        "atelier.transform.traverse_v_if_branch",
                        traverse_children(ctx, &mut branch.children)
                    );
                }
                // Run exit functions and return early
                for exit_fn in exit_fns.into_iter().rev() {
                    profile!("atelier.transform.exit_fn", exit_fn(ctx));
                }
                return;
            }
            TemplateChildNode::For(for_node) => {
                // Enter v-for scope with aliases
                let value = for_node.value_alias.as_ref().and_then(|e| {
                    if let ExpressionNode::Simple(exp) = e {
                        Some(exp.content)
                    } else {
                        None
                    }
                });
                let key = for_node.key_alias.as_ref().and_then(|e| {
                    if let ExpressionNode::Simple(exp) = e {
                        Some(exp.content)
                    } else {
                        None
                    }
                });
                let index = for_node.object_index_alias.as_ref().and_then(|e| {
                    if let ExpressionNode::Simple(exp) = e {
                        Some(exp.content)
                    } else {
                        None
                    }
                });
                let compound_source;
                let source = match &for_node.source {
                    ExpressionNode::Simple(exp) => exp.content,
                    ExpressionNode::Compound(c) => {
                        compound_source = vize_l0::String::new(c.loc.span.slice(ctx.source));
                        compound_source.as_str()
                    }
                };
                ctx.enter_v_for_scope_with_names(
                    value,
                    key,
                    index,
                    source,
                    [
                        for_node.value_alias.as_ref().and_then(alias_names),
                        for_node.key_alias.as_ref().and_then(alias_names),
                        for_node.object_index_alias.as_ref().and_then(alias_names),
                    ],
                );

                // Traverse for children
                profile!(
                    "atelier.transform.traverse_v_for_children",
                    traverse_children(ctx, &mut for_node.children)
                );

                // Exit v-for scope
                ctx.exit_scope();

                // Add helpers
                ctx.helper(RuntimeHelper::RenderList);
                ctx.helper(RuntimeHelper::Fragment);

                // Run exit functions and return early
                for exit_fn in exit_fns.into_iter().rev() {
                    profile!("atelier.transform.exit_fn", exit_fn(ctx));
                }
                return;
            }
            TemplateChildNode::Element(el) => {
                // Still an element, process it
                if let Some(exits) =
                    profile!("atelier.transform.element", transform_element(ctx, el))
                {
                    exit_fns.extend(exits);
                }
            }
            _ => {}
        }
    } else {
        // Apply node transforms based on node type
        match node {
            TemplateChildNode::Element(el) => {
                if let Some(exits) =
                    profile!("atelier.transform.element", transform_element(ctx, el))
                {
                    exit_fns.extend(exits);
                }
            }
            TemplateChildNode::If(if_node) => {
                // Traverse if branches
                for branch in if_node.branches.iter_mut() {
                    profile!(
                        "atelier.transform.traverse_v_if_branch",
                        traverse_children(ctx, &mut branch.children)
                    );
                }
            }
            TemplateChildNode::For(for_node) => {
                // Enter v-for scope with aliases
                let value = for_node.value_alias.as_ref().and_then(|e| {
                    if let ExpressionNode::Simple(exp) = e {
                        Some(exp.content)
                    } else {
                        None
                    }
                });
                let key = for_node.key_alias.as_ref().and_then(|e| {
                    if let ExpressionNode::Simple(exp) = e {
                        Some(exp.content)
                    } else {
                        None
                    }
                });
                let index = for_node.object_index_alias.as_ref().and_then(|e| {
                    if let ExpressionNode::Simple(exp) = e {
                        Some(exp.content)
                    } else {
                        None
                    }
                });
                let compound_source;
                let source = match &for_node.source {
                    ExpressionNode::Simple(exp) => exp.content,
                    ExpressionNode::Compound(c) => {
                        compound_source = vize_l0::String::new(c.loc.span.slice(ctx.source));
                        compound_source.as_str()
                    }
                };
                ctx.enter_v_for_scope_with_names(
                    value,
                    key,
                    index,
                    source,
                    [
                        for_node.value_alias.as_ref().and_then(alias_names),
                        for_node.key_alias.as_ref().and_then(alias_names),
                        for_node.object_index_alias.as_ref().and_then(alias_names),
                    ],
                );

                // Traverse for children
                profile!(
                    "atelier.transform.traverse_v_for_children",
                    traverse_children(ctx, &mut for_node.children)
                );

                // Exit v-for scope
                ctx.exit_scope();

                // Add helpers
                ctx.helper(RuntimeHelper::RenderList);
                ctx.helper(RuntimeHelper::Fragment);
            }
            _ => {}
        }
    }

    // Traverse children for element nodes
    if let TemplateChildNode::Element(el) = node
        && !el.children.is_empty()
    {
        let entered_slot_scope = enter_v_slot_scope_if_needed(ctx, el);
        profile!(
            "atelier.transform.traverse_element_children",
            traverse_children(ctx, &mut el.children)
        );
        if entered_slot_scope {
            ctx.exit_scope();
        }
    }

    // Call exit functions in reverse order
    for exit_fn in exit_fns.into_iter().rev() {
        profile!("atelier.transform.exit_fn", exit_fn(ctx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TransformOptions, parser::parse};
    use vize_l0::Allocator;

    #[test]
    fn leaf_fast_path_preserves_helpers_and_scopes() {
        for vapor in [false, true] {
            let allocator = Allocator::new();
            let source = "text<!--comment-->{{ item }}{{ outer }}";
            let (mut root, errors) = parse(&allocator, source);
            assert!(errors.is_empty());
            let mut ctx = TransformContext::new(
                &allocator,
                source,
                TransformOptions {
                    prefix_identifiers: true,
                    vapor,
                    ..Default::default()
                },
            );
            ctx.enter_v_for_scope(Some("item"), None, None, "items");

            traverse_children(&mut ctx, &mut root.children);

            assert!(ctx.errors.is_empty());
            assert_eq!(ctx.helpers.count(RuntimeHelper::CreateText), 1);
            assert_eq!(ctx.helpers.count(RuntimeHelper::CreateComment), 1);
            assert_eq!(ctx.helpers.count(RuntimeHelper::ToDisplayString), 2);
            assert_eq!(
                ctx.helpers.iter().collect::<std::vec::Vec<_>>(),
                [
                    RuntimeHelper::CreateText,
                    RuntimeHelper::CreateComment,
                    RuntimeHelper::ToDisplayString,
                ]
            );
            let mut expressions = std::vec::Vec::new();
            for node in root.children.iter_mut() {
                if let TemplateChildNode::Interpolation(interp) = node {
                    let ExpressionNode::Simple(exp) = &interp.content else {
                        panic!("expected a simple interpolation expression");
                    };
                    expressions.push((exp.content, exp.loc.span.slice(source)));
                }
            }
            assert_eq!(expressions, [("item", "item"), ("_ctx.outer", "outer")]);
        }
    }
}

fn alias_names<'n, 'a>(expression: &'n ExpressionNode<'a>) -> Option<&'n [&'a str]> {
    match expression {
        ExpressionNode::Simple(node) => node.retained_parameter_names(),
        _ => None,
    }
}
