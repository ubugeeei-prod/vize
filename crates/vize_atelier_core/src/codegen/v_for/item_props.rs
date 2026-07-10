//! v-for item prop object generation.
//!
//! Builds the prop object for a v-for item element, delegating the merged
//! (`_mergeProps`) and single-prop cases to `item_props_merge`. Split out of
//! `v_for/generate` to keep that file focused on item/block generation.

use crate::{ElementNode, ExpressionNode};
use vize_rendu::RenduOp;

use super::super::{
    context::CodegenContext, element::helpers::is_is_prop, expression::generate_expression,
};
use super::helpers::{has_other_props, should_skip_prop};
use super::item_props_merge::{generate_for_item_props_merged, generate_single_prop};

/// Generate props for v-for item, including key and all other props
pub(crate) fn generate_for_item_props(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    key_exp: Option<&ExpressionNode<'_>>,
    skip_is_prop: bool,
) {
    let has_other = if skip_is_prop {
        el.props
            .iter()
            .any(|prop| !is_is_prop(prop) && !should_skip_prop(prop))
    } else {
        has_other_props(el)
    };
    // skip_scope_id suppresses duplicate scope attrs for synthetic prop objects.
    let scope_id = if ctx.skip_scope_id {
        None
    } else {
        ctx.options.scope_id.clone()
    };

    if key_exp.is_none() && !has_other && scope_id.is_none() {
        ctx.push(", null");
        return;
    }

    ctx.push(", ");

    if !has_other {
        // Only key (and optionally scope_id), no other props
        if let Some(key) = key_exp {
            ctx.push("{ key: ");
            generate_expression(ctx, key);
            if let Some(sid) = scope_id {
                ctx.push(", \"");
                ctx.push(sid.as_str());
                ctx.push("\": \"\"");
            }
            ctx.push(" }");
        } else if let Some(sid) = scope_id {
            // No key, no other props, but has scope_id
            ctx.push("{ \"");
            ctx.push(sid.as_str());
            ctx.push("\": \"\" }");
        }
        return;
    }

    // Check for v-bind/v-on object spreads (v-bind="obj", v-on="handlers")
    let has_vbind_spread = super::super::props::has_vbind_object(&el.props);
    let has_von_spread = super::super::props::has_von_object(&el.props);

    if has_vbind_spread || has_von_spread {
        generate_for_item_props_merged(
            ctx,
            el,
            key_exp,
            &scope_id,
            has_vbind_spread,
            has_von_spread,
            skip_is_prop,
        );
        return;
    }

    // Detect static class/style that need to be merged with dynamic :class/:style
    let static_class = el
        .props
        .iter()
        .find_map(|prop| match RenduOp::from_prop(prop) {
            RenduOp::Attribute {
                name: "class",
                value,
                ..
            } => value,
            _ => None,
        });

    let static_style = el
        .props
        .iter()
        .find_map(|prop| match RenduOp::from_prop(prop) {
            RenduOp::Attribute {
                name: "style",
                value,
                ..
            } => value,
            _ => None,
        });

    let has_dynamic_class = el.props.iter().any(|prop| {
        matches!(
            RenduOp::from_prop(prop),
            RenduOp::Directive { name: "bind", arg: Some(arg), .. }
                if arg.is_simple("class")
        )
    });

    let has_dynamic_style = el.props.iter().any(|prop| {
        matches!(
            RenduOp::from_prop(prop),
            RenduOp::Directive { name: "bind", arg: Some(arg), .. }
                if arg.is_simple("style")
        )
    });

    let skip_static_class = static_class.is_some() && has_dynamic_class;
    let skip_static_style = static_style.is_some() && has_dynamic_style;

    // Static class/style are only merged into the dynamic binding's array when a
    // dynamic counterpart exists; source ordering is preserved via StaticMerge.
    let full_merge = super::super::props::StaticMerge::from_props(&el.props);
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

    if let Some(key) = key_exp {
        // Merge key with other props
        ctx.push("{");
        ctx.indent();
        ctx.newline();
        ctx.push("key: ");
        generate_expression(ctx, key);

        for prop in el.props.iter() {
            if should_skip_prop(prop) || (skip_is_prop && is_is_prop(prop)) {
                continue;
            }
            if skip_static_class
                && matches!(
                    RenduOp::from_prop(prop),
                    RenduOp::Attribute { name: "class", .. }
                )
            {
                continue;
            }
            if skip_static_style
                && matches!(
                    RenduOp::from_prop(prop),
                    RenduOp::Attribute { name: "style", .. }
                )
            {
                continue;
            }
            ctx.push(",");
            ctx.newline();
            generate_single_prop(ctx, prop, merge_static);
        }

        if let Some(sid) = scope_id {
            ctx.push(",");
            ctx.newline();
            ctx.push("\"");
            ctx.push(sid.as_str());
            ctx.push("\": \"\"");
        }

        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    } else {
        // No key, generate props directly (skipping v-for directive)
        ctx.push("{");
        let mut first = true;
        for prop in el.props.iter() {
            if should_skip_prop(prop) || (skip_is_prop && is_is_prop(prop)) {
                continue;
            }
            if skip_static_class
                && matches!(
                    RenduOp::from_prop(prop),
                    RenduOp::Attribute { name: "class", .. }
                )
            {
                continue;
            }
            if skip_static_style
                && matches!(
                    RenduOp::from_prop(prop),
                    RenduOp::Attribute { name: "style", .. }
                )
            {
                continue;
            }
            if !first {
                ctx.push(",");
            }
            ctx.push(" ");
            generate_single_prop(ctx, prop, merge_static);
            first = false;
        }

        if let Some(sid) = scope_id {
            if !first {
                ctx.push(",");
            }
            ctx.push(" \"");
            ctx.push(sid.as_str());
            ctx.push("\": \"\"");
        }

        ctx.push(" }");
    }
}
