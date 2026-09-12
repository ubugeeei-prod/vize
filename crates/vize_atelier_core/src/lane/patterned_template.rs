//! Experimental patterned template desugaring (`v-match` / `v-when`).

mod directives;
mod lowering;
mod syntax;

use vize_s0::{Allocator, Box, String, Vec, cstr, ensure_sufficient_stack};

use crate::{
    DirectiveNode, ElementNode, ElementType, ErrorCode, PropNode, RootNode, SourceLocation,
    TemplateChildNode, TransformContext,
};

use self::directives::{create_directive, rewrite_case_directive};
use self::lowering::{
    build_binding_for_expression, build_case_pattern, build_guard_condition, guard_suffix,
    is_wildcard_pattern, pattern_condition_with_guard, pattern_without_guard,
};
use self::syntax::expression_source;

/// Rewrite experimental `v-match` / `v-when` syntax into the existing `v-if`
/// structural directive chain.
pub fn desugar_patterned_templates<'a>(ctx: &mut TransformContext<'a>, root: &mut RootNode<'a>) {
    let allocator = ctx.allocator;
    let source = root.source;
    rewrite_children(ctx, allocator, &mut root.children, source);
    diagnose_remaining_patterned_directives(ctx, &root.children);
}

/// Report patterned-template directives when their opt-in flag is disabled.
pub fn diagnose_disabled_patterned_templates<'a>(
    ctx: &mut TransformContext<'a>,
    root: &RootNode<'a>,
) {
    diagnose_disabled_children(ctx, &root.children);
}

fn diagnose_disabled_children<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
) {
    for child in children {
        let TemplateChildNode::Element(el) = child else {
            continue;
        };
        for prop in el.props.iter() {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if matches!(dir.name, "match" | "when" | "case") {
                ctx.on_error_with_message(
                    ErrorCode::ExtendPoint,
                    "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`.",
                    Some(dir.loc.clone()),
                );
            }
        }
        ensure_sufficient_stack(|| diagnose_disabled_children(ctx, &el.children));
    }
}

fn rewrite_children<'a>(
    ctx: &mut TransformContext<'a>,
    allocator: &'a Allocator,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    source: &str,
) {
    for child in children.iter_mut() {
        let TemplateChildNode::Element(el) = child else {
            continue;
        };

        ensure_sufficient_stack(|| rewrite_children(ctx, allocator, &mut el.children, source));
        rewrite_match_element(ctx, allocator, el, source);
    }
}

fn rewrite_match_element<'a>(
    ctx: &mut TransformContext<'a>,
    allocator: &'a Allocator,
    el: &mut ElementNode<'a>,
    source: &str,
) {
    let Some((match_idx, match_expr, match_loc)) = find_match_expression(el, source) else {
        return;
    };

    let case_child_indexes: std::vec::Vec<usize> = el
        .children
        .iter()
        .enumerate()
        .filter_map(|(idx, child)| {
            let TemplateChildNode::Element(case_el) = child else {
                return None;
            };
            find_when_directive(case_el).map(|_| idx)
        })
        .collect();
    let mut has_case = false;
    let mut fallback_seen = false;
    for (case_order, child_index) in case_child_indexes.iter().copied().enumerate() {
        let TemplateChildNode::Element(case_el) = &mut el.children[child_index] else {
            continue;
        };
        if is_unconditional_fallback_branch(case_el, source) {
            if fallback_seen {
                report_pattern_error(
                    ctx,
                    &case_el.loc,
                    "`v-when=\"_\"` fallback arms must be unique within a `v-match` block.",
                );
            }
            if case_order + 1 != case_child_indexes.len() {
                report_pattern_error(
                    ctx,
                    &case_el.loc,
                    "`v-when=\"_\"` fallback arms must be the last branch in a `v-match` block.",
                );
            }
            fallback_seen = true;
        }
        if rewrite_case_element(ctx, allocator, case_el, has_case, source) {
            has_case = true;
        }
    }

    if !has_case {
        return;
    }

    el.props.remove(match_idx);
    el.props.push(create_directive(
        allocator,
        "for",
        "v-for",
        Some(cstr!("{MATCH_VALUE_IDENT} in [{match_expr}]")),
        match_loc,
    ));
    el.tag = "template";
    el.tag_type = ElementType::Template;
    el.is_self_closing = false;
}

const MATCH_VALUE_IDENT: &str = "__vize_match";

fn find_match_expression(
    el: &ElementNode<'_>,
    source: &str,
) -> Option<(usize, String, SourceLocation)> {
    for (idx, prop) in el.props.iter().enumerate() {
        if let PropNode::Directive(dir) = prop
            && dir.name == "match"
        {
            let exp = dir.exp.as_ref().map(|exp| expression_source(exp, source))?;
            return Some((idx, exp, dir.loc.clone()));
        }
    }
    None
}

fn rewrite_case_element<'a>(
    ctx: &mut TransformContext<'a>,
    allocator: &'a Allocator,
    el: &mut ElementNode<'a>,
    has_previous_branch: bool,
    source: &str,
) -> bool {
    let Some(case_idx) = find_when_directive(el) else {
        return false;
    };

    let (is_default, case_expr, case_loc) = match &el.props[case_idx] {
        PropNode::Directive(dir) => {
            diagnose_when_directive_shape(ctx, dir);
            let case_expr = dir.exp.as_ref().map(|exp| expression_source(exp, source));
            let is_default = is_unconditional_fallback_directive(dir, case_expr.as_deref());
            (is_default, case_expr, dir.loc.clone())
        }
        PropNode::Attribute(_) => return false,
    };

    if !is_default && case_expr.is_none() {
        return false;
    }

    let mut case_dir = match el.props.remove(case_idx) {
        PropNode::Directive(dir) => Box::unbox(dir),
        PropNode::Attribute(_) => return false,
    };

    let pattern = case_expr
        .as_deref()
        .map(|case_expr| build_case_pattern(ctx, MATCH_VALUE_IDENT, case_expr, &case_loc));
    let has_guard = case_expr.as_deref().and_then(guard_suffix).is_some();
    let has_condition = !is_default || has_guard || !has_previous_branch;

    let (directive_name, directive_raw_name) = if has_condition {
        if has_previous_branch {
            ("else-if", "v-else-if")
        } else {
            ("if", "v-if")
        }
    } else {
        ("else", "v-else")
    };

    let condition = match (is_default, pattern.as_ref()) {
        (true, Some(pattern)) if pattern.guard.is_some() => {
            Some(build_guard_condition(pattern, MATCH_VALUE_IDENT))
        }
        (true, _) if has_previous_branch => None,
        (true, _) => Some(String::from("true")),
        (false, Some(pattern)) => Some(pattern_condition_with_guard(pattern, MATCH_VALUE_IDENT)),
        (false, None) => None,
    };

    rewrite_case_directive(
        allocator,
        &mut case_dir,
        directive_name,
        directive_raw_name,
        condition,
    );
    el.props
        .push(PropNode::Directive(Box::new_in(case_dir, &allocator)));

    if let Some(pattern) = pattern
        && let Some(binding_expr) = build_binding_for_expression(&pattern, MATCH_VALUE_IDENT)
    {
        el.props.push(create_directive(
            allocator,
            "for",
            "v-for",
            Some(binding_expr),
            case_loc,
        ));
    }

    true
}

fn find_when_directive(el: &ElementNode<'_>) -> Option<usize> {
    el.props.iter().position(
        |prop| matches!(prop, PropNode::Directive(dir) if matches!(dir.name, "when" | "case")),
    )
}

fn diagnose_remaining_patterned_directives<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
) {
    for child in children {
        let TemplateChildNode::Element(el) = child else {
            continue;
        };
        for prop in el.props.iter() {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            match dir.name {
                "match" => ctx.on_error_with_message(
                    ErrorCode::ExtendPoint,
                    "`v-match` requires at least one direct `v-when` branch.",
                    Some(dir.loc.clone()),
                ),
                "when" | "case" => ctx.on_error_with_message(
                    ErrorCode::ExtendPoint,
                    "`v-when` branches must be direct children of a `v-match` container.",
                    Some(dir.loc.clone()),
                ),
                _ => {}
            }
        }
        ensure_sufficient_stack(|| diagnose_remaining_patterned_directives(ctx, &el.children));
    }
}

fn diagnose_when_directive_shape(ctx: &mut TransformContext<'_>, dir: &DirectiveNode<'_>) {
    if dir.arg.is_some() {
        report_pattern_error(
            ctx,
            &dir.loc,
            "`v-when` does not accept directive arguments.",
        );
    }
    let allowed_case_default =
        dir.name == "case" && dir.modifiers.len() == 1 && dir.modifiers[0].content == "default";
    if !dir.modifiers.is_empty() && !allowed_case_default {
        report_pattern_error(
            ctx,
            &dir.loc,
            "`v-when` does not accept directive modifiers; use `v-when=\"_\"` for a fallback.",
        );
    }
}

fn is_unconditional_fallback_branch(el: &ElementNode<'_>, source: &str) -> bool {
    let Some(case_idx) = find_when_directive(el) else {
        return false;
    };
    let PropNode::Directive(dir) = &el.props[case_idx] else {
        return false;
    };
    let case_expr = dir.exp.as_ref().map(|exp| expression_source(exp, source));
    is_unconditional_fallback_directive(dir, case_expr.as_deref())
}

fn is_unconditional_fallback_directive(dir: &DirectiveNode<'_>, case_expr: Option<&str>) -> bool {
    if dir.name == "case" && dir.modifiers.iter().any(|m| m.content == "default") {
        return true;
    }
    let Some(case_expr) = case_expr else {
        return false;
    };
    if guard_suffix(case_expr).is_some() {
        return false;
    }
    is_wildcard_pattern(pattern_without_guard(case_expr))
}

fn report_pattern_error(ctx: &mut TransformContext<'_>, loc: &SourceLocation, message: &str) {
    ctx.on_error_with_message(ErrorCode::ExtendPoint, message, Some(loc.clone()));
}
