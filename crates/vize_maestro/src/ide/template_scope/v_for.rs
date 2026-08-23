//! `v-for` scope alias lookup in authored templates.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use vize_relief::{ElementNode, ExpressionNode, PropNode, SourceLocation, TemplateChildNode};

use super::{TemplateScopeBinding, TemplateScopeBindingKind};
use crate::ide::IdeContext;

pub(super) fn binding_at(ctx: &IdeContext<'_>, word: &str) -> Option<TemplateScopeBinding> {
    if word.is_empty() {
        return None;
    }

    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let template = descriptor.template.as_ref()?;
    if ctx.offset < template.loc.start || ctx.offset > template.loc.end {
        return None;
    }

    let cursor = ctx.offset - template.loc.start;
    let allocator = vize_carton::Allocator::new();
    let (ast, _) = vize_armature::parse(&allocator, template.content.as_ref());

    find_in_children(&ast.children, template.loc.start, cursor, word)
}

fn find_in_children(
    children: &[TemplateChildNode<'_>],
    template_start: usize,
    cursor: usize,
    word: &str,
) -> Option<TemplateScopeBinding> {
    children
        .iter()
        .find_map(|child| find_in_child(child, template_start, cursor, word))
}

fn find_in_child(
    child: &TemplateChildNode<'_>,
    template_start: usize,
    cursor: usize,
    word: &str,
) -> Option<TemplateScopeBinding> {
    match child {
        TemplateChildNode::Element(element) => {
            find_in_element(element, template_start, cursor, word)
        }
        TemplateChildNode::If(if_node) => {
            if !contains_subtree(child, cursor) {
                return None;
            }
            if_node.branches.iter().find_map(|branch| {
                if contains(&branch.loc, cursor) || contains_children(&branch.children, cursor) {
                    find_in_children(&branch.children, template_start, cursor, word)
                } else {
                    None
                }
            })
        }
        TemplateChildNode::IfBranch(branch) => {
            if contains(&branch.loc, cursor) || contains_children(&branch.children, cursor) {
                find_in_children(&branch.children, template_start, cursor, word)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn find_in_element(
    element: &ElementNode<'_>,
    template_start: usize,
    cursor: usize,
    word: &str,
) -> Option<TemplateScopeBinding> {
    if !contains_element_subtree(element, cursor) {
        return None;
    }

    find_in_children(&element.children, template_start, cursor, word).or_else(|| {
        element.props.iter().find_map(|prop| {
            let PropNode::Directive(directive) = prop else {
                return None;
            };
            if directive.name != "for" {
                return None;
            }
            binding_from_v_for_expression(directive.exp.as_ref()?, template_start, word)
        })
    })
}

fn binding_from_v_for_expression(
    expr: &ExpressionNode<'_>,
    template_start: usize,
    word: &str,
) -> Option<TemplateScopeBinding> {
    let ExpressionNode::Simple(simple) = expr else {
        return None;
    };
    let (left_start, left) = v_for_left_side(simple.content)?;
    split_alias_parts(left, left_start)
        .into_iter()
        .enumerate()
        .find_map(|(index, (part_start, part))| {
            let kind = match index {
                0 => TemplateScopeBindingKind::Value,
                1 => TemplateScopeBindingKind::Key,
                _ => TemplateScopeBindingKind::Index,
            };
            binding_from_text(
                part,
                simple.loc.span.start as usize + part_start,
                template_start,
                word,
                kind,
            )
        })
}

fn v_for_left_side(text: &str) -> Option<(usize, &str)> {
    let separator = find_v_for_separator(text)?;
    let (start, end) = trim_range(text, 0, separator);
    if start >= end {
        return None;
    }

    let (start, end) = strip_wrapping_parens(text, start, end);
    Some((start, &text[start..end]))
}

fn find_v_for_separator(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 && (text[index..].starts_with(" in ") || text[index..].starts_with(" of ")) {
            return Some(index);
        }
    }
    None
}

fn strip_wrapping_parens(text: &str, start: usize, end: usize) -> (usize, usize) {
    if text[start..end].starts_with('(') && text[start..end].ends_with(')') {
        trim_range(text, start + 1, end - 1)
    } else {
        (start, end)
    }
}

fn split_alias_parts(text: &str, base_start: usize) -> Vec<(usize, &str)> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut part_start = 0usize;

    for (index, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                push_trimmed_part(text, base_start, part_start, index, &mut parts);
                part_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    push_trimmed_part(text, base_start, part_start, text.len(), &mut parts);

    parts
}

fn push_trimmed_part<'a>(
    text: &'a str,
    base_start: usize,
    start: usize,
    end: usize,
    parts: &mut Vec<(usize, &'a str)>,
) {
    let (trimmed_start, trimmed_end) = trim_range(text, start, end);
    if trimmed_start < trimmed_end {
        parts.push((
            base_start + trimmed_start,
            &text[trimmed_start..trimmed_end],
        ));
    }
}

fn binding_from_text(
    text: &str,
    relative_start: usize,
    template_start: usize,
    word: &str,
    kind: TemplateScopeBindingKind,
) -> Option<TemplateScopeBinding> {
    if text.is_empty() {
        return None;
    }

    find_identifier(text, word).map(|offset| TemplateScopeBinding {
        name: word.to_string(),
        start: template_start + relative_start + offset,
        end: template_start + relative_start + offset + word.len(),
        kind,
    })
}

fn find_identifier(text: &str, word: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut search_start = 0;
    while let Some(relative) = text[search_start..].find(word) {
        let start = search_start + relative;
        let end = start + word.len();
        if is_identifier_boundary(bytes, start, end) {
            return Some(start);
        }
        search_start = end;
    }
    None
}

fn contains(loc: &SourceLocation, cursor: usize) -> bool {
    let start = loc.span.start as usize;
    let end = loc.span.end as usize;
    start <= cursor && cursor <= end
}

fn contains_subtree(child: &TemplateChildNode<'_>, cursor: usize) -> bool {
    child_subtree_span(child).is_some_and(|(start, end)| start <= cursor && cursor <= end)
}

fn contains_children(children: &[TemplateChildNode<'_>], cursor: usize) -> bool {
    children.iter().any(|child| contains_subtree(child, cursor))
}

fn contains_element_subtree(element: &ElementNode<'_>, cursor: usize) -> bool {
    let (start, end) = element_subtree_span(element);
    start <= cursor && cursor <= end
}

fn child_subtree_span(child: &TemplateChildNode<'_>) -> Option<(usize, usize)> {
    match child {
        TemplateChildNode::Element(element) => Some(element_subtree_span(element)),
        TemplateChildNode::If(if_node) => merge_span(
            loc_span(&if_node.loc),
            if_node
                .branches
                .iter()
                .filter_map(|branch| {
                    merge_span(
                        loc_span(&branch.loc),
                        children_subtree_span(&branch.children),
                    )
                })
                .reduce(|left, right| (left.0.min(right.0), left.1.max(right.1))),
        ),
        TemplateChildNode::IfBranch(branch) => merge_span(
            loc_span(&branch.loc),
            children_subtree_span(&branch.children),
        ),
        TemplateChildNode::Hoisted(_) => None,
        _ => Some(loc_span(child.loc())),
    }
}

fn element_subtree_span(element: &ElementNode<'_>) -> (usize, usize) {
    merge_span(
        loc_span(&element.loc),
        children_subtree_span(&element.children),
    )
    .unwrap_or_else(|| loc_span(&element.loc))
}

fn children_subtree_span(children: &[TemplateChildNode<'_>]) -> Option<(usize, usize)> {
    children
        .iter()
        .filter_map(child_subtree_span)
        .reduce(|left, right| (left.0.min(right.0), left.1.max(right.1)))
}

fn merge_span(left: (usize, usize), right: Option<(usize, usize)>) -> Option<(usize, usize)> {
    Some(match right {
        Some(right) => (left.0.min(right.0), left.1.max(right.1)),
        None => left,
    })
}

fn loc_span(loc: &SourceLocation) -> (usize, usize) {
    (loc.span.start as usize, loc.span.end as usize)
}

fn is_identifier_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);
    !before.is_some_and(|byte| is_identifier_byte(*byte))
        && !after.is_some_and(|byte| is_identifier_byte(*byte))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn trim_range(text: &str, mut start: usize, mut end: usize) -> (usize, usize) {
    while start < end && text.as_bytes()[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && text.as_bytes()[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    (start, end)
}
