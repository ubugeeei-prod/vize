//! Branch-key comparison helpers for structural directives.

use vize_carton::String;

use crate::{ExpressionNode, PropNode};

/// Extract a key identity when two branches can be proven to use the same key.
pub(super) fn extract_key_value_str(
    prop: &PropNode<'_>,
    template_syntax_quirks: bool,
    source: &str,
) -> Option<String> {
    match prop {
        PropNode::Attribute(attr) => attr.value.as_ref().map(|value| value.content.into()),
        PropNode::Directive(dir) => {
            let expression = dir.exp.as_ref()?;
            if template_syntax_quirks && !is_stable_quirks_key(expression) {
                return None;
            }
            Some(match expression {
                ExpressionNode::Simple(simple) => simple.content.into(),
                ExpressionNode::Compound(compound) => String::new(compound.loc.span.slice(source)),
            })
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

    let source = simple.content.trim();
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
