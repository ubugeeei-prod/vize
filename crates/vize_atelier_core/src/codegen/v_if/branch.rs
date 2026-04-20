//! v-if branch generation.
//!
//! Generates code for individual v-if/v-else-if/v-else branches including
//! component, element, template fragment, and regular fragment rendering.

use crate::ast::{
    ElementNode, ElementType, ExpressionNode, IfBranchNode, PropNode, RuntimeHelper,
    TemplateChildNode,
};
use vize_carton::ToCompactString;

use super::{
    super::{
        children::{generate_children_force_array, is_directive_comment},
        context::CodegenContext,
        element::is_whitespace_or_comment,
        element::{
            generate_custom_directives_closing, generate_vmodel_closing, generate_vshow_closing,
            has_custom_directives, has_vmodel_directive, has_vshow_directive,
        },
        expression::generate_expression,
        helpers::{escape_js_string, is_builtin_component, to_valid_asset_identifier},
        node::generate_node,
        patch_flag::{
            calculate_element_patch_info, calculate_element_patch_info_skip_is, patch_flag_name,
        },
        slots::{
            generate_slot_outlet_name, generate_slot_outlet_props_entries, generate_slots,
            has_dynamic_slots_flag, has_slot_children, has_slot_outlet_props,
        },
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
                if el.tag_type == ElementType::Template {
                    // Template with single child -> unwrap to single element
                    if el.children.len() == 1 {
                        if let TemplateChildNode::Element(inner) = &el.children[0] {
                            // Check if inner element is a component
                            if inner.tag_type == ElementType::Component {
                                generate_if_branch_component(ctx, inner, branch, branch_index);
                            } else if inner.tag_type == ElementType::Slot {
                                generate_if_branch_slot(ctx, inner, branch, branch_index);
                            } else {
                                generate_if_branch_element(ctx, inner, branch, branch_index);
                            }
                            return;
                        }
                    }
                    // Template with multiple children -> fragment
                    generate_if_branch_template_fragment(ctx, &el.children, branch, branch_index);
                } else if el.tag_type == ElementType::Component {
                    // Component
                    generate_if_branch_component(ctx, el, branch, branch_index);
                } else if el.tag_type == ElementType::Slot {
                    generate_if_branch_slot(ctx, el, branch, branch_index);
                } else {
                    // Regular element
                    generate_if_branch_element(ctx, el, branch, branch_index);
                }
            }
            _ => {
                // Other node types - wrap in fragment
                generate_if_branch_fragment(ctx, branch, branch_index);
            }
        }
    } else {
        // Multiple children - wrap in fragment
        generate_if_branch_fragment(ctx, branch, branch_index);
    }
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
    ctx.push(", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    if has_slot_outlet_props(el) {
        ctx.push(", ");
        generate_slot_outlet_props_entries(ctx, el);
    }
    ctx.push("}");

    if !el.children.is_empty() {
        ctx.push(", () => [");
        let filtered: Vec<_> = el
            .children
            .iter()
            .filter(|c| !is_directive_comment(c))
            .collect();
        for (i, child) in filtered.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            generate_node(ctx, child);
        }
        ctx.push("]");
    }
    ctx.push(")");
}

/// Generate component for if branch.
fn generate_if_branch_component(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    let is_dynamic_component = el.tag == "component" || el.tag == "Component";
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

    // Components: skip scope_id in props -- Vue runtime applies it via __scopeId
    let prev_skip_scope_id = ctx.skip_scope_id;
    ctx.skip_scope_id = true;
    ctx.use_helper(RuntimeHelper::CreateBlock);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateBlock));
    ctx.push("(");
    // Generate component name
    // Handle dynamic component (<component :is="..."> / <Component :is="...">)
    if is_dynamic_component {
        let dynamic_is = el.props.iter().find_map(|p| {
            if let PropNode::Directive(dir) = p {
                if dir.name == "bind" {
                    if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                        if arg.content == "is" {
                            return dir.exp.as_ref();
                        }
                    }
                }
            }
            None
        });
        let static_is = el.props.iter().find_map(|p| {
            if let PropNode::Attribute(attr) = p {
                if attr.name == "is" {
                    return attr.value.as_ref().map(|v| v.content.as_str());
                }
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
    } else if ctx.is_component_in_bindings(&el.tag) {
        // In inline mode, components are directly in scope (imported at module level)
        // In function mode, use $setup.ComponentName to access setup bindings
        if !ctx.options.inline {
            ctx.push("$setup.");
        }
        ctx.push(el.tag.as_str());
    } else {
        ctx.push(&to_valid_asset_identifier("component", &el.tag));
    }

    let (mut patch_flag, dynamic_props) = if is_dynamic_component {
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

    if has_slot_children(el) {
        if let Some(flag) = patch_flag {
            let new_flag = flag & !1;
            patch_flag = if new_flag > 0 { Some(new_flag) } else { None };
        }
    }

    if el.tag == "KeepAlive" || el.tag == "keep-alive" || has_dynamic_slots_flag(el) {
        patch_flag = Some(patch_flag.unwrap_or(0) | 1024);
    }

    let has_patch_info = patch_flag.is_some() || dynamic_props.is_some();

    // Extract static class/style for merging with dynamic bindings
    let (static_class, static_style) = extract_static_class_style(el);
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
            if let PropNode::Directive(dir) = prop {
                if dir.name == "bind" && dir.arg.is_none() {
                    if let Some(exp) = &dir.exp {
                        if !first_merge_arg {
                            ctx.push(", ");
                        }
                        generate_expression(ctx, exp);
                        first_merge_arg = false;
                    }
                }
            }
        }

        // Add v-on spreads wrapped with _toHandlers
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop {
                if dir.name == "on" && dir.arg.is_none() {
                    if let Some(exp) = &dir.exp {
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
            static_class,
            static_style,
            has_dyn_class,
            has_dyn_style,
        );
        ctx.push(")");
    } else {
        ctx.push(", ");
        generate_if_branch_props_object(
            ctx,
            el,
            branch,
            branch_index,
            static_class,
            static_style,
            has_dyn_class,
            has_dyn_style,
        );
    }

    ctx.skip_scope_id = prev_skip_scope_id;

    // Generate children/slots for v-if branch component (same pattern as element.rs)
    if has_slot_children(el) {
        ctx.push(", ");
        generate_slots(ctx, el);
    } else if el.children.iter().any(|c| !is_whitespace_or_comment(c)) {
        // Teleport/KeepAlive: pass children as array, not slot object
        ctx.push(", [");
        let filtered: Vec<_> = el
            .children
            .iter()
            .filter(|c| !is_directive_comment(c))
            .collect();
        for (i, child) in filtered.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            generate_node(ctx, child);
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
    let (static_class, static_style) = extract_static_class_style(el);
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
            if let PropNode::Directive(dir) = prop {
                if dir.name == "bind" && dir.arg.is_none() {
                    if let Some(exp) = &dir.exp {
                        if !first_merge_arg {
                            ctx.push(", ");
                        }
                        generate_expression(ctx, exp);
                        first_merge_arg = false;
                    }
                }
            }
        }

        // Add all v-on spreads wrapped with _toHandlers
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop {
                if dir.name == "on" && dir.arg.is_none() {
                    if let Some(exp) = &dir.exp {
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
            static_class,
            static_style,
            has_dyn_class,
            has_dyn_style,
        );
        ctx.push(")");
    } else {
        ctx.push(", ");
        generate_if_branch_props_object(
            ctx,
            el,
            branch,
            branch_index,
            static_class,
            static_style,
            has_dyn_class,
            has_dyn_style,
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
                generate_if_branch_children(ctx, &el.children);
            }
        } else {
            generate_if_branch_children(ctx, &el.children);
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

/// Generate template fragment for if branch (multiple children from template).
fn generate_if_branch_template_fragment(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.use_helper(RuntimeHelper::Fragment);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::Fragment));
    ctx.push(", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    ctx.push(" }, ");
    generate_children_force_array(ctx, children);
    ctx.push(", 64 /* STABLE_FRAGMENT */))");
}

/// Generate fragment wrapper for if branch with multiple children.
fn generate_if_branch_fragment(
    ctx: &mut CodegenContext,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.use_helper(RuntimeHelper::Fragment);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::Fragment));
    ctx.push(", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    ctx.push(" }, ");
    generate_children_force_array(ctx, &branch.children);
    ctx.push(", 64 /* STABLE_FRAGMENT */))");
}

/// Generate children for if branch element.
fn generate_if_branch_children(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    if children.is_empty() {
        return;
    }

    // Check if all children are simple (text or interpolation)
    let has_only_text_or_interpolation = children.iter().all(|c| {
        matches!(
            c,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });

    if has_only_text_or_interpolation {
        // Use string concatenation for text/interpolation mix
        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                ctx.push(" + ");
            }
            match child {
                TemplateChildNode::Interpolation(interp) => {
                    ctx.use_helper(RuntimeHelper::ToDisplayString);
                    ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
                    ctx.push("(");
                    generate_expression(ctx, &interp.content);
                    ctx.push(")");
                }
                TemplateChildNode::Text(text) => {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(text.content.as_str()));
                    ctx.push("\"");
                }
                _ => {}
            }
        }
    } else {
        // Complex branch children need the same text/interpolation grouping as
        // regular element children to avoid raw string array entries.
        generate_children_force_array(ctx, children);
    }
}
