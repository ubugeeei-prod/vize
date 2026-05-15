//! Expression walking for scope discovery.
//!
//! Recursively walks expression nodes to find nested function scopes,
//! callback arguments, reactivity losses, and client-only lifecycle hooks.

use oxc_ast::ast::{Argument, AssignmentTarget, CallExpression, ObjectPropertyKind, Statement};

use super::{
    detect_call_argument_reactivity_loss, detect_provide_inject_call, detect_race_condition_call,
    extract_function_params, is_client_only_hook, walk_statement, ClientOnlyScopeData,
    ClosureScopeData, CompactString, Expression, ScriptParseResult,
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
                        walk_expression(result, &p.value, source);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        // Check for reactive spread: { ...state }
                        if let Expression::Identifier(id) = &spread.argument {
                            let var_name = CompactString::new(id.name.as_str());
                            if result.reactivity.is_reactive(var_name.as_str()) {
                                result.reactivity.record_spread(
                                    var_name,
                                    spread.span.start,
                                    spread.span.end,
                                );
                            }
                        }
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
            walk_expression(result, &assign.right, source);
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

/// Walk call expression arguments to find callbacks
#[inline]
pub(in crate::script_parser) fn walk_call_arguments(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    // First, walk the callee (might be a chained call like foo.bar().baz())
    walk_expression(result, &call.callee, source);

    // Check for provide/inject calls
    detect_provide_inject_call(result, call, source);
    detect_race_condition_call(result, call, source);
    detect_call_argument_reactivity_loss(result, call, source);

    // Check if this is a client-only lifecycle hook
    let is_lifecycle_hook = if let Expression::Identifier(id) = &call.callee {
        is_client_only_hook(id.name.as_str())
    } else {
        false
    };

    let hook_name = if is_lifecycle_hook {
        if let Expression::Identifier(id) = &call.callee {
            Some(id.name.as_str())
        } else {
            None
        }
    } else {
        None
    };

    // Then walk each argument
    for arg in call.arguments.iter() {
        match arg {
            Argument::SpreadElement(spread) => {
                walk_expression(result, &spread.argument, source);
            }
            _ => {
                if let Some(expr) = arg.as_expression() {
                    // If this is a lifecycle hook and the argument is a function,
                    // wrap it in a ClientOnly scope
                    if let Some(name) = hook_name {
                        match expr {
                            Expression::ArrowFunctionExpression(arrow) => {
                                // Enter client-only scope
                                result.scopes.enter_client_only_scope(
                                    ClientOnlyScopeData {
                                        hook_name: CompactString::new(name),
                                    },
                                    call.span.start,
                                    call.span.end,
                                );

                                // Now create the closure scope inside the client-only scope
                                let params = extract_function_params(&arrow.params);
                                result.scopes.enter_closure_scope(
                                    ClosureScopeData {
                                        name: None,
                                        param_names: params,
                                        is_arrow: true,
                                        is_async: arrow.r#async,
                                        is_generator: false,
                                    },
                                    arrow.span.start,
                                    arrow.span.end,
                                );

                                // Walk the body
                                if arrow.expression {
                                    if let Some(Statement::ExpressionStatement(expr_stmt)) =
                                        arrow.body.statements.first()
                                    {
                                        walk_expression(result, &expr_stmt.expression, source);
                                    }
                                } else {
                                    for stmt in arrow.body.statements.iter() {
                                        walk_statement(result, stmt, source);
                                    }
                                }

                                result.scopes.exit_scope(); // Exit closure scope
                                result.scopes.exit_scope(); // Exit client-only scope
                                continue;
                            }
                            Expression::FunctionExpression(func) => {
                                // Enter client-only scope
                                result.scopes.enter_client_only_scope(
                                    ClientOnlyScopeData {
                                        hook_name: CompactString::new(name),
                                    },
                                    call.span.start,
                                    call.span.end,
                                );

                                // Create closure scope inside client-only scope
                                let params = extract_function_params(&func.params);
                                let fn_name = func
                                    .id
                                    .as_ref()
                                    .map(|id| CompactString::new(id.name.as_str()));

                                result.scopes.enter_closure_scope(
                                    ClosureScopeData {
                                        name: fn_name,
                                        param_names: params,
                                        is_arrow: false,
                                        is_async: func.r#async,
                                        is_generator: func.generator,
                                    },
                                    func.span.start,
                                    func.span.end,
                                );

                                if let Some(body) = &func.body {
                                    for stmt in body.statements.iter() {
                                        walk_statement(result, stmt, source);
                                    }
                                }

                                result.scopes.exit_scope(); // Exit closure scope
                                result.scopes.exit_scope(); // Exit client-only scope
                                continue;
                            }
                            _ => {}
                        }
                    }
                    walk_expression(result, expr, source);
                }
            }
        }
    }
}
