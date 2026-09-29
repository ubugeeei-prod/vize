//! Expression walking for scope discovery.
//!
//! Recursively walks expression nodes to find nested function scopes,
//! callback arguments, reactivity losses, and client-only lifecycle hooks.

mod calls;
pub(in crate::script_parser) use calls::walk_call_arguments;
use calls::{identifier_might_be_browser_global, note_script_browser_global};

use oxc_ast::ast::{Argument, AssignmentTarget, CallExpression, ObjectPropertyKind, Statement};

use super::{
    ClientOnlyScopeData, ClosureScopeData, CompactString, Expression, ScriptParseResult,
    detect_call_argument_reactivity_loss, detect_provide_inject_call, detect_race_condition_call,
    extract_function_params, is_client_only_hook, walk_statement,
};

/// Walk an expression to find nested scopes (arrow functions, callbacks, etc.)
///
/// This is called recursively to build the scope chain for the script.
/// Performance: Only walks into expressions that might contain function scopes.
#[inline]
pub(in crate::script_parser) fn walk_expression(
    result: &mut ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) {
    match expr {
        // Arrow functions create closure scopes (no `arguments`, no `this` binding)
        Expression::ArrowFunctionExpression(arrow) => {
            let params = extract_function_params(&arrow.params);

            result.scopes.enter_closure_scope(
                ClosureScopeData {
                    name: None,
                    param_names: params,
                    is_arrow: true,
                    is_async: arrow.r#async,
                    is_generator: false, // Arrow functions cannot be generators
                },
                arrow.span.start,
                arrow.span.end,
            );

            // Walk the body for nested scopes
            // Arrow function body is always a FunctionBody (not a variant)
            // but may have expression property set for concise arrows
            if arrow.expression {
                // Concise arrow: () => expr
                // The expression is the first statement's expression
                if let Some(Statement::ExpressionStatement(expr_stmt)) =
                    arrow.body.statements.first()
                {
                    walk_expression(result, &expr_stmt.expression, source);
                }
            } else {
                // Block arrow: () => { ... }
                for stmt in arrow.body.statements.iter() {
                    walk_statement(result, stmt, source);
                }
            }

            result.scopes.exit_scope();
        }

        // Function expressions create closure scopes
        Expression::FunctionExpression(func) => {
            let params = extract_function_params(&func.params);
            let name = func
                .id
                .as_ref()
                .map(|id| CompactString::new(id.name.as_str()));

            result.scopes.enter_closure_scope(
                ClosureScopeData {
                    name,
                    param_names: params,
                    is_arrow: false,
                    is_async: func.r#async,
                    is_generator: func.generator,
                },
                func.span.start,
                func.span.end,
            );

            // Walk the body for nested scopes
            if let Some(body) = &func.body {
                for stmt in body.statements.iter() {
                    walk_statement(result, stmt, source);
                }
            }

            result.scopes.exit_scope();
        }

        // Call expressions may contain callbacks as arguments
        Expression::CallExpression(call) => {
            walk_call_arguments(result, call, source);
        }

        Expression::Identifier(id) => {
            if identifier_might_be_browser_global(id.name.as_str()) {
                note_script_browser_global(result, id.name.as_str(), id.span.start);
            }
        }

        // Member expressions - walk the object
        Expression::StaticMemberExpression(member) => {
            walk_expression(result, &member.object, source);
        }
        Expression::ComputedMemberExpression(member) => {
            walk_expression(result, &member.object, source);
            walk_expression(result, &member.expression, source);
        }

        // Chained expressions
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                walk_call_arguments(result, call, source);
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                walk_expression(result, &expr.expression, source);
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                walk_expression(result, &member.object, source);
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                walk_expression(result, &member.object, source);
                walk_expression(result, &member.expression, source);
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                walk_expression(result, &field.object, source);
            }
        },

        // Conditional expression
        Expression::ConditionalExpression(cond) => {
            walk_expression(result, &cond.test, source);
            walk_expression(result, &cond.consequent, source);
            walk_expression(result, &cond.alternate, source);
        }

        // Logical/Binary expressions
        Expression::LogicalExpression(logical) => {
            walk_expression(result, &logical.left, source);
            walk_expression(result, &logical.right, source);
        }
        Expression::BinaryExpression(binary) => {
            walk_expression(result, &binary.left, source);
            walk_expression(result, &binary.right, source);
        }

        // Array/Object expressions
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                match elem {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                        super::super::extract::check_reactive_spread_expression(
                            result,
                            &spread.argument,
                            source,
                            spread.span.start,
                            spread.span.end,
                        );
                        walk_expression(result, &spread.argument, source);
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(expr) = elem.as_expression() {
                            walk_expression(result, expr, source);
                        }
                    }
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    ObjectPropertyKind::ObjectProperty(p) => {
                        if let Some(key) = p.key.as_expression() {
                            walk_expression(result, key, source);
                        }
                        walk_expression(result, &p.value, source);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        super::super::extract::check_reactive_spread_expression(
                            result,
                            &spread.argument,
                            source,
                            spread.span.start,
                            spread.span.end,
                        );
                        walk_expression(result, &spread.argument, source);
                    }
                }
            }
        }

        // Await/Unary
        Expression::AwaitExpression(await_expr) => {
            walk_expression(result, &await_expr.argument, source);
        }
        Expression::UnaryExpression(unary) => {
            walk_expression(result, &unary.argument, source);
        }

        // Sequence expression
        Expression::SequenceExpression(seq) => {
            for expr in seq.expressions.iter() {
                walk_expression(result, expr, source);
            }
        }

        // Parenthesized
        Expression::ParenthesizedExpression(paren) => {
            walk_expression(result, &paren.expression, source);
        }

        // Assignment
        Expression::AssignmentExpression(assign) => {
            super::super::extract::check_reactive_plain_assignment_mutation(
                result,
                &assign.left,
                source,
            );

            // Check for reactive variable reassignment: state = newValue
            if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
                let var_name = CompactString::new(id.name.as_str());
                if result.reactivity.is_reactive(var_name.as_str()) {
                    // Use id.span for the variable name, assign.span for the full expression
                    result
                        .reactivity
                        .record_reassign(var_name, id.span.start, assign.span.end);
                } else {
                    super::super::extract::check_reactive_plain_assignment_alias(
                        result,
                        id.name.as_str(),
                        &assign.right,
                    );
                }
            }
            if let Some(root) = super::super::extract::member_assignment_root(&assign.left) {
                let previous = result.reactive_assignment_root.replace(root);
                walk_expression(result, &assign.right, source);
                result.reactive_assignment_root = previous;
            } else {
                walk_expression(result, &assign.right, source);
            }
        }

        Expression::UpdateExpression(update) => {
            super::super::extract::check_reactive_plain_update_mutation(
                result,
                &update.argument,
                source,
            );
        }

        // TypeScript type assertions (as, satisfies, !)
        Expression::TSAsExpression(ts_as) => {
            walk_expression(result, &ts_as.expression, source);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            walk_expression(result, &ts_satisfies.expression, source);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            walk_expression(result, &ts_non_null.expression, source);
        }

        // Other expressions don't need walking for scopes
        _ => {}
    }
}
