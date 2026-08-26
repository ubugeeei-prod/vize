//! Scope-variable extraction helpers used by the lint visitor.
//!
//! Parses `v-for` and `v-slot` expressions to collect the variable names they
//! introduce into the template scope, so downstream rules can distinguish
//! template-local bindings from unresolved identifiers.

use vize_croquis::drawer::{extract_slot_props, parse_v_for_expression};
use vize_relief::ExpressionNode;
use vize_s0::CompactString;

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
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(_) => return Vec::new(),
    };

    parse_v_for_expression(content)
        .0
        .into_iter()
        .collect::<Vec<_>>()
}

/// Parse a scoped slot expression to extract variable names.
#[inline]
pub fn parse_slot_scope_variables(exp: &ExpressionNode) -> Vec<CompactString> {
    let content = match exp {
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(_) => return Vec::new(),
    };

    extract_slot_props(content.trim()).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{CompactString, ExpressionNode, parse_slot_scope_variables, parse_v_for_variables};
    use vize_relief::SimpleExpressionNode;
    use vize_s0::Allocator;

    fn make_simple_exp<'a>(allocator: &'a Allocator, content: &'a str) -> ExpressionNode<'a> {
        ExpressionNode::Simple(vize_s0::Box::new_in(
            SimpleExpressionNode::new(content, false, vize_relief::SourceLocation::STUB),
            &allocator,
        ))
    }

    #[test]
    fn test_parse_v_for_simple() {
        let allocator = Allocator::new();
        let exp = make_simple_exp(&allocator, "item in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(vars, vec![CompactString::from("item")]);
    }

    #[test]
    fn test_parse_v_for_with_index() {
        let allocator = Allocator::new();
        let exp = make_simple_exp(&allocator, "(item, index) in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("item"), CompactString::from("index")]
        );
    }

    #[test]
    fn test_parse_v_for_object() {
        let allocator = Allocator::new();
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
        let allocator = Allocator::new();
        let exp = make_simple_exp(&allocator, "{ id } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(vars, vec![CompactString::from("id")]);
    }

    #[test]
    fn test_parse_v_for_object_destructuring_multiple() {
        let allocator = Allocator::new();
        let exp = make_simple_exp(&allocator, "{ id, name } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("id"), CompactString::from("name")]
        );
    }

    #[test]
    fn test_parse_v_for_object_destructuring_with_rename() {
        let allocator = Allocator::new();
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
        let allocator = Allocator::new();
        let exp = make_simple_exp(&allocator, "[first, second] in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("first"), CompactString::from("second")]
        );
    }

    #[test]
    fn test_parse_slot_scope_object_destructuring() {
        let allocator = Allocator::new();
        let exp = make_simple_exp(&allocator, "{ open, item: slotItem }");
        let vars = parse_slot_scope_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("open"), CompactString::from("slotItem")]
        );
    }

    #[test]
    fn test_parse_slot_scope_default_and_rest_bindings() {
        let allocator = Allocator::new();
        let exp = make_simple_exp(&allocator, "{ open = false, ...rest }");
        let vars = parse_slot_scope_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("open"), CompactString::from("rest")]
        );
    }
}
