//! Whitespace condensing logic for the parser.
//!
//! Implements the `condense` whitespace strategy which removes or condenses
//! whitespace-only text nodes between elements and collapses runs of
//! whitespace inside mixed text nodes, matching `@vue/compiler-sfc`. Vue's
//! whitespace alphabet is the ASCII set `[ \t\n\f\r]`, so this module uses
//! `is_vue_whitespace` rather than the full-Unicode `char::is_whitespace`.

use vize_carton::{String, Vec};
use vize_relief::TemplateChildNode;

/// Per Vue: only `[ \t\n\f\r]` is whitespace for the condense strategy.
#[inline]
fn is_vue_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000C}' | '\r')
}

/// Collapse every maximal run of `[ \t\n\f\r]` in `text` to a single U+0020.
fn condense_internal_whitespace(text: &str) -> Option<String> {
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

    let mut out = String::with_capacity(text.len());
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
    Some(out)
}

/// Condense whitespace in children
pub(super) fn condense_whitespace<'a>(
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    is_pre_tag: fn(&str) -> bool,
) {
    // First pass: remove leading whitespace-only text nodes
    while !children.is_empty() {
        if let TemplateChildNode::Text(ref text) = children[0]
            && text.content.chars().all(is_vue_whitespace)
        {
            children.remove(0);
            continue;
        }
        break;
    }

    // Remove trailing whitespace-only text nodes
    while !children.is_empty() {
        let last = children.len() - 1;
        if let TemplateChildNode::Text(ref text) = children[last]
            && text.content.chars().all(is_vue_whitespace)
        {
            children.remove(last);
            continue;
        }
        break;
    }

    let mut i = 0;
    while i < children.len() {
        let action = if is_whitespace_text(&children[i]) {
            let mut run_end = i + 1;
            let mut has_newline = whitespace_has_newline(&children[i]);
            while run_end < children.len() && is_whitespace_text(&children[run_end]) {
                has_newline |= whitespace_has_newline(&children[run_end]);
                run_end += 1;
            }

            let prev = (0..i)
                .rev()
                .find(|&idx| !is_whitespace_text(&children[idx]));
            let next = (run_end..children.len()).find(|&idx| !is_whitespace_text(&children[idx]));

            let prev_is_text = prev.is_some_and(|idx| is_text_like(&children[idx]));
            let next_is_text = next.is_some_and(|idx| is_text_like(&children[idx]));

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
                if let TemplateChildNode::Text(ref mut text) = children[i] {
                    text.content = " ".into();
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
                if let TemplateChildNode::Text(ref mut text) = children[i]
                    && let Some(condensed) = condense_internal_whitespace(text.content.as_str())
                {
                    text.content = condensed;
                }
            }
        }

        // Recurse into elements
        if let TemplateChildNode::Element(ref mut el) = children[i]
            && !is_pre_tag(el.tag.as_str())
        {
            condense_whitespace(&mut el.children, is_pre_tag);
        }

        i += 1;
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
