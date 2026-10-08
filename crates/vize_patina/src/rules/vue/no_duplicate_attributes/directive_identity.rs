use super::get_static_expression_content;
use vize_l0::{String, ToCompactString, is_builtin_directive};
use vize_relief::DirectiveNode;

pub(super) fn key(directive: &DirectiveNode<'_>) -> Option<String> {
    let mut key = directive
        .raw_name
        .unwrap_or(directive.name)
        .to_compact_string();
    if !is_builtin_directive(directive.name) {
        if let Some(argument) = &directive.arg {
            // A dynamic argument cannot be proven to identify the same binding.
            let argument = get_static_expression_content(argument)?;
            key.push(':');
            key.push_str(&argument);
        }
        for modifier in &directive.modifiers {
            key.push('.');
            key.push_str(modifier.content);
        }
    }
    Some(key)
}
