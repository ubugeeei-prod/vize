//! Scope-variable extraction helpers used by the lint visitor.
//!
//! Parses `v-for` and `v-slot` expressions to collect the variable names they
//! introduce into the template scope, so downstream rules can distinguish
//! template-local bindings from unresolved identifiers.

use vize_carton::CompactString;
use vize_relief::ExpressionNode;

/// Parse v-for expression to extract variable names.
///
/// Uses CompactString for efficient small string storage.
///
/// Handles formats like:
/// - `item in items`
/// - `(item, index) in items`
/// - `(value, key, index) in object`
#[inline]
pub fn parse_v_for_variables(exp: &ExpressionNode) -> Vec<CompactString> {
    let content = match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(_) => return Vec::new(),
    };

    // Split on " in " or " of " - use byte search for speed
    let bytes = content.as_bytes();
    let (alias_part, _) = if let Some(idx) = find_pattern(bytes, b" in ") {
        (&content[..idx], &content[idx + 4..])
    } else if let Some(idx) = find_pattern(bytes, b" of ") {
        (&content[..idx], &content[idx + 4..])
    } else {
        return Vec::new();
    };

    let alias_str = alias_part.trim();

    parse_binding_variables(alias_str)
}

/// Parse a scoped slot expression to extract variable names.
#[inline]
pub fn parse_slot_scope_variables(exp: &ExpressionNode) -> Vec<CompactString> {
    let content = match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(_) => return Vec::new(),
    };

    parse_binding_variables(content.trim())
}

fn parse_binding_variables(alias_str: &str) -> Vec<CompactString> {
    if alias_str.is_empty() {
        return Vec::new();
    }

    // Handle destructuring: (item, index), { id, name }, or [first, second]
    let is_tuple = alias_str.starts_with('(') && alias_str.ends_with(')');
    let is_object = alias_str.starts_with('{') && alias_str.ends_with('}');
    let is_array = alias_str.starts_with('[') && alias_str.ends_with(']');

    if is_tuple || is_object || is_array {
        let inner = &alias_str[1..alias_str.len() - 1];
        // Pre-allocate with estimated capacity
        let mut vars = Vec::with_capacity(3);
        for s in inner.split(',') {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Handle object shorthand: { id } -> id, { id: itemId } -> itemId
            if is_object {
                if let Some(colon_idx) = trimmed.find(':') {
                    // { id: itemId } -> itemId
                    let value_part = trimmed[colon_idx + 1..].trim();
                    if let Some(name) = normalize_binding_name(value_part) {
                        vars.push(CompactString::from(name));
                    }
                } else {
                    // { id } -> id (shorthand)
                    if let Some(name) = normalize_binding_name(trimmed) {
                        vars.push(CompactString::from(name));
                    }
                }
            } else {
                if let Some(name) = normalize_binding_name(trimmed) {
                    vars.push(CompactString::from(name));
                }
            }
        }
        vars
    } else {
        // Single variable - avoid allocation if possible
        normalize_binding_name(alias_str)
            .map(|name| vec![CompactString::from(name)])
            .unwrap_or_default()
    }
}

fn normalize_binding_name(binding: &str) -> Option<&str> {
    let binding = binding.trim().trim_start_matches("...").trim();
    let binding = binding
        .split_once('=')
        .map(|(name, _)| name.trim())
        .unwrap_or(binding);

    (!binding.is_empty()).then_some(binding)
}

/// Fast byte pattern search
#[inline]
fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::{CompactString, ExpressionNode, parse_slot_scope_variables, parse_v_for_variables};
    use vize_carton::Bump;
    use vize_relief::SimpleExpressionNode;

    fn make_simple_exp<'a>(allocator: &'a Bump, content: &str) -> ExpressionNode<'a> {
        ExpressionNode::Simple(vize_carton::Box::new_in(
            SimpleExpressionNode::new(
                vize_carton::String::from(content),
                false,
                vize_relief::SourceLocation::STUB,
            ),
            allocator,
        ))
    }

    #[test]
    fn test_parse_v_for_simple() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "item in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(vars, vec![CompactString::from("item")]);
    }

    #[test]
    fn test_parse_v_for_with_index() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "(item, index) in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("item"), CompactString::from("index")]
        );
    }

    #[test]
    fn test_parse_v_for_object() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "(value, key, index) in object");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![
                CompactString::from("value"),
                CompactString::from("key"),
                CompactString::from("index"),
            ]
        );
    }

    #[test]
    fn test_parse_v_for_object_destructuring() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ id } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(vars, vec![CompactString::from("id")]);
    }

    #[test]
    fn test_parse_v_for_object_destructuring_multiple() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ id, name } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("id"), CompactString::from("name")]
        );
    }

    #[test]
    fn test_parse_v_for_object_destructuring_with_rename() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ id: itemId, name: itemName } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![
                CompactString::from("itemId"),
                CompactString::from("itemName")
            ]
        );
    }

    #[test]
    fn test_parse_v_for_array_destructuring() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "[first, second] in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("first"), CompactString::from("second")]
        );
    }

    #[test]
    fn test_parse_slot_scope_object_destructuring() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ open, item: slotItem }");
        let vars = parse_slot_scope_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("open"), CompactString::from("slotItem")]
        );
    }

    #[test]
    fn test_parse_slot_scope_default_and_rest_bindings() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ open = false, ...rest }");
        let vars = parse_slot_scope_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("open"), CompactString::from("rest")]
        );
    }
}
