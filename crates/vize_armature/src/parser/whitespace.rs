//! Whitespace condensing logic for the parser.
//!
//! Implements the `condense` whitespace strategy which removes or condenses
//! whitespace-only text nodes between elements.

use vize_carton::Vec;
use vize_relief::ast::TemplateChildNode;

/// Condense whitespace in children
pub(super) fn condense_whitespace<'a>(
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    is_pre_tag: fn(&str) -> bool,
) {
    // First pass: remove leading whitespace-only text nodes
    while !children.is_empty() {
        if let TemplateChildNode::Text(ref text) = children[0] {
            if text.content.chars().all(char::is_whitespace) {
                children.remove(0);
                continue;
            }
        }
        break;
    }

    // Remove trailing whitespace-only text nodes
    while !children.is_empty() {
        let last = children.len() - 1;
        if let TemplateChildNode::Text(ref text) = children[last] {
            if text.content.chars().all(char::is_whitespace) {
                children.remove(last);
                continue;
            }
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
            WhitespaceAction::Keep => {}
        }

        // Recurse into elements
        if let TemplateChildNode::Element(ref mut el) = children[i] {
            if !is_pre_tag(el.tag.as_str()) {
                condense_whitespace(&mut el.children, is_pre_tag);
            }
        }

        i += 1;
    }
}

#[inline]
fn is_whitespace_text(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Text(text) if text.content.chars().all(char::is_whitespace))
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
        TemplateChildNode::Text(text) => !text.content.chars().all(char::is_whitespace),
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
