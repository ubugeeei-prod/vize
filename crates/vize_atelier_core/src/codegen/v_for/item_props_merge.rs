//! v-for item prop emission: merged-props (`_mergeProps`) and single-prop.
//!
//! Split out of `v_for/generate` to keep that file focused on item/block
//! generation. `generate_for_item_props` (still in `generate`) calls these.

use crate::{ElementNode, ExpressionNode, PropNode, RuntimeHelper};

use super::super::{
    context::CodegenContext, element::helpers::is_is_prop, expression::generate_expression,
    helpers::escape_js_string,
};
use super::helpers::should_skip_prop;

/// Generate props using _mergeProps when v-bind/v-on object spreads are present.
pub(super) fn generate_for_item_props_merged(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    key_exp: Option<&ExpressionNode<'_>>,
    scope_id: &Option<vize_carton::String>,
    has_vbind_spread: bool,
    has_von_spread: bool,
    skip_is_prop: bool,
) {
    ctx.use_helper(RuntimeHelper::MergeProps);
    ctx.push(ctx.helper(RuntimeHelper::MergeProps));
    ctx.push("(");

    let mut first_merge_arg = true;

    if has_vbind_spread {
        super::super::props::generate_vbind_object_exp(ctx, &el.props);
        first_merge_arg = false;
    }

    if has_von_spread {
        if !first_merge_arg {
            ctx.push(", ");
        }
        super::super::props::generate_von_object_exp(ctx, &el.props);
        first_merge_arg = false;
    }

    let has_remaining = key_exp.is_some()
        || scope_id.is_some()
        || el.props.iter().any(|p| {
            if should_skip_prop(p) || (skip_is_prop && is_is_prop(p)) {
                return false;
            }
            if let PropNode::Directive(dir) = p
                && dir.arg.is_none()
                && (dir.name == "bind" || dir.name == "on")
            {
                return false;
            }
            true
        });

    if has_remaining {
        if !first_merge_arg {
            ctx.push(", ");
        }
        ctx.push("{");
        ctx.indent();
        let mut first_prop = true;

        if let Some(key) = key_exp {
            ctx.newline();
            ctx.push("key: ");
            generate_expression(ctx, key);
            first_prop = false;
        }

        for prop in el.props.iter() {
            if should_skip_prop(prop) || (skip_is_prop && is_is_prop(prop)) {
                continue;
            }
            if let PropNode::Directive(dir) = prop
                && dir.arg.is_none()
                && (dir.name == "bind" || dir.name == "on")
            {
                continue;
            }
            if !first_prop {
                ctx.push(",");
            }
            ctx.newline();
            generate_single_prop(ctx, prop, super::super::props::StaticMerge::default());
            first_prop = false;
        }

        if let Some(sid) = scope_id {
            if !first_prop {
                ctx.push(",");
            }
            ctx.newline();
            ctx.push("\"");
            ctx.push(sid.as_str());
            ctx.push("\": \"\"");
        }

        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    }

    ctx.push(")");
}

/// Generate a single prop (attribute or directive)
pub(super) fn generate_single_prop(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_merge: super::super::props::StaticMerge<'_>,
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
            let needs_quotes = !super::super::helpers::is_valid_js_identifier(&attr.name);
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
                    ctx.push(&escape_js_string(&value.content));
                    ctx.push("\"");
                }
            } else {
                ctx.push("\"\"");
            }
        }
        PropNode::Directive(dir) => {
            super::super::props::generate_directive_prop_with_static(ctx, dir, static_merge);
        }
    }
}
