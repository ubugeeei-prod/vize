//! Branch-key comparison helpers for structural directives.

use vize_l0::String;

use crate::{ExpressionNode, PropNode};

/// Expression kinds matter: a processed compound key is never a raw simple key.
pub(super) fn keys_equal(
    existing: &PropNode<'_>,
    new_key: &PropNode<'_>,
    template_syntax_quirks: bool,
    source: &str,
    existing_raw: bool,
) -> bool {
    if !matches!(
        (existing, new_key),
        (PropNode::Attribute(_), PropNode::Attribute(_))
            | (PropNode::Directive(_), PropNode::Directive(_))
    ) {
        return false;
    }
    matches!((extract_key_value_str(existing, template_syntax_quirks, source, existing_raw),
        extract_key_value_str(new_key, template_syntax_quirks, source, false)),
        (Some(existing), Some(new_key)) if existing == new_key)
}

/// Extract a key identity when two branches can be proven to use the same key.
pub(super) fn extract_key_value_str(
    prop: &PropNode<'_>,
    template_syntax_quirks: bool,
    source: &str,
    raw: bool,
) -> Option<String> {
    match prop {
        PropNode::Attribute(attr) => attr.value.as_ref().map(|value| value.content.into()),
        PropNode::Directive(dir) => {
            let expression = dir.exp.as_ref()?;
            if raw {
                let raw = expression.loc().span.slice(source);
                if template_syntax_quirks && !is_stable_text(raw) {
                    return None;
                }
                return Some(raw.into());
            }
            if template_syntax_quirks && !is_stable_quirks_key(expression) {
                return None;
            }
            match expression {
                ExpressionNode::Simple(simple) => Some(simple.content.into()),
                ExpressionNode::Compound(_) => None,
            }
        }
    }
}

fn is_stable_quirks_key(expression: &ExpressionNode<'_>) -> bool {
    let ExpressionNode::Simple(simple) = expression else {
        return false;
    };
    if simple.is_static {
        return true;
    }

    is_stable_text(simple.content)
}

fn is_stable_text(source: &str) -> bool {
    let source = source.trim();
    matches!(source, "true" | "false" | "null")
        || source.parse::<f64>().is_ok()
        || is_quoted_literal(source)
}

fn is_quoted_literal(source: &str) -> bool {
    let Some(first) = source.as_bytes().first() else {
        return false;
    };
    let Some(last) = source.as_bytes().last() else {
        return false;
    };
    if first != last || !matches!(first, b'\'' | b'"' | b'`') {
        return false;
    }
    *first != b'`' || !source.contains("${")
}
