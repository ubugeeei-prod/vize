//! AST traversal functions for template transformation.

use crate::steps::v_slot::{get_slot_name, get_slot_prop_names, get_slot_props_string};
use crate::{
    ElementNode, ElementType, ExpressionNode, ForNode, IfBranchNode, PropNode, RuntimeHelper,
    TemplateChildNode,
};
use vize_carton::{ensure_sufficient_stack, profile};

use super::element::{transform_element, transform_interpolation};
use super::structural::{
    StructuralDirectiveKind, take_structural_directive, transform_v_for,
    transform_v_if_with_directive,
};
use super::{ExitFns, ParentNode, TransformContext};

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

/// Traverse children of a parent node
pub fn traverse_children<'a>(ctx: &mut TransformContext<'a>, parent: ParentNode<'a>) {
    let mut i = 0;

    while i < parent.children_mut().len() {
        ctx.grandparent = ctx.parent;
        ctx.parent = Some(parent);
        ctx.child_index = i;
        ctx.reset_node_removed();

        {
            let children = parent.children_mut();
            traverse_node(ctx, &mut children[i]);
        }

        let node_removed = ctx.was_node_removed();
        ctx.reset_node_removed();

        if node_removed {
            let children = parent.children_mut();
            if i < children.len() {
                children.remove(i);
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
/// (`vize_carton::recursion`).
pub fn traverse_node<'a>(ctx: &mut TransformContext<'a>, node: &mut TemplateChildNode<'a>) {
    crate::walk_probe::record_visit(crate::walk_probe::WalkStage::Transform);
    ensure_sufficient_stack(|| traverse_node_guarded(ctx, node));
}

fn traverse_node_guarded<'a>(ctx: &mut TransformContext<'a>, node: &mut TemplateChildNode<'a>) {
    // Collect exit functions from transforms
    let mut exit_fns = ExitFns::new();

    // Check structural directives before storing the current-node pointer so
    // the `Element` borrow ends before structural transforms replace the slot.
    let structural_result = if let TemplateChildNode::Element(el) = node {
        profile!(
            "atelier.transform.check_structural",
            take_structural_directive(el, ctx.source)
        )
    } else {
        None
    };

    ctx.current_node = Some(node as *mut _);

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
                        true
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
                        false
                    )
                ) {
                    exit_fns.extend(exits);
                }
            }
            StructuralDirectiveKind::For => {
                if let Some(exits) = profile!(
                    "atelier.transform.v_for",
                    transform_v_for(ctx, exp.as_ref())
                ) {
                    exit_fns.extend(exits);
                }
            }
        }

        // If node was replaced (e.g., by v-if transform), we need to traverse the new node
        if let Some(current_ptr) = ctx.current_node {
            // SAFETY: `ctx.current_node` points at the child slot currently owned
            // by the traversal loop. Structural transforms may replace that slot,
            // but they do not retain another mutable reference to it.
            let current = unsafe { &mut *current_ptr };
            match current {
                TemplateChildNode::If(if_node) => {
                    // Traverse if branches that were just created
                    for i in 0..if_node.branches.len() {
                        let branch_ptr = &mut if_node.branches[i] as *mut IfBranchNode<'a>;
                        profile!(
                            "atelier.transform.traverse_v_if_branch",
                            traverse_children(ctx, ParentNode::IfBranch(branch_ptr))
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
                            compound_source =
                                vize_carton::String::new(c.loc.span.slice(ctx.source));
                            compound_source.as_str()
                        }
                    };
                    ctx.enter_v_for_scope(value, key, index, source);

                    // Traverse for children
                    let for_ptr = for_node.as_mut() as *mut ForNode<'a>;
                    profile!(
                        "atelier.transform.traverse_v_for_children",
                        traverse_children(ctx, ParentNode::For(for_ptr))
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
            // Node was removed, return early
            return;
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
            TemplateChildNode::Interpolation(interp) => {
                profile!(
                    "atelier.transform.interpolation",
                    transform_interpolation(ctx, interp)
                );
            }
            TemplateChildNode::Text(_) => {
                ctx.helper(RuntimeHelper::CreateText);
            }
            TemplateChildNode::Comment(_) => {
                ctx.helper(RuntimeHelper::CreateComment);
            }
            TemplateChildNode::If(if_node) => {
                // Traverse if branches
                for i in 0..if_node.branches.len() {
                    let branch_ptr = &mut if_node.branches[i] as *mut IfBranchNode<'a>;
                    profile!(
                        "atelier.transform.traverse_v_if_branch",
                        traverse_children(ctx, ParentNode::IfBranch(branch_ptr))
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
                        compound_source = vize_carton::String::new(c.loc.span.slice(ctx.source));
                        compound_source.as_str()
                    }
                };
                ctx.enter_v_for_scope(value, key, index, source);

                // Traverse for children
                let for_ptr = for_node.as_mut() as *mut ForNode<'a>;
                profile!(
                    "atelier.transform.traverse_v_for_children",
                    traverse_children(ctx, ParentNode::For(for_ptr))
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
        let el_ptr = el.as_mut() as *mut ElementNode<'a>;
        profile!(
            "atelier.transform.traverse_element_children",
            traverse_children(ctx, ParentNode::Element(el_ptr))
        );
        if entered_slot_scope {
            ctx.exit_scope();
        }
    }

    // Call exit functions in reverse order
    ctx.current_node = Some(node as *mut _);
    for exit_fn in exit_fns.into_iter().rev() {
        profile!("atelier.transform.exit_fn", exit_fn(ctx));
    }
}
