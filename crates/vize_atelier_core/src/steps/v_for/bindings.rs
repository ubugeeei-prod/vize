//! Retain exceptional aliases' actual original parameter goal once.

use oxc_ast::ast::{BindingPattern, FormalParameters};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::{Allocator, Box, SmallVec, Vec, profiler::global_profiler};

use crate::{
    ExpressionNode, RetainedJsAst, RetainedJsAstKind, SimpleExpressionNode, SourceLocation,
};

pub(crate) fn alias_node<'a>(
    allocator: &'a Allocator,
    alias: &'a str,
) -> Option<ExpressionNode<'a>> {
    if alias.is_empty() {
        return None;
    }
    let mut node = SimpleExpressionNode::new(alias, false, SourceLocation::default());
    // Ordinary aliases without initializer/computed-key work retain their
    // existing zero-parse representation. Rewritable aliases keep the real
    // parameter carrier so their cooked names survive a content rewrite.
    let expression_candidate = alias.contains('=') || alias.contains('{') && alias.contains('[');
    let exceptional_alias = !alias.is_ascii() || alias.contains('\\');
    if exceptional_alias || expression_candidate {
        let mut names = Vec::new_in(&allocator);
        if vize_l0::expression_guard::expression_is_safe_to_parse(alias) {
            global_profiler().record_counter("atelier.for_binding.parses", 1);
            if let Some(parameters) = Parser::new(allocator.as_oxc(), alias, SourceType::ts())
                .parse_v_for_parameters()
                .ok()
                .filter(|parameters| {
                    alias
                        .get(parameters.span().end as usize..)
                        .is_some_and(vize_l0::expression_guard::is_expression_trailing_trivia)
                })
            {
                let mut pending = SmallVec::<[&BindingPattern<'_>; 8]>::new();
                if let Some(rest) = &parameters.rest {
                    pending.push(&rest.rest.argument);
                }
                for parameter in parameters.items.iter().rev() {
                    pending.push(&parameter.pattern);
                }
                while let Some(pattern) = pending.pop() {
                    match pattern {
                        BindingPattern::BindingIdentifier(identifier) => {
                            names.push(allocator.alloc_str(identifier.name.as_str()));
                        }
                        BindingPattern::ObjectPattern(object) => {
                            if let Some(rest) = &object.rest {
                                pending.push(&rest.argument);
                            }
                            for property in object.properties.iter().rev() {
                                pending.push(&property.value);
                            }
                        }
                        BindingPattern::ArrayPattern(array) => {
                            if let Some(rest) = &array.rest {
                                pending.push(&rest.argument);
                            }
                            for element in array.elements.iter().rev().flatten() {
                                pending.push(element);
                            }
                        }
                        BindingPattern::AssignmentPattern(assignment) => {
                            pending.push(&assignment.left)
                        }
                    }
                }
                drop(pending);
                if exceptional_alias || parameters_need_expression_rewrite(&parameters) {
                    // The existing parameter carrier retains the genuine goal;
                    // ForNode ownership distinguishes it from a v-slot goal.
                    node.js_ast = Some(RetainedJsAst {
                        ast: allocator
                            .as_oxc()
                            .alloc(RetainedJsAstKind::SlotBindings(Ok(parameters))),
                        raw: alias,
                    });
                    global_profiler().record_counter("atelier.for_binding.retained", 1);
                } else {
                    names.clear();
                }
            }
        }
        // A refused goal yields no partial names or raw-escape fallback. An
        // ordinary parsed candidate returns to the zero-parse representation.
        if exceptional_alias || node.js_ast.is_some() {
            node.identifiers = Some(names);
        }
    }
    Some(ExpressionNode::Simple(Box::new_in(node, &allocator)))
}

pub(crate) fn alias_needs_expression_rewrite(node: &SimpleExpressionNode<'_>) -> bool {
    node.retained_parameters()
        .is_some_and(parameters_need_expression_rewrite)
}

fn parameters_need_expression_rewrite(parameters: &FormalParameters<'_>) -> bool {
    parameters
        .items
        .iter()
        .any(|parameter| pattern_needs_expression_rewrite(&parameter.pattern))
        || parameters
            .rest
            .as_ref()
            .is_some_and(|rest| pattern_needs_expression_rewrite(&rest.rest.argument))
}

fn pattern_needs_expression_rewrite(pattern: &BindingPattern<'_>) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(_) => false,
        BindingPattern::AssignmentPattern(_) => true,
        BindingPattern::ObjectPattern(object) => {
            object.properties.iter().any(|property| {
                property.computed || pattern_needs_expression_rewrite(&property.value)
            }) || object
                .rest
                .as_ref()
                .is_some_and(|rest| pattern_needs_expression_rewrite(&rest.argument))
        }
        BindingPattern::ArrayPattern(array) => {
            array
                .elements
                .iter()
                .flatten()
                .any(pattern_needs_expression_rewrite)
                || array
                    .rest
                    .as_ref()
                    .is_some_and(|rest| pattern_needs_expression_rewrite(&rest.argument))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{alias_needs_expression_rewrite, alias_node};
    use crate::ExpressionNode;
    use vize_l0::Allocator;

    #[test]
    fn ordinary_aliases_keep_zero_parse_representation_and_original_bytes() {
        let allocator = Allocator::default();
        for alias in [
            "row",
            "{ label: row }",
            "[row, index]",
            "{ items: [row] }",
            "row: string",
        ] {
            let ExpressionNode::Simple(node) = alias_node(&allocator, alias).unwrap() else {
                unreachable!()
            };
            assert_eq!(node.content, alias);
            assert!(node.identifiers.is_none());
            assert!(node.js_ast.is_none());
            assert_eq!(node.loc.span, vize_l0::Span::new(0, 0));
        }
    }

    #[test]
    fn rewritable_ascii_aliases_retain_their_parameter_names() {
        let allocator = Allocator::default();
        for (alias, expected) in [
            ("[row = fallback, index]", vec!["row", "index"]),
            ("{ label = fallback, [key]: tag }", vec!["label", "tag"]),
        ] {
            let ExpressionNode::Simple(node) = alias_node(&allocator, alias).unwrap() else {
                unreachable!()
            };
            assert_eq!(node.content, alias);
            assert!(alias_needs_expression_rewrite(&node));
            assert_eq!(node.retained_parameter_names().unwrap(), expected);
        }
    }

    #[test]
    fn original_parameter_goal_retains_only_genuine_cooked_bindings() {
        let allocator = Allocator::default();
        for (alias, expected) in [
            (r"\u005frow", vec!["_row"]),
            (r"\u{5f}row", vec!["_row"]),
            ("行", vec!["行"]),
            (
                r"{ label: \u005frow, count = 1, ...rest }",
                vec!["_row", "count", "rest"],
            ),
            (
                r"{ value: \u005frow = ((nested) => nested)(1) }",
                vec!["_row"],
            ),
            (r"[\u005frow, , ...rest]", vec!["_row", "rest"]),
        ] {
            let ExpressionNode::Simple(node) = alias_node(&allocator, alias).unwrap() else {
                unreachable!()
            };
            assert_eq!(node.content, alias);
            assert_eq!(node.identifiers.as_ref().unwrap().as_slice(), expected);
            let retained = node.js_ast.unwrap();
            assert_eq!(retained.raw, alias);
            assert!(retained.as_expression().is_none());
            assert!(retained.as_slot_parameters().unwrap().is_ok());
        }
    }

    #[test]
    fn malformed_or_depth_refused_goals_never_leak_partial_names() {
        let allocator = Allocator::default();
        for alias in [r"\u005frow; injected", r"{ broken: \u005frow", r"\u00ZZrow"] {
            let ExpressionNode::Simple(node) = alias_node(&allocator, alias).unwrap() else {
                unreachable!()
            };
            assert!(node.identifiers.as_ref().unwrap().is_empty());
            assert!(node.js_ast.is_none());
        }
        let deep = ["[".repeat(33), r"\u005frow".to_owned(), "]".repeat(33)].concat();
        let ExpressionNode::Simple(node) = alias_node(&allocator, &deep).unwrap() else {
            unreachable!()
        };
        assert!(node.identifiers.as_ref().unwrap().is_empty());
        assert!(node.js_ast.is_none());
    }
}
