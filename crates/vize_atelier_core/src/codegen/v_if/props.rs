//! Source-ordered props generation for v-if branches.

use crate::{ElementNode, ExpressionNode, IfBranchNode, PropNode, RuntimeHelper};
use vize_s0::{FxHashSet, String};

use super::{
    super::{
        context::CodegenContext,
        element::helpers::is_is_prop,
        expression::generate_expression,
        props::{
            StaticMerge, duplicate_von_event_keys, generate_merged_event_handlers,
            get_von_event_key, is_supported_directive,
        },
    },
    generate::{
        generate_single_prop_for_if, is_vbind_spread_prop, is_von_spread_prop,
        should_skip_prop_for_if,
    },
    generate_if_branch_key,
};

pub(super) fn generate_if_branch_props(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
    skip_is_prop: bool,
) {
    let scope_id = if ctx.skip_scope_id {
        None
    } else {
        ctx.options.scope_id.clone()
    };
    let has_spread = el.props.iter().any(is_usable_spread);

    if !has_spread {
        generate_segment_object(
            ctx,
            &el.props,
            branch,
            branch_index,
            true,
            scope_id.as_deref(),
            skip_is_prop,
            false,
        );
        return;
    }

    let normalize_keyed_bind_spread = branch.user_key.is_none()
        && scope_id.is_none()
        && only_key_and_bind_spread(&el.props, skip_is_prop);
    if normalize_keyed_bind_spread {
        ctx.use_helper(RuntimeHelper::NormalizeProps);
        ctx.use_helper(RuntimeHelper::GuardReactiveProps);
        ctx.push(ctx.helper(RuntimeHelper::NormalizeProps));
        ctx.push("(");
    }
    ctx.use_helper(RuntimeHelper::MergeProps);
    ctx.push(ctx.helper(RuntimeHelper::MergeProps));
    ctx.push("(");

    let mut first_arg = true;
    let mut key_pending = true;
    let mut segment_start = 0;

    for (index, prop) in el.props.iter().enumerate() {
        let Some(dir) = usable_spread(prop) else {
            continue;
        };
        let Some(expression) = &dir.exp else { continue };

        flush_segment(
            ctx,
            &el.props[segment_start..index],
            branch,
            branch_index,
            key_pending,
            None,
            skip_is_prop,
            &mut first_arg,
        );
        key_pending = false;
        segment_start = index + 1;

        push_separator(ctx, &mut first_arg);
        if dir.name == "bind" {
            generate_expression(ctx, expression);
        } else {
            ctx.use_helper(RuntimeHelper::ToHandlers);
            ctx.push(ctx.helper(RuntimeHelper::ToHandlers));
            ctx.push("(");
            generate_expression(ctx, expression);
            ctx.push(", true)");
        }
    }

    flush_segment(
        ctx,
        &el.props[segment_start..],
        branch,
        branch_index,
        key_pending,
        scope_id.as_deref(),
        skip_is_prop,
        &mut first_arg,
    );
    ctx.push(")");
    if normalize_keyed_bind_spread {
        ctx.push(")");
    }
}

#[allow(clippy::too_many_arguments)]
fn flush_segment(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    branch: &IfBranchNode<'_>,
    branch_index: usize,
    include_key: bool,
    scope_id: Option<&str>,
    skip_is_prop: bool,
    first_arg: &mut bool,
) {
    if !include_key && scope_id.is_none() && !has_renderable_prop(props, skip_is_prop) {
        return;
    }
    push_separator(ctx, first_arg);
    generate_segment_object(
        ctx,
        props,
        branch,
        branch_index,
        include_key,
        scope_id,
        skip_is_prop,
        true,
    );
}

fn push_separator(ctx: &mut CodegenContext, first_arg: &mut bool) {
    if !*first_arg {
        ctx.push(", ");
    }
    *first_arg = false;
}

#[allow(clippy::too_many_arguments)]
fn generate_segment_object(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    branch: &IfBranchNode<'_>,
    branch_index: usize,
    include_key: bool,
    scope_id: Option<&str>,
    skip_is_prop: bool,
    inside_merge_props: bool,
) {
    let has_dynamic_class = has_dynamic_binding(props, "class");
    let has_dynamic_style = has_dynamic_binding(props, "style");
    let has_other = props
        .iter()
        .any(|prop| is_renderable_prop(prop, skip_is_prop, has_dynamic_class, has_dynamic_style));

    if include_key && !has_other && scope_id.is_none() {
        ctx.push("{ key: ");
        generate_if_branch_key(ctx, branch, branch_index);
        ctx.push(" }");
        return;
    }

    let static_merge = StaticMerge::from_props(props);
    let duplicate_events = if ctx.merge_props {
        duplicate_von_event_keys(props, ctx.props_is_plain_element)
    } else {
        FxHashSet::default()
    };
    let mut emitted_events: FxHashSet<String> = FxHashSet::default();
    let previous_skip_normalize = ctx.skip_normalize;
    if inside_merge_props {
        ctx.skip_normalize = true;
    }

    ctx.push("{");
    ctx.indent();
    let mut first_prop = true;
    if include_key {
        ctx.newline();
        ctx.push("key: ");
        generate_if_branch_key(ctx, branch, branch_index);
        first_prop = false;
    }

    for prop in props {
        if !is_renderable_prop(prop, skip_is_prop, has_dynamic_class, has_dynamic_style) {
            continue;
        }
        let merged_event_key = if let PropNode::Directive(dir) = prop
            && let Some(key) = get_von_event_key(dir, ctx.props_is_plain_element)
            && duplicate_events.contains(&key)
        {
            if !emitted_events.insert(key.clone()) {
                continue;
            }
            Some(key)
        } else {
            None
        };
        if !first_prop {
            ctx.push(",");
        }
        ctx.newline();
        if let Some(key) = merged_event_key {
            generate_merged_event_handlers(ctx, props, &key);
        } else {
            generate_single_prop_for_if(ctx, prop, static_merge);
        }
        first_prop = false;
    }

    if let Some(scope_id) = scope_id {
        if !first_prop {
            ctx.push(",");
        }
        ctx.newline();
        ctx.push("\"");
        ctx.push(scope_id);
        ctx.push("\": \"\"");
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("}");
    ctx.skip_normalize = previous_skip_normalize;
}

fn has_renderable_prop(props: &[PropNode<'_>], skip_is_prop: bool) -> bool {
    let has_dynamic_class = has_dynamic_binding(props, "class");
    let has_dynamic_style = has_dynamic_binding(props, "style");
    props
        .iter()
        .any(|prop| is_renderable_prop(prop, skip_is_prop, has_dynamic_class, has_dynamic_style))
}

fn is_renderable_prop(
    prop: &PropNode<'_>,
    skip_is_prop: bool,
    has_dynamic_class: bool,
    has_dynamic_style: bool,
) -> bool {
    if let PropNode::Directive(dir) = prop
        && !is_supported_directive(dir)
    {
        return false;
    }
    if skip_is_prop && is_is_prop(prop) {
        return false;
    }
    !should_skip_prop_for_if(prop, has_dynamic_class, has_dynamic_style)
        && !is_vbind_spread_prop(prop)
        && !is_von_spread_prop(prop)
}

fn has_dynamic_binding(props: &[PropNode<'_>], name: &str) -> bool {
    props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.content == name)
        )
    })
}

fn is_usable_spread(prop: &PropNode<'_>) -> bool {
    usable_spread(prop).is_some()
}

fn only_key_and_bind_spread(props: &[PropNode<'_>], skip_is_prop: bool) -> bool {
    let has_one_bind = props
        .iter()
        .filter(|prop| {
            matches!(
                usable_spread(prop),
                Some(dir) if dir.name == "bind" && dir.exp.is_some()
            )
        })
        .count()
        == 1;
    has_one_bind
        && !props.iter().any(|prop| {
            matches!(
                usable_spread(prop),
                Some(dir) if dir.name == "on" && dir.exp.is_some()
            ) || has_renderable_prop(core::slice::from_ref(prop), skip_is_prop)
        })
}

fn usable_spread<'a, 'b>(prop: &'a PropNode<'b>) -> Option<&'a crate::DirectiveNode<'b>> {
    let PropNode::Directive(dir) = prop else {
        return None;
    };
    if dir.exp.is_some() && (is_vbind_spread_prop(prop) || is_von_spread_prop(prop)) {
        Some(dir)
    } else {
        None
    }
}
