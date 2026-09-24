//! Whitespace condensing logic for the parser.
//!
//! Implements the `condense` whitespace strategy which removes or condenses
//! whitespace-only text nodes between elements and collapses runs of
//! whitespace inside mixed text nodes, matching `@vue/compiler-sfc`. Vue's
//! whitespace alphabet is the ASCII set `[ \t\n\f\r]`, so this module uses
//! `is_vue_whitespace` rather than the full-Unicode `char::is_whitespace`.

use vize_relief::TemplateChildNode;
use vize_s0::{Allocator, StringBuilder, Vec, ensure_sufficient_stack};

/// Per Vue: only `[ \t\n\f\r]` is whitespace for the condense strategy.
#[inline]
fn is_vue_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000C}' | '\r')
}

/// Collapse every maximal run of `[ \t\n\f\r]` in `text` to a single U+0020.
///
/// Returns `None` when the text already satisfies the condense strategy, so
/// the untouched node keeps borrowing the template source.
fn condense_internal_whitespace<'a>(allocator: &'a Allocator, text: &str) -> Option<&'a str> {
    let needs_condense = {
        let mut prev_ws = false;
        let mut any_run = false;
        let mut has_non_space_ws = false;
        for c in text.chars() {
            if is_vue_whitespace(c) {
                if prev_ws {
                    any_run = true;
                }
                if c != ' ' {
                    has_non_space_ws = true;
                }
                prev_ws = true;
            } else {
                prev_ws = false;
            }
        }
        any_run || has_non_space_ws
    };

    if !needs_condense {
        return None;
    }

    let mut out = StringBuilder::with_capacity_in(text.len(), allocator);
    let mut prev_ws = false;
    for c in text.chars() {
        if is_vue_whitespace(c) {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(c);
            prev_ws = false;
        }
    }
    Some(out.into_str())
}

/// Condense whitespace in children
///
/// Tokenizing is iterative — the parser keeps its own open-element stack — but
/// this post-pass walks the finished tree with the call stack, so it is the one
/// place where parsing costs a frame per nesting level. Its descent is therefore
/// guarded (`vize_s0::recursion`).
pub(super) fn condense_whitespace<'a>(
    allocator: &'a Allocator,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    is_pre_tag: fn(&str) -> bool,
) {
    // First pass: remove leading whitespace-only text nodes
    while children.first().is_some_and(is_whitespace_text) {
        children.remove(0);
    }

    // Remove trailing whitespace-only text nodes
    while children.last().is_some_and(is_whitespace_text) {
        children.pop();
    }

    let mut i = 0;
    while let Some(child) = children.get(i) {
        let action = if is_whitespace_text(child) {
            let mut run_end = i + 1;
            let mut has_newline = whitespace_has_newline(child);
            while let Some(next) = children
                .get(run_end)
                .filter(|next| is_whitespace_text(next))
            {
                has_newline |= whitespace_has_newline(next);
                run_end += 1;
            }

            let before = children.get(..i).unwrap_or_default();
            let after = children.get(run_end..).unwrap_or_default();
            let prev_is_text = before
                .iter()
                .rev()
                .find(|node| !is_whitespace_text(node))
                .is_some_and(is_text_like);
            let next_is_text = after
                .iter()
                .find(|node| !is_whitespace_text(node))
                .is_some_and(is_text_like);

            if !prev_is_text && !next_is_text && has_newline {
                WhitespaceAction::Remove(run_end - i)
            } else {
                WhitespaceAction::Condense(run_end - i)
            }
        } else {
            WhitespaceAction::Keep
        };

        match action {
            WhitespaceAction::Remove(len) => {
                for _ in 0..len {
                    children.remove(i);
                }
                continue;
            }
            WhitespaceAction::Condense(len) => {
                // Condense whitespace runs to a single space.
                if let Some(TemplateChildNode::Text(text)) = children.get_mut(i) {
                    text.content = " ";
                }
                for _ in 1..len {
                    children.remove(i + 1);
                }
            }
            WhitespaceAction::Keep => {
                // For mixed-content text nodes (text + whitespace runs),
                // collapse internal whitespace runs to a single U+0020 too,
                // matching Vue's `condense` strategy. Without this `x   y\n
                // z` would keep its raw whitespace and diverge from
                // `@vue/compiler-sfc`. (#960)
                if let Some(TemplateChildNode::Text(text)) = children.get_mut(i)
                    && let Some(condensed) = condense_internal_whitespace(allocator, text.content)
                {
                    text.content = condensed;
                }
            }
        }

        // Recurse into elements
        if let Some(TemplateChildNode::Element(el)) = children.get_mut(i)
            && !is_pre_tag(el.tag)
        {
            ensure_sufficient_stack(|| {
                condense_whitespace(allocator, &mut el.children, is_pre_tag)
            });
        }

        i += 1;
    }
}

/// Vue's `preserve` mode keeps mixed text verbatim, but still drops leading
/// and trailing whitespace-only children and normalizes whitespace-only nodes
/// between meaningful siblings to one space. `<pre>` remains raw in both modes.
pub(super) fn preserve_whitespace<'a>(
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    is_pre_tag: fn(&str) -> bool,
) {
    while children.first().is_some_and(is_whitespace_text) {
        children.remove(0);
    }
    while children.last().is_some_and(is_whitespace_text) {
        children.pop();
    }
    for child in children.iter_mut() {
        match child {
            TemplateChildNode::Text(text) if text.content.chars().all(is_vue_whitespace) => {
                text.content = " ";
            }
            TemplateChildNode::Element(element) if !is_pre_tag(element.tag) => {
                ensure_sufficient_stack(|| preserve_whitespace(&mut element.children, is_pre_tag));
            }
            _ => {}
        }
    }
}

#[inline]
fn is_whitespace_text(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Text(text) if text.content.chars().all(is_vue_whitespace))
}

#[inline]
fn whitespace_has_newline(child: &TemplateChildNode<'_>) -> bool {
    matches!(
        child,
        TemplateChildNode::Text(text) if text.content.contains('\n') || text.content.contains('\r')
    )
}

#[inline]
fn is_text_like(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Interpolation(_) => true,
        TemplateChildNode::Text(text) => !text.content.chars().all(is_vue_whitespace),
        _ => false,
    }
}

/// Action to take for a whitespace-only text node during condensing
enum WhitespaceAction {
    /// Keep the node as-is
    Keep,
    /// Remove the node entirely
    Remove(usize),
    /// Condense a run to a single space
    Condense(usize),
}
