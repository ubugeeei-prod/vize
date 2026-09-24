//! Experimental patterned templates, sharing the parser used by Croquis/Canon.

mod directives;
mod lowering;
mod names;
mod syntax;

use vize_armature::patterns::{
    MatchArm, PatternKind, attribute_source_offset, parse_match_pattern,
};
use vize_s0::{String, Vec, cstr, ensure_sufficient_stack};

use crate::{
    DirectiveNode, ElementNode, ErrorCode, PropNode, RootNode, SourceLocation, TemplateChildNode,
    TransformContext,
};

use self::directives::{create_directive, install_arm_scope, install_match_scope};
use self::lowering::generate_selector;
use self::syntax::expression_source;
use super::structural::MATCH_SCOPE_RAW_NAME;

pub fn desugar_patterned_templates<'a>(ctx: &mut TransformContext<'a>, root: &mut RootNode<'a>) {
    if !has_match(&root.children) {
        diagnose_remaining_patterned_directives(ctx, &root.children, false);
        return;
    }
    let prefix = names::unique_prefix(ctx, root);
    rewrite_children(ctx, &mut root.children, root.source, &prefix, &mut 0);
    diagnose_remaining_patterned_directives(ctx, &root.children, false);
}

fn has_match(children: &[TemplateChildNode<'_>]) -> bool {
    children.iter().any(|child| {
        let TemplateChildNode::Element(el) = child else {
            return false;
        };
        el.props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "match"))
            || ensure_sufficient_stack(|| has_match(&el.children))
    })
}

/// Report patterned-template directives when their opt-in flag is disabled.
pub fn diagnose_disabled_patterned_templates<'a>(
    ctx: &mut TransformContext<'a>,
    root: &RootNode<'a>,
) {
    diagnose_remaining_patterned_directives(ctx, &root.children, true);
}

fn rewrite_children<'a>(
    ctx: &mut TransformContext<'a>,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    source: &str,
    prefix: &str,
    next: &mut usize,
) {
    for child in children.iter_mut() {
        let TemplateChildNode::Element(el) = child else {
            continue;
        };
        rewrite_match_element(ctx, el, source, prefix, next);
        ensure_sufficient_stack(|| rewrite_children(ctx, &mut el.children, source, prefix, next));
    }
}

fn rewrite_match_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut ElementNode<'a>,
    source: &str,
    prefix: &str,
    next: &mut usize,
) {
    let Some(match_idx) = el
        .props
        .iter()
        .position(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "match"))
    else {
        return;
    };
    let Some(PropNode::Directive(dir)) = el.props.get(match_idx) else {
        return;
    };
    let match_loc = dir.loc.clone();
    let errors_before = ctx.errors.len();
    let subject = dir
        .exp
        .as_ref()
        .map(|exp| expression_source(exp, source))
        .filter(|text| !text.trim().is_empty());
    if subject.is_none() {
        report_pattern_error(ctx, &match_loc, "`v-match` requires a subject expression.");
    }
    if dir.arg.is_some() || !dir.modifiers.is_empty() {
        report_pattern_error(
            ctx,
            &match_loc,
            "`v-match` does not accept directive arguments or modifiers.",
        );
    }
    let valid_header = ctx.errors.len() == errors_before;
    let local = cstr!("{prefix}_{}", *next);
    *next += 1;
    let mut arms: std::vec::Vec<MatchArm> = std::vec::Vec::new();
    let mut fallback_seen = false;
    for child in &mut el.children {
        let TemplateChildNode::Element(child) = child else {
            continue;
        };
        let Some(case_idx) = child.props.iter().position(
            |prop| matches!(prop, PropNode::Directive(dir) if matches!(dir.name, "when" | "case")),
        ) else {
            continue;
        };
        let Some(PropNode::Directive(dir)) = child.props.get(case_idx) else {
            continue;
        };
        let case_loc = dir.loc.clone();
        if dir.name == "when" && child.props.iter().any(|prop| matches!(prop, PropNode::Directive(other) if matches!(other.name, "if" | "else-if" | "else" | "for" | "match"))) {
            report_pattern_error(ctx, &case_loc, "v-when cannot share an element with v-if, v-else-if, v-else, v-for or v-match.");
            child.props.remove(case_idx);
            continue;
        }
        let Some(arm) = parse_arm(ctx, dir, source) else {
            child.props.remove(case_idx);
            continue;
        };
        if fallback_seen {
            report_pattern_error(
                ctx,
                &case_loc,
                "`v-when=\"_\"` fallback arms must be last and unique within a `v-match` block.",
            );
        }
        fallback_seen |= matches!(arm.pattern.kind, PatternKind::Wildcard) && arm.guard.is_none();
        child.props.remove(case_idx);
        if valid_header {
            install_arm_scope(ctx.allocator, child, &arm, &local, arms.len(), case_loc);
        }
        arms.push(arm);
    }
    el.props.remove(match_idx);
    if arms.is_empty() {
        if ctx.errors.len() == errors_before {
            report_pattern_error(
                ctx,
                &match_loc,
                "`v-match` requires at least one direct `v-when` branch.",
            );
        }
        return;
    }
    // Invalid headers still consume and validate their direct arms, but cannot
    // produce a selector or turn those arms into misleading orphan errors.
    let Some(subject) = subject.filter(|_| valid_header) else {
        return;
    };
    let selector = generate_selector(&arms, &subject, &local);
    let scope = create_directive(
        ctx.allocator,
        "for",
        MATCH_SCOPE_RAW_NAME,
        Some(cstr!("{local} in {selector}")),
        match_loc,
    );
    install_match_scope(ctx.allocator, el, scope);
}

fn parse_arm(
    ctx: &mut TransformContext<'_>,
    dir: &DirectiveNode<'_>,
    source: &str,
) -> Option<MatchArm> {
    let legacy_default = dir.name == "case"
        && matches!(dir.modifiers.as_slice(), [modifier] if modifier.content == "default");
    if dir.arg.is_some() || (!dir.modifiers.is_empty() && !legacy_default) {
        report_pattern_error(
            ctx,
            &dir.loc,
            "`v-when` does not accept directive arguments or modifiers; use `v-when=\"_\"` for a fallback.",
        );
        return None;
    }
    let expression = if legacy_default {
        String::from("_")
    } else if let Some(exp) = &dir.exp {
        expression_source(exp, source)
    } else {
        report_pattern_error(ctx, &dir.loc, "`v-when` requires a pattern.");
        return None;
    };
    match parse_match_pattern(&expression) {
        Ok(arm) => Some(arm),
        Err(error) => {
            let mut loc = dir
                .exp
                .as_ref()
                .map_or_else(|| dir.loc.clone(), |exp| exp.loc().clone());
            if let Some(raw) = source.get(loc.span.start as usize..loc.span.end as usize) {
                let at = loc.span.start + attribute_source_offset(raw, error.offset);
                if at < loc.span.end {
                    loc.span.start = at;
                }
            }
            report_pattern_error(ctx, &loc, &error.message);
            None
        }
    }
}

fn diagnose_remaining_patterned_directives<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
    disabled: bool,
) {
    for child in children {
        let TemplateChildNode::Element(el) = child else {
            continue;
        };
        for prop in &el.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if !matches!(dir.name, "match" | "when" | "case") {
                continue;
            }
            let message = if disabled {
                "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`."
            } else if dir.name == "match" {
                "`v-match` requires at least one direct `v-when` branch."
            } else {
                "`v-when` branches must be direct children of a `v-match` container."
            };
            report_pattern_error(ctx, &dir.loc, message);
        }
        ensure_sufficient_stack(|| {
            diagnose_remaining_patterned_directives(ctx, &el.children, disabled)
        });
    }
}

fn report_pattern_error(ctx: &mut TransformContext<'_>, loc: &SourceLocation, message: &str) {
    ctx.on_error_with_message(ErrorCode::ExtendPoint, message, Some(loc.clone()));
}
