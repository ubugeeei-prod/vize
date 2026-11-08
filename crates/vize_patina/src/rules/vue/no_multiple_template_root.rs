//! vue/no-multiple-template-root
//!
//! Require exactly one rendered root in templates that opt into a single-root
//! contract, including Nuxt layouts, pages, and server components.
//!
//! Vue 3 normally permits fragments, so this rule is deliberately opt-in. Once
//! enabled it mirrors `eslint-plugin-vue@10.9.2`: comments and blank text are
//! ignored, a `v-if`/`v-else-if`/`v-else` chain is one logical root, and any
//! other element or meaningful text is an additional root. A lone `<slot>`,
//! `<template>`, or `v-for` root is also rejected because it may render a shape
//! other than one element.
//!
//! The upstream `disallowComments` option is outside Patina's current
//! severity-only rule configuration. Nuxt uses the default (`false`), so root
//! comments remain accepted here.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-multiple-template-root",
    description: "Disallow multiple root nodes in a template",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow template shapes that do not render exactly one element root.
pub struct NoMultipleTemplateRoot;

#[inline]
fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}

/// Recover the start-tag range, matching vue-eslint-parser's reported span.
/// Quoted `>` bytes do not terminate the tag.
fn start_tag_loc(source: &str, element: &ElementNode<'_>) -> SourceLocation {
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    let Some(element_source) = source.get(start..end) else {
        return element.loc.clone();
    };

    let mut quote = None;
    for (relative, &byte) in element_source.as_bytes().iter().enumerate() {
        match (quote, byte) {
            (Some(open), close) if open == close => quote = None,
            (Some(_), _) => {}
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => {
                let mut loc = element.loc.clone();
                loc.span.end = (start + relative + 1) as u32;
                return loc;
            }
            (None, _) => {}
        }
    }

    element.loc.clone()
}

fn tag_end(source: &[u8], start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, &byte) in source.get(start..)?.iter().enumerate() {
        match (quote, byte) {
            (Some(open), close) if open == close => quote = None,
            (Some(_), _) => {}
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => return Some(start + offset),
            (None, _) => {}
        }
    }
    None
}

fn tag_name_matches(bytes: &[u8], start: usize, tag: &[u8]) -> bool {
    let end = start + tag.len();
    bytes
        .get(start..end)
        .is_some_and(|name| name.eq_ignore_ascii_case(tag))
        && bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(*byte, b'/' | b'>'))
}

/// Recover the whole authored element range. Relief intentionally keeps only
/// the start tag in `ElementNode::loc`. Counting same-name authored tags avoids
/// relying on a repaired AST subtree: browser tree construction can split a
/// nested interactive element into siblings, while recovered component trees
/// can conversely retain a later authored root among the first node's children.
fn full_element_loc(source: &str, element: &ElementNode<'_>) -> SourceLocation {
    let mut loc = element.loc.clone();
    if element.is_self_closing {
        return loc;
    }

    let bytes = source.as_bytes();
    let tag = element.tag.as_bytes();
    let mut cursor = loc.span.end as usize;
    let mut depth = 1usize;

    while cursor < bytes.len() {
        let Some(relative) = bytes
            .get(cursor..)
            .and_then(|rest| rest.iter().position(|&byte| byte == b'<'))
        else {
            break;
        };
        let start = cursor + relative;
        if bytes
            .get(start..)
            .is_some_and(|rest| rest.starts_with(b"<!--"))
        {
            cursor = bytes
                .get(start + 4..)
                .and_then(|rest| memchr::memmem::find(rest, b"-->"))
                .map_or(bytes.len(), |offset| start + 4 + offset + 3);
            continue;
        }
        let Some(end) = tag_end(bytes, start) else {
            break;
        };
        if bytes.get(start + 1) == Some(&b'/') && tag_name_matches(bytes, start + 2, tag) {
            depth -= 1;
            if depth == 0 {
                loc.span.end = (end + 1) as u32;
                return loc;
            }
        } else if tag_name_matches(bytes, start + 1, tag)
            && bytes.get(start + 1..end).is_some_and(|inside| {
                inside.iter().rfind(|byte| !byte.is_ascii_whitespace()) != Some(&b'/')
            })
        {
            depth += 1;
        }
        cursor = end + 1;
    }

    loc
}

#[inline]
fn is_non_blank(source: &str, loc: &SourceLocation) -> bool {
    source
        .get(loc.span.start as usize..loc.span.end as usize)
        .is_some_and(|text| !text.trim().is_empty())
}

/// Results from the first, allocation-free pass over the template root.
struct RootShape<'ast, 'root> {
    has_element: bool,
    extra_element: Option<&'root ElementNode<'ast>>,
    extra_text: Option<&'root SourceLocation>,
}

impl<'ast, 'root> RootShape<'ast, 'root> {
    fn classify(source: &str, root: &'root RootNode<'ast>) -> Self {
        let mut shape = Self {
            has_element: false,
            extra_element: None,
            extra_text: None,
        };
        let mut chain_open = false;

        for child in &root.children {
            match child {
                TemplateChildNode::Element(element) if !shape.has_element => {
                    shape.has_element = true;
                    chain_open = has_directive(element, "if");
                }
                TemplateChildNode::Element(element)
                    if chain_open && has_directive(element, "else-if") => {}
                TemplateChildNode::Element(element)
                    if chain_open && has_directive(element, "else") =>
                {
                    chain_open = false;
                }
                TemplateChildNode::Element(element) => shape.extra_element = Some(element),
                TemplateChildNode::Comment(_) => {}
                other if is_non_blank(source, other.loc()) => {
                    shape.extra_text = Some(other.loc());
                }
                _ => {}
            }
        }

        shape
    }
}

impl Rule for NoMultipleTemplateRoot {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let shape = RootShape::classify(ctx.source, root);

        // Upstream gives meaningful text precedence over every element report.
        if let Some(loc) = shape.extra_text {
            ctx.error_with_help(
                ctx.t("vue/no-multiple-template-root.text_root"),
                loc,
                ctx.t("vue/no-multiple-template-root.help"),
            );
            return;
        }

        // Only the last additional element is reported.
        if let Some(element) = shape.extra_element {
            // HTML tree construction splits authored nested interactive
            // elements such as `<button><button>` into root siblings. If the
            // apparent extra root still lies before the authored closing tag
            // of the first root, it was not authored as a Vue root.
            let repaired_nested_root = root.children.iter().find_map(|child| {
                let TemplateChildNode::Element(first) = child else {
                    return None;
                };
                Some(element.loc.span.start < full_element_loc(ctx.source, first).span.end)
            });
            if repaired_nested_root == Some(true) {
                return;
            }
            let loc = full_element_loc(ctx.source, element);
            ctx.error_with_help(
                ctx.t("vue/no-multiple-template-root.multiple_root"),
                &loc,
                ctx.t("vue/no-multiple-template-root.help"),
            );
            return;
        }

        if !shape.has_element {
            return;
        }

        // With no additional root, every element is either the first root or a
        // branch in its conditional chain. Each branch receives the root-kind
        // and root-v-for checks, exactly as upstream does.
        for child in &root.children {
            let TemplateChildNode::Element(element) = child else {
                continue;
            };
            let loc = start_tag_loc(ctx.source, element);
            let tag = element.tag;

            if matches!(tag, "template" | "slot") {
                ctx.error_with_help(
                    ctx.t_fmt(
                        "vue/no-multiple-template-root.disallowed_element",
                        &[("tag", tag)],
                    ),
                    &loc,
                    ctx.t("vue/no-multiple-template-root.help"),
                );
            }
            if has_directive(element, "for") {
                ctx.error_with_help(
                    ctx.t("vue/no-multiple-template-root.disallowed_directive"),
                    &loc,
                    ctx.t("vue/no-multiple-template-root.help"),
                );
            }
        }
    }
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
