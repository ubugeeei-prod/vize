//! Whitespace condensing logic for the parser.
//!
//! Implements the `condense` whitespace strategy which removes or condenses
//! whitespace-only text nodes between elements.

use vize_carton::Vec;
use vize_relief::ast::TemplateChildNode;

/// Condense whitespace in children
pub(super) fn condense_whitespace<'a>(children: &mut Vec<'a, TemplateChildNode<'a>>) {
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
        // Determine what action to take for whitespace-only text nodes
        let action = if let TemplateChildNode::Text(ref text) = children[i] {
            let content = text.content.as_str();
            if content.chars().all(char::is_whitespace) {
                let prev_is_text = i > 0
                    && matches!(
                        children[i - 1],
                        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
                    );
                let next_is_text = i + 1 < children.len()
                    && matches!(
                        children[i + 1],
                        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
                    );

                if !prev_is_text && !next_is_text {
                    // Between non-text nodes (e.g. two elements):
                    // If whitespace contains a newline, remove it entirely
                    // (this handles indentation between block-level elements).
                    // If it's just spaces (no newline), condense to single space
                    // to preserve inline spacing (vuejs/core #7542).
                    let has_newline = content.contains('\n');
                    if has_newline {
                        WhitespaceAction::Remove
                    } else {
                        WhitespaceAction::Condense
                    }
                } else {
                    WhitespaceAction::Keep
                }
            } else {
                WhitespaceAction::Keep
            }
        } else {
            WhitespaceAction::Keep
        };

        match action {
            WhitespaceAction::Remove => {
                children.remove(i);
                continue;
            }
            WhitespaceAction::Condense => {
                // Condense whitespace between two elements to a single space
                if let TemplateChildNode::Text(ref mut text) = children[i] {
                    text.content = " ".into();
                }
            }
            WhitespaceAction::Keep => {}
        }

        // Recurse into elements
        if let TemplateChildNode::Element(ref mut el) = children[i] {
            condense_whitespace(&mut el.children);
        }

        i += 1;
    }
}

/// Action to take for a whitespace-only text node during condensing
enum WhitespaceAction {
    /// Keep the node as-is
    Keep,
    /// Remove the node entirely
    Remove,
    /// Condense to a single space
    Condense,
}
