//! v-if component-branch generation.
//!
//! Emits a single v-if / v-else-if / v-else branch whose content is a
//! component. Split out of `branch` to keep that file focused on branch
//! dispatch and the element/fragment shapes.

use crate::{
    ElementNode, ExpressionNode, IfBranchNode, PropNode, RuntimeHelper, rendu::RenduChildren,
};
use vize_carton::ToCompactString;

use super::{
    super::{
        context::CodegenContext,
        element::{
            generate_custom_directives_closing, generate_vshow_closing, has_custom_directives,
            has_vshow_directive,
        },
        element::{helpers::is_dynamic_component, is_whitespace_or_comment},
        expression::generate_expression,
        helpers::{is_builtin_component, to_valid_asset_identifier},
        node::dispatch_rendu_op,
        patch_flag::{
            calculate_element_patch_info, calculate_element_patch_info_skip_is, patch_flag_name,
        },
        slots::{
            generate_slots, has_dynamic_slots_flag, has_forwarded_slot_outlet, has_slot_children,
        },
    },
    generate::{
        extract_static_class_style, generate_if_branch_props_object, has_dynamic_class,
        has_dynamic_style, has_vbind_spread, has_von_spread,
    },
};

/// Generate component for if branch.
pub(super) fn generate_if_branch_component(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    let is_dynamic = is_dynamic_component(el);
    let has_custom_dirs = has_custom_directives(el);
    if has_custom_dirs {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }
    let has_vshow = has_vshow_directive(el) && !has_custom_dirs;
    if has_vshow {
        ctx.use_helper(RuntimeHelper::WithDirectives);
        ctx.use_helper(RuntimeHelper::VShow);
        ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
        ctx.push("(");
    }

    let prev_skip_scope_id = ctx.skip_scope_id;
    ctx.use_helper(RuntimeHelper::CreateBlock);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateBlock));
    ctx.push("(");
    // Generate component name
    // Handle dynamic component (<component :is="..."> / <Component :is="...">)
    if is_dynamic {
        let dynamic_is = el.props.iter().find_map(|p| {
            if let PropNode::Directive(dir) = p
                && dir.name == "bind"
                && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                && arg.content == "is"
            {
                return dir.exp.as_ref();
            }
            None
        });
        let static_is = el.props.iter().find_map(|p| {
            if let PropNode::Attribute(attr) = p
                && attr.name == "is"
            {
                return attr.value.as_ref().map(|v| v.content.as_str());
            }
            None
        });
        if let Some(is_exp) = dynamic_is {
            ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
            ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
            ctx.push("(");
            generate_expression(ctx, is_exp);
            ctx.push(")");
        } else if let Some(name) = static_is {
            ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
            ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
            ctx.push("(\"");
            ctx.push(name);
            ctx.push("\")");
        } else {
            ctx.push("_component_component");
        }
    } else if let Some(builtin) = is_builtin_component(&el.tag) {
        ctx.use_helper(builtin);
        ctx.push(ctx.helper(builtin));
    } else if ctx.push_component_binding_tag(&el.tag) {
    } else {
        ctx.push(&to_valid_asset_identifier("component", &el.tag));
    }

    let (mut patch_flag, dynamic_props) = if is_dynamic {
        calculate_element_patch_info_skip_is(
            el,
            ctx.options.binding_metadata.as_ref(),
            ctx.cache_handlers_in_current_scope(),
        )
    } else {
        calculate_element_patch_info(
            el,
            ctx.options.binding_metadata.as_ref(),
            ctx.cache_handlers_in_current_scope(),
        )
    };

    if has_slot_children(el)
        && let Some(flag) = patch_flag
    {
        let new_flag = flag & !1;
        patch_flag = if new_flag > 0 { Some(new_flag) } else { None };
    }

    if el.tag == "KeepAlive"
        || el.tag == "keep-alive"
        || has_dynamic_slots_flag(el)
        || (ctx.has_slot_params() && has_forwarded_slot_outlet(el))
    {
        patch_flag = Some(patch_flag.unwrap_or(0) | 1024);
    }

    let has_patch_info = patch_flag.is_some() || dynamic_props.is_some();

    // Extract static class/style for merging with dynamic bindings
    let static_merge = extract_static_class_style(el);
    let has_dyn_class = has_dynamic_class(el);
    let has_dyn_style = has_dynamic_style(el);

    // Check if component has v-bind spread or v-on spread
    let has_vbind = has_vbind_spread(el);
    let has_von = has_von_spread(el);
    if has_vbind || has_von {
        ctx.use_helper(RuntimeHelper::MergeProps);
        ctx.push(", ");
        ctx.push(ctx.helper(RuntimeHelper::MergeProps));
        ctx.push("(");

        let mut first_merge_arg = true;
        // Add v-bind spreads
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

        // Add v-on spreads wrapped with _toHandlers
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
            is_dynamic,
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
            is_dynamic,
        );
    }

    ctx.skip_scope_id = prev_skip_scope_id;

    // Generate children/slots for v-if branch component (same pattern as element.rs)
    if has_slot_children(el) {
        ctx.push(", ");
        generate_slots(ctx, el);
    } else if el.children.iter().any(|c| !is_whitespace_or_comment(c)) {
        // Teleport passes children as an array, not a slot object.
        ctx.push(", [");
        for (i, (op, node)) in RenduChildren::new(&el.children).rendered().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            dispatch_rendu_op(ctx, op, node);
        }
        ctx.push("]");
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
    if has_vshow {
        generate_vshow_closing(ctx, el);
    }
}
