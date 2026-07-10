//! v-for item prop emission: merged-props (`_mergeProps`) and single-prop.
//!
//! Split out of `v_for/generate` to keep that file focused on item/block
//! generation. `generate_for_item_props` (still in `generate`) calls these.

use crate::{ElementNode, ExpressionNode, PropNode, RuntimeHelper, rendu::RenduOp};

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

    let mut extra_pending = key_exp.is_some() || scope_id.is_some();
    let mut seg_start = 0usize;

    for (index, prop) in el.props.iter().enumerate() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        let is_vbind_spread = has_vbind_spread && dir.name == "bind" && dir.arg.is_none();
        let is_von_spread = has_von_spread && dir.name == "on" && dir.arg.is_none();
        if !is_vbind_spread && !is_von_spread {
            continue;
        }

        flush_for_item_props_segment(
            ctx,
            &el.props[seg_start..index],
            key_exp,
            scope_id,
            extra_pending,
            &mut first_merge_arg,
            skip_is_prop,
        );
        extra_pending = false;
        seg_start = index + 1;

        if !first_merge_arg {
            ctx.push(", ");
        }
        first_merge_arg = false;
        if is_vbind_spread {
            if let Some(exp) = &dir.exp {
                generate_expression(ctx, exp);
            }
        } else {
            super::super::props::generate_von_object_exp(ctx, &el.props[index..=index]);
        }
    }

    flush_for_item_props_segment(
        ctx,
        &el.props[seg_start..],
        key_exp,
        scope_id,
        extra_pending,
        &mut first_merge_arg,
        skip_is_prop,
    );

    ctx.push(")");
}

fn flush_for_item_props_segment(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    key_exp: Option<&ExpressionNode<'_>>,
    scope_id: &Option<vize_carton::String>,
    include_extra: bool,
    first_merge_arg: &mut bool,
    skip_is_prop: bool,
) {
    let has_props = props
        .iter()
        .any(|prop| !is_for_item_segment_skip_prop(prop, skip_is_prop));
    if !has_props && !include_extra {
        return;
    }

    if !*first_merge_arg {
        ctx.push(", ");
    }
    *first_merge_arg = false;

    let skip_static_class = props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Attribute(attr) if attr.name == "class"
        )
    }) && props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(exp)) if exp.content == "class")
        )
    });
    let skip_static_style = props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Attribute(attr) if attr.name == "style"
        )
    }) && props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(exp)) if exp.content == "style")
        )
    });

    let full_merge = super::super::props::StaticMerge::from_props(props);
    let merge_static = super::super::props::StaticMerge {
        class: if skip_static_class {
            full_merge.class
        } else {
            None
        },
        class_before: full_merge.class_before,
        style: if skip_static_style {
            full_merge.style
        } else {
            None
        },
        style_before: full_merge.style_before,
    };

    ctx.push("{");
    ctx.indent();
    let mut first_prop = true;
    let prev_skip_normalize = ctx.skip_normalize;
    ctx.skip_normalize = true;

    if include_extra && let Some(key) = key_exp {
        ctx.newline();
        ctx.push("key: ");
        generate_expression(ctx, key);
        first_prop = false;
    }

    for prop in props {
        if is_for_item_segment_skip_prop(prop, skip_is_prop) {
            continue;
        }
        if skip_static_class
            && let PropNode::Attribute(attr) = prop
            && attr.name == "class"
        {
            continue;
        }
        if skip_static_style
            && let PropNode::Attribute(attr) = prop
            && attr.name == "style"
        {
            continue;
        }
        if !first_prop {
            ctx.push(",");
        }
        ctx.newline();
        generate_single_prop(ctx, prop, merge_static);
        first_prop = false;
    }

    if include_extra && let Some(sid) = scope_id {
        if !first_prop {
            ctx.push(",");
        }
        ctx.newline();
        ctx.push("\"");
        ctx.push(sid.as_str());
        ctx.push("\": \"\"");
    }

    ctx.skip_normalize = prev_skip_normalize;
    ctx.deindent();
    ctx.newline();
    ctx.push("}");
}

fn is_for_item_segment_skip_prop(prop: &PropNode<'_>, skip_is_prop: bool) -> bool {
    if should_skip_prop(prop) || (skip_is_prop && is_is_prop(prop)) {
        return true;
    }
    matches!(
        prop,
        PropNode::Directive(dir)
            if dir.arg.is_none() && (dir.name == "bind" || dir.name == "on")
    )
}

/// Generate a single prop (attribute or directive)
pub(super) fn generate_single_prop(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_merge: super::super::props::StaticMerge<'_>,
) {
    match RenduOp::from_prop(prop) {
        RenduOp::Attribute {
            name,
            name_span,
            value,
            value_span,
            ..
        } => {
            let PropNode::Attribute(attr) = prop else {
                unreachable!("Rendu attribute must borrow an attribute prop");
            };
            let ref_value = if name == "ref" && ctx.options.inline {
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
            let needs_ref_for = name == "ref" && ctx.in_v_for;

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
            let needs_quotes = !super::super::helpers::is_valid_js_identifier(name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.record_mapping_named(&name_span.start, name);
            ctx.push(name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
            if let Some(value) = value {
                if should_ref_runtime_binding {
                    ctx.push(value);
                } else {
                    ctx.push("\"");
                    if let Some(span) = value_span {
                        ctx.record_mapping(&span.start);
                    }
                    ctx.push(&escape_js_string(value));
                    ctx.push("\"");
                }
            } else {
                ctx.push("\"\"");
            }
        }
        RenduOp::Directive { .. } => {
            let PropNode::Directive(dir) = prop else {
                unreachable!("Rendu directive must borrow a directive prop");
            };
            super::super::props::generate_directive_prop_with_static(ctx, dir, static_merge);
        }
        _ => unreachable!("element props lower to attribute or directive Rendu ops"),
    }
}
