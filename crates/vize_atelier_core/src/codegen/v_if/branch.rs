//! v-if branch generation.
//!
//! Generates code for individual v-if/v-else-if/v-else branches including
//! component, element, template fragment, and regular fragment rendering.

use crate::relief_projection::{ReliefChildren, ReliefElementKind};
use crate::{ElementNode, ForNode, IfBranchNode, PropNode, RuntimeHelper, TemplateChildNode};
use vize_carton::ToCompactString;

use super::{
    super::{
        context::CodegenContext,
        element::helpers::child_namespace,
        element::{
            generate_custom_directives_closing, generate_vmodel_closing, generate_vshow_closing,
            has_custom_directives, has_vmodel_directive, has_vshow_directive,
        },
        expression::generate_expression,
        helpers::escape_js_string,
        node::dispatch_relief_op,
        patch_flag::{calculate_element_patch_info, patch_flag_name},
        slots::{generate_slot_outlet_name, generate_slot_outlet_props_with_key},
    },
    branch_component::generate_if_branch_component,
    branch_fragment::{
        generate_if_branch_children, generate_if_branch_fragment,
        generate_if_branch_template_fragment,
    },
    generate::{
        extract_static_class_style, generate_if_branch_props_object, has_dynamic_class,
        has_dynamic_style, has_vbind_spread, has_von_spread,
    },
    generate_if_branch_key,
};

/// Generate a single if branch.
pub(super) fn generate_if_branch(
    ctx: &mut CodegenContext,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    // Single child optimization
    if branch.children.len() == 1 {
        match &branch.children[0] {
            TemplateChildNode::Element(el) => {
                // Check if it's a template element - treat as fragment
                if ReliefElementKind::from(el.tag_type).is_template() {
                    // Template with single child -> unwrap to single element
                    if el.children.len() == 1 {
                        match &el.children[0] {
                            TemplateChildNode::Element(inner) => {
                                if ReliefElementKind::from(inner.tag_type).is_component() {
                                    generate_if_branch_component(ctx, inner, branch, branch_index);
                                } else if ReliefElementKind::from(inner.tag_type).is_slot_outlet() {
                                    generate_if_branch_slot(ctx, inner, branch, branch_index);
                                } else {
                                    generate_if_branch_element(ctx, inner, branch, branch_index);
                                }
                                return;
                            }
                            TemplateChildNode::For(for_node) => {
                                generate_if_branch_for(ctx, for_node, branch, branch_index);
                                return;
                            }
                            _ => {}
                        }
                    }
                    // Template with multiple children -> fragment
                    generate_if_branch_template_fragment(ctx, &el.children, branch, branch_index);
                } else if ReliefElementKind::from(el.tag_type).is_component() {
                    // Component
                    generate_if_branch_component(ctx, el, branch, branch_index);
                } else if ReliefElementKind::from(el.tag_type).is_slot_outlet() {
                    generate_if_branch_slot(ctx, el, branch, branch_index);
                } else {
                    // Regular element
                    generate_if_branch_element(ctx, el, branch, branch_index);
                }
            }
            _ => {
                // Other node types - wrap in fragment
                if let TemplateChildNode::For(for_node) = &branch.children[0] {
                    generate_if_branch_for(ctx, for_node, branch, branch_index);
                } else {
                    generate_if_branch_fragment(ctx, branch, branch_index);
                }
            }
        }
    } else {
        // Multiple children - wrap in fragment
        generate_if_branch_fragment(ctx, branch, branch_index);
    }
}

fn generate_if_branch_for(
    ctx: &mut CodegenContext,
    for_node: &ForNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    super::super::v_for::generate_for_with_fragment_key(ctx, for_node, &|ctx| {
        super::generate_if_branch_key(ctx, branch, branch_index);
    });
}

/// Generate slot outlet for if branch.
fn generate_if_branch_slot(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    // Slots don't use blocks in branch output; use renderSlot directly.
    ctx.use_helper(RuntimeHelper::RenderSlot);
    ctx.push(ctx.helper(RuntimeHelper::RenderSlot));
    ctx.push("(_ctx.$slots, ");
    generate_slot_outlet_name(ctx, el);
    ctx.push(", ");
    let generate_key = |ctx: &mut CodegenContext| generate_if_branch_key(ctx, branch, branch_index);
    generate_slot_outlet_props_with_key(ctx, el, &generate_key);

    if !el.children.is_empty() {
        ctx.push(", () => [");
        for (i, (op, node)) in ReliefChildren::new(&el.children).rendered().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            dispatch_relief_op(ctx, op, node);
        }
        ctx.push("]");
    }
    ctx.push(")");
}

/// Generate element for if branch.
fn generate_if_branch_element(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    let (patch_flag, dynamic_props) = calculate_element_patch_info(
        el,
        ctx.options.binding_metadata.as_ref(),
        ctx.cache_handlers_in_current_scope(),
    );
    let has_patch_info = patch_flag.is_some() || dynamic_props.is_some();

    let has_custom_dirs = has_custom_directives(el);
    if has_custom_dirs {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }
    let has_vmodel = has_vmodel_directive(el) && !has_custom_dirs;
    if has_vmodel {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }
    let has_vshow = has_vshow_directive(el) && !has_vmodel && !has_custom_dirs;
    if has_vshow {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.use_helper(RuntimeHelper::VShow);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }

    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(\"");
    ctx.push(el.tag.as_str());
    ctx.push("\"");

    // Extract static class/style for merging with dynamic bindings
    let static_merge = extract_static_class_style(el);
    let has_dyn_class = has_dynamic_class(el);
    let has_dyn_style = has_dynamic_style(el);

    // Generate props with key and all other props (handle v-bind/v-on spreads)
    let has_vbind = has_vbind_spread(el);
    let has_von = has_von_spread(el);
    if has_vbind || has_von {
        ctx.use_helper(RuntimeHelper::MergeProps);
        ctx.push(", ");
        ctx.push(ctx.helper(RuntimeHelper::MergeProps));
        ctx.push("(");

        // Add all v-bind spreads
        let mut first_merge_arg = true;
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "bind"
                && dir.arg.is_none()
                && let Some(exp) = &dir.exp
            {
                if !first_merge_arg {
                    ctx.push(", ");
                }
                generate_expression(ctx, exp);
                first_merge_arg = false;
            }
        }

        // Add all v-on spreads wrapped with _toHandlers
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "on"
                && dir.arg.is_none()
                && let Some(exp) = &dir.exp
            {
                if !first_merge_arg {
                    ctx.push(", ");
                }
                ctx.use_helper(RuntimeHelper::ToHandlers);
                ctx.push(ctx.helper(RuntimeHelper::ToHandlers));
                ctx.push("(");
                generate_expression(ctx, exp);
                ctx.push(", true)");
                first_merge_arg = false;
            }
        }

        if !first_merge_arg {
            ctx.push(", ");
        }
        generate_if_branch_props_object(
            ctx,
            el,
            branch,
            branch_index,
            static_merge,
            has_dyn_class,
            has_dyn_style,
            false,
        );
        ctx.push(")");
    } else {
        ctx.push(", ");
        generate_if_branch_props_object(
            ctx,
            el,
            branch,
            branch_index,
            static_merge,
            has_dyn_class,
            has_dyn_style,
            false,
        );
    }

    // Generate children if any
    if !el.children.is_empty() {
        ctx.push(", ");
        if el.children.len() == 1 {
            if let TemplateChildNode::Text(text) = &el.children[0] {
                ctx.push("\"");
                ctx.push(&escape_js_string(text.content.as_str()));
                ctx.push("\"");
            } else {
                ctx.with_parent_namespace(child_namespace(el), |ctx| {
                    generate_if_branch_children(ctx, &el.children);
                });
            }
        } else {
            ctx.with_parent_namespace(child_namespace(el), |ctx| {
                generate_if_branch_children(ctx, &el.children);
            });
        }
    } else if has_patch_info {
        ctx.push(", null");
    }

    if let Some(flag) = patch_flag {
        ctx.push(", ");
        ctx.push(&flag.to_compact_string());
        ctx.push(" /* ");
        let flag_name = patch_flag_name(flag);
        ctx.push(&flag_name);
        ctx.push(" */");
    }

    if let Some(props) = dynamic_props {
        ctx.push(", [");
        for (i, prop) in props.iter().enumerate() {
            if i > 0 {
                ctx.push(", ");
            }
            ctx.push("\"");
            ctx.push(prop);
            ctx.push("\"");
        }
        ctx.push("]");
    }

    ctx.push("))");

    if has_custom_dirs {
        generate_custom_directives_closing(ctx, el);
    }
    if has_vmodel {
        generate_vmodel_closing(ctx, el);
    }
    if has_vshow {
        generate_vshow_closing(ctx, el);
    }
}
