//! Experimental patterned template desugaring (`v-match` / `v-case`).

use vize_carton::{Allocator, Box, String, Vec, cstr, ensure_sufficient_stack};

use crate::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, RootNode,
    SimpleExpressionNode, TemplateChildNode,
};

/// Rewrite experimental `v-match` / `v-case` syntax into the existing `v-if`
/// structural directive chain.
pub fn desugar_patterned_templates<'a>(allocator: &'a Allocator, root: &mut RootNode<'a>) {
    let source = root.source;
    rewrite_children(allocator, &mut root.children, source);
}

fn rewrite_children<'a>(
    allocator: &'a Allocator,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    source: &str,
) {
    for child in children.iter_mut() {
        let TemplateChildNode::Element(el) = child else {
            continue;
        };

        ensure_sufficient_stack(|| rewrite_children(allocator, &mut el.children, source));
        rewrite_match_element(allocator, el, source);
    }
}

fn rewrite_match_element<'a>(allocator: &'a Allocator, el: &mut ElementNode<'a>, source: &str) {
    let Some((match_idx, match_expr)) = find_match_expression(el, source) else {
        return;
    };

    let mut has_case = false;
    for child in el.children.iter_mut() {
        let TemplateChildNode::Element(case_el) = child else {
            continue;
        };
        if rewrite_case_element(allocator, case_el, &match_expr, has_case, source) {
            has_case = true;
        }
    }

    if !has_case {
        return;
    }

    el.props.remove(match_idx);
    el.tag = "template";
    el.tag_type = ElementType::Template;
    el.is_self_closing = false;
}

fn find_match_expression(el: &ElementNode<'_>, source: &str) -> Option<(usize, String)> {
    for (idx, prop) in el.props.iter().enumerate() {
        if let PropNode::Directive(dir) = prop
            && dir.name == "match"
        {
            let exp = dir.exp.as_ref().map(|exp| expression_source(exp, source))?;
            return Some((idx, exp));
        }
    }
    None
}

fn rewrite_case_element<'a>(
    allocator: &'a Allocator,
    el: &mut ElementNode<'a>,
    match_expr: &str,
    has_previous_branch: bool,
    source: &str,
) -> bool {
    let Some(case_idx) = find_case_directive(el) else {
        return false;
    };

    let (is_default, case_expr) = match &el.props[case_idx] {
        PropNode::Directive(dir) => (
            dir.modifiers.iter().any(|m| m.content == "default"),
            dir.exp.as_ref().map(|exp| expression_source(exp, source)),
        ),
        PropNode::Attribute(_) => return false,
    };

    if !is_default && case_expr.is_none() {
        return false;
    }

    let mut case_dir = match el.props.remove(case_idx) {
        PropNode::Directive(dir) => Box::unbox(dir),
        PropNode::Attribute(_) => return false,
    };

    let (directive_name, directive_raw_name) = if is_default {
        if has_previous_branch {
            ("else", "v-else")
        } else {
            ("if", "v-if")
        }
    } else if has_previous_branch {
        ("else-if", "v-else-if")
    } else {
        ("if", "v-if")
    };

    let condition = if is_default {
        if has_previous_branch {
            None
        } else {
            Some(String::from("true"))
        }
    } else {
        case_expr.map(|case_expr| build_case_condition(match_expr, &case_expr))
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

    true
}

fn find_case_directive(el: &ElementNode<'_>) -> Option<usize> {
    el.props
        .iter()
        .position(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "case"))
}

fn rewrite_case_directive<'a>(
    allocator: &'a Allocator,
    dir: &mut DirectiveNode<'a>,
    name: &'static str,
    raw_name: &'static str,
    condition: Option<String>,
) {
    let loc = dir.loc.clone();
    dir.name = name;
    dir.raw_name = Some(raw_name);
    dir.exp = condition.map(|condition| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(allocator.alloc_str(&condition), false, loc),
            &allocator,
        ))
    });
    dir.arg = None;
    dir.modifiers = Vec::new_in(&allocator);
    dir.for_parse_result = None;
    dir.shorthand = false;
}

fn build_case_condition(match_expr: &str, case_expr: &str) -> String {
    let match_expr = match_expr.trim();
    let case_expr = case_expr.trim();

    if is_array_literal(case_expr) {
        cstr!("({case_expr}).includes({match_expr})")
    } else {
        cstr!("({match_expr}) === ({case_expr})")
    }
}

fn is_array_literal(expr: &str) -> bool {
    expr.starts_with('[') && expr.ends_with(']')
}

fn expression_source(exp: &ExpressionNode<'_>, source: &str) -> String {
    match exp {
        ExpressionNode::Simple(simple) => simple.content.into(),
        ExpressionNode::Compound(compound) => String::new(compound.loc.span.slice(source)),
    }
}
