//! Scope-variable extraction helpers used by the lint visitor.
//!
//! Parses `v-for` and `v-slot` expressions to collect the variable names they
//! introduce into the template scope, so downstream rules can distinguish
//! template-local bindings from unresolved identifiers.

use oxc_ast::ast::BindingPattern;
use vize_croquis::drawer::parse_v_for_expression;
use vize_l0::CompactString;
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
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(_) => return Vec::new(),
    };

    parse_v_for_expression(content)
        .0
        .into_iter()
        .collect::<Vec<_>>()
}

/// Read declarations from the scoped slot's original retained parameter goal.
#[inline]
pub fn parse_slot_scope_variables(exp: &ExpressionNode) -> Vec<CompactString> {
    let ExpressionNode::Simple(node) = exp else {
        return Vec::new();
    };
    let Some(retained) = node.js_ast else {
        return Vec::new();
    };
    if retained.raw != node.content {
        return Vec::new();
    }
    let Some(Ok(parameters)) = retained.as_slot_parameters() else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for parameter in &parameters.items {
        collect_slot_bindings(&parameter.pattern, &mut names);
    }
    if let Some(rest) = &parameters.rest {
        collect_slot_bindings(&rest.rest.argument, &mut names);
    }
    names
}

fn collect_slot_bindings(pattern: &BindingPattern<'_>, names: &mut Vec<CompactString>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.push(identifier.name.as_str().into())
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_slot_bindings(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_slot_bindings(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_slot_bindings(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_slot_bindings(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(default) => collect_slot_bindings(&default.left, names),
    }
}

#[cfg(test)]
mod tests {
    use super::{CompactString, ExpressionNode, parse_slot_scope_variables, parse_v_for_variables};
    use vize_l0::Allocator;
    use vize_relief::SimpleExpressionNode;

    fn make_simple_exp<'a>(allocator: &'a Allocator, content: &'a str) -> ExpressionNode<'a> {
        ExpressionNode::Simple(vize_l0::Box::new_in(
            SimpleExpressionNode::new(content, false, vize_relief::SourceLocation::STUB),
            &allocator,
        ))
    }

    fn make_slot_exp<'a>(allocator: &'a Allocator, content: &str) -> ExpressionNode<'a> {
        let source = format!("<Panel v-slot=\"{content}\" />");
        let source = allocator.as_oxc().alloc_str(&source);
        let (mut root, errors) = vize_armature::parse(allocator, source);
        assert!(errors.is_empty());
        let vize_relief::TemplateChildNode::Element(element) = &mut root.children[0] else {
            panic!("original slot owner");
        };
        let vize_relief::PropNode::Directive(directive) = &mut element.props[0] else {
            panic!("original slot directive");
        };
        directive.exp.take().expect("original slot value")
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
        let exp = make_slot_exp(&allocator, "{ open, item: slotItem }");
        let vars = parse_slot_scope_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("open"), CompactString::from("slotItem")]
        );
    }

    #[test]
    fn test_parse_slot_scope_default_and_rest_bindings() {
        let allocator = Allocator::new();
        let exp = make_slot_exp(&allocator, "{ open = false, ...rest }");
        let vars = parse_slot_scope_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("open"), CompactString::from("rest")]
        );
    }
}
