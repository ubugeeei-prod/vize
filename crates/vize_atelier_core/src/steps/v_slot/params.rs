//! Slot parameter binding extraction.

use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{SmallVec, String};

use crate::steps::expression::expression_is_safe_to_parse;

/// Extract binding names from a slot props pattern.
pub fn extract_slot_prop_names(pattern: &str) -> Vec<String> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() || !expression_is_safe_to_parse(trimmed) {
        return Vec::new();
    }

    let mut source = String::with_capacity(trimmed.len() + 18);
    source.push_str("let ");
    source.push_str(trimmed);
    source.push_str(" = __slotProps");

    let allocator = crate::expr_parse_probe::parse_arena();
    let source_type = SourceType::default().with_typescript(true);
    let parsed = Parser::new(&allocator, source.as_str(), source_type).parse();

    let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = parsed.program.body.first()
    else {
        return Vec::new();
    };

    let Some(declarator) = var_decl.declarations.first() else {
        return Vec::new();
    };

    let mut names = Vec::new();
    collect_slot_binding_names(&declarator.id, &mut names);
    names
}

fn collect_slot_binding_names(pattern: &BindingPattern<'_>, names: &mut Vec<String>) {
    let mut pending = SmallVec::<[&BindingPattern<'_>; 8]>::new();
    pending.push(pattern);

    while let Some(pattern) = pending.pop() {
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                names.push(String::new(id.name.as_str()));
            }
            BindingPattern::ObjectPattern(obj) => {
                if let Some(rest) = &obj.rest {
                    pending.push(&rest.argument);
                }
                for prop in obj.properties.iter().rev() {
                    pending.push(&prop.value);
                }
            }
            BindingPattern::ArrayPattern(arr) => {
                if let Some(rest) = &arr.rest {
                    pending.push(&rest.argument);
                }
                for elem in arr.elements.iter().rev().flatten() {
                    pending.push(elem);
                }
            }
            BindingPattern::AssignmentPattern(assign) => {
                pending.push(&assign.left);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steps::expression::MAX_EXPRESSION_NESTING_DEPTH;

    #[test]
    fn extracts_simple_destructure() {
        let names = extract_slot_prop_names("{ item, index }");
        let names: Vec<_> = names.iter().map(|name| name.as_str()).collect();
        assert_eq!(names, vec!["item", "index"]);
    }

    #[test]
    fn extracts_nested_defaults_and_rest_in_source_order() {
        let names = extract_slot_prop_names("{ item: { id }, index = 0, ...rest }");
        let names: Vec<_> = names.iter().map(|name| name.as_str()).collect();
        assert_eq!(names, vec!["id", "index", "rest"]);
    }

    #[test]
    fn accepts_patterns_at_the_nesting_limit() {
        let pattern = nested_pattern(MAX_EXPRESSION_NESTING_DEPTH);
        let names = extract_slot_prop_names(&pattern);
        let names: Vec<_> = names.iter().map(|name| name.as_str()).collect();
        assert_eq!(names, ["value"]);
    }

    #[test]
    fn rejects_deep_nesting_on_a_small_stack() {
        let pattern = nested_pattern(512);
        let names = std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || extract_slot_prop_names(&pattern))
            .expect("spawn slot binding extraction thread")
            .join()
            .expect("reject nested slot bindings without overflowing the stack");

        assert!(names.is_empty());
    }

    fn nested_pattern(depth: usize) -> String {
        let mut pattern = String::with_capacity(depth * 4 + 5);
        for _ in 0..depth {
            pattern.push_str("{x:");
        }
        pattern.push_str("value");
        for _ in 0..depth {
            pattern.push('}');
        }
        pattern
    }
}
