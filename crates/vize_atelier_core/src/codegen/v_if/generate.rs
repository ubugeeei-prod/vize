//! Props and helper utilities for v-if branch code generation.
//!
//! Contains single-prop generation, event key deduplication, and spread detection.

use crate::{ExpressionNode, PropNode};

use super::super::{
    context::CodegenContext,
    helpers::{escape_js_string, is_valid_js_identifier},
    props::{StaticMerge, generate_directive_prop_with_static},
};

/// Check if prop should be skipped for v-if branch element.
pub(super) fn should_skip_prop_for_if(
    p: &PropNode<'_>,
    has_dynamic_class: bool,
    has_dynamic_style: bool,
) -> bool {
    match p {
        PropNode::Attribute(attr) => {
            // Skip static class if there's a dynamic :class (will be merged)
            if attr.name == "class" && has_dynamic_class {
                return true;
            }
            // Skip static style if there's a dynamic :style (will be merged)
            if attr.name == "style" && has_dynamic_style {
                return true;
            }
            false
        }
        PropNode::Directive(dir) => {
            if dir.name == "bind"
                && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                && arg.content == "key"
            {
                return true;
            }
            // Skip v-if/v-else-if/v-else directives
            if matches!(dir.name.as_str(), "if" | "else-if" | "else") {
                return true;
            }
            false
        }
    }
}

/// Generate a single prop for v-if branch element.
pub(super) fn generate_single_prop_for_if(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_merge: StaticMerge<'_>,
) {
    match prop {
        PropNode::Attribute(attr) => {
            let ref_value = if attr.name == "ref" && ctx.options.inline {
                attr.value.as_ref()
            } else {
                None
            };
            let ref_binding_type = ref_value.and_then(|v| {
                ctx.options
                    .binding_metadata
                    .as_ref()
                    .and_then(|m| m.bindings.get(v.content.as_str()).copied())
            });
            let should_ref_runtime_binding = matches!(
                ref_binding_type,
                Some(
                    crate::options::BindingType::SetupLet
                        | crate::options::BindingType::SetupRef
                        | crate::options::BindingType::SetupMaybeRef
                )
            );
            let needs_ref_for = attr.name == "ref" && ctx.in_v_for;

            if let (true, Some(ref_value)) = (should_ref_runtime_binding, ref_value) {
                let ref_name = &ref_value.content;
                if needs_ref_for {
                    ctx.push("ref_for: true, ");
                }
                ctx.push("ref_key: \"");
                ctx.push(ref_name);
                ctx.push("\", ref: ");
                ctx.push(ref_name);
                return;
            }

            if needs_ref_for {
                ctx.push("ref_for: true, ");
            }
            let needs_quotes = !is_valid_js_identifier(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
            if let Some(value) = &attr.value {
                if should_ref_runtime_binding {
                    ctx.push(&value.content);
                } else {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(value.content.as_str()));
                    ctx.push("\"");
                }
            } else {
                ctx.push("\"\"");
            }
        }
        PropNode::Directive(dir) => {
            generate_directive_prop_with_static(ctx, dir, static_merge);
        }
    }
}

/// Check if prop is a v-bind object spread (`v-bind="obj"`).
pub(super) fn is_vbind_spread_prop(prop: &PropNode<'_>) -> bool {
    if let PropNode::Directive(dir) = prop {
        return dir.name == "bind" && dir.arg.is_none();
    }
    false
}

/// Check if prop is a v-on object spread (`v-on="obj"`).
pub(super) fn is_von_spread_prop(prop: &PropNode<'_>) -> bool {
    if let PropNode::Directive(dir) = prop {
        return dir.name == "on" && dir.arg.is_none();
    }
    false
}
