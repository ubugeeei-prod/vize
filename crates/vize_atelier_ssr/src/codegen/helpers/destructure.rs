//! Destructuring-pattern parameter extraction for SSR codegen scopes.

use vize_atelier_core::{CompoundExpressionChild, ExpressionNode, ForNode};
use vize_s0::{FxHashSet, SmallVec, String, ToCompactString};

pub(crate) fn collect_for_scoped_params(for_node: &ForNode, source: &str) -> FxHashSet<String> {
    let mut params = FxHashSet::default();

    if let Some(value) = &for_node.value_alias {
        collect_expression_params(value, &mut params, source);
    }
    if let Some(key) = &for_node.key_alias {
        collect_expression_params(key, &mut params, source);
    }
    if let Some(index) = &for_node.object_index_alias {
        collect_expression_params(index, &mut params, source);
    }

    params
}

fn collect_expression_params(expr: &ExpressionNode, params: &mut FxHashSet<String>, source: &str) {
    let content = match expr {
        ExpressionNode::Simple(simple) => String::new(simple.content),
        ExpressionNode::Compound(compound) => {
            let mut content = String::default();
            for child in &compound.children {
                match child {
                    CompoundExpressionChild::Simple(simple) => content.push_str(simple.content),
                    CompoundExpressionChild::String(value) => content.push_str(value),
                    _ => {}
                }
            }
            if content.is_empty() {
                String::new(compound.loc.span.slice(source))
            } else {
                content
            }
        }
    };
    extract_destructure_params(content.trim(), params);
}

pub(crate) fn extract_destructure_params(value: &str, params: &mut FxHashSet<String>) {
    let mut pending = SmallVec::<[&str; 8]>::new();
    pending.push(value);

    while let Some(value) = pending.pop() {
        if value.starts_with('(') && value.ends_with(')') {
            pending.push(value[1..value.len() - 1].trim());
            continue;
        }
        if value.contains(',') && !value.starts_with('{') && !value.starts_with('[') {
            for part in split_top_level(value).into_iter().rev() {
                if part != value {
                    pending.push(part.trim());
                }
            }
            continue;
        }
        if value.starts_with('{') && value.ends_with('}') {
            for part in split_top_level(&value[1..value.len() - 1])
                .into_iter()
                .rev()
            {
                let part = part.trim();
                if let Some(rest) = part.strip_prefix("...") {
                    pending.push(rest.trim());
                    continue;
                }
                if let Some(eq_pos) = part.find('=') {
                    pending.push(part[..eq_pos].trim());
                    continue;
                }
                if let Some(colon_pos) = part.find(':') {
                    pending.push(part[colon_pos + 1..].trim());
                    continue;
                }
                pending.push(part);
            }
        } else if value.starts_with('[') && value.ends_with(']') {
            for part in split_top_level(&value[1..value.len() - 1])
                .into_iter()
                .rev()
            {
                let part = part.trim();
                if let Some(rest) = part.strip_prefix("...") {
                    pending.push(rest.trim());
                } else {
                    pending.push(part);
                }
            }
        } else if is_valid_identifier(value) {
            params.insert(value.to_compact_string());
        }
    }
}

fn split_top_level(value: &str) -> std::vec::Vec<&str> {
    let mut parts = std::vec::Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }

    parts.push(&value[start..]);
    parts
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

#[cfg(test)]
mod tests {
    use super::extract_destructure_params;
    use vize_s0::{FxHashSet, String};

    #[test]
    fn extracts_nested_params_and_rejects_unsplit_malformed_aliases() {
        let mut params = FxHashSet::default();
        extract_destructure_params(
            "{ id: itemId, nested: [first, ...rest], count = fallback }",
            &mut params,
        );
        extract_destructure_params("(index)", &mut params);

        for expected in ["itemId", "first", "rest", "count", "index"] {
            assert!(params.contains(expected), "missing {expected:?}");
        }
        assert_eq!(params.len(), 5);

        for malformed in ["$data.[label, value]", "(dep(, file)"] {
            let mut params = FxHashSet::default();
            extract_destructure_params(malformed, &mut params);
            assert!(params.is_empty(), "unexpected params for {malformed:?}");
        }
    }

    #[test]
    fn extract_destructure_params_handles_deep_nesting_on_a_small_stack() {
        let depth = 512;
        let mut pattern = String::with_capacity(depth * 4 + 5);
        for _ in 0..depth {
            pattern.push_str("{x:");
        }
        pattern.push_str("value");
        for _ in 0..depth {
            pattern.push('}');
        }

        let params = std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || {
                let mut params = FxHashSet::default();
                extract_destructure_params(&pattern, &mut params);
                params
            })
            .expect("spawn extraction thread")
            .join()
            .expect("extract parameters without overflowing the stack");

        assert_eq!(params.len(), 1);
        assert!(params.contains("value"));
    }
}
