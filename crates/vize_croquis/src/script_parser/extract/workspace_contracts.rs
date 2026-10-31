//! The existing setup walk's event/slot projection, without unrelated analysis.

use oxc_ast::ast::{Argument, BindingPattern, CallExpression, Expression, Program, Statement};

use super::{
    extract_call_expression, process_call_expression, record_static_runtime_object_literal,
};
use crate::macros::{MacroKind, MacroTracker};
use crate::script_parser::ScriptParseResult;

pub(in crate::script_parser) fn workspace_symbol_contracts(
    program: &Program<'_>,
    source: &str,
) -> MacroTracker {
    // Existing emit/slot extractors read only macros and runtime_object_literals.
    // Keep their carrier and leaf implementations; no setup/global hierarchy,
    // lexical walk, type resolution, or reactivity analysis is requested here.
    // Default still constructs the universal root, so this is not allocation-free.
    let mut result = ScriptParseResult::default();
    for statement in &program.body {
        match statement {
            Statement::ExpressionStatement(statement) => {
                // Unlike initializer calls, the original expression-statement
                // path does not unwrap parentheses or TypeScript wrappers.
                if let Expression::CallExpression(call) = &statement.expression {
                    visit_call(&mut result, call, source);
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    match &declarator.id {
                        BindingPattern::BindingIdentifier(identifier) => {
                            if let Some(call) =
                                declarator.init.as_ref().and_then(extract_call_expression)
                            {
                                visit_call(&mut result, call, source);
                            }
                            if let Some(expression) = &declarator.init {
                                // The full walk records these in source order,
                                // including let/var and object aliases/spreads.
                                // Calls cannot form a cached object expression.
                                record_static_runtime_object_literal(
                                    &mut result,
                                    identifier.name.as_str(),
                                    expression,
                                    declaration.kind,
                                    source,
                                );
                            }
                        }
                        BindingPattern::ArrayPattern(_) => {
                            // The defineModel destructure helper dispatches any
                            // initializer macro before checking its returned kind.
                            if let Some(call) =
                                declarator.init.as_ref().and_then(extract_call_expression)
                            {
                                visit_call(&mut result, call, source);
                            }
                        }
                        // Object patterns dispatch only direct defineProps or
                        // withDefaults(defineProps(...)); neither yields symbols.
                        _ => {}
                    }
                }
            }
            // The original nested statement/expression walk does not dispatch
            // macro calls. Imports, exports, functions, blocks, and callbacks
            // must not become new event/slot producers in this projection.
            _ => {}
        }
    }
    result.macros
}

fn visit_call(result: &mut ScriptParseResult, call: &CallExpression<'_>, source: &str) {
    let Expression::Identifier(identifier) = &call.callee else {
        return;
    };
    match MacroKind::from_name(identifier.name.as_str()) {
        Some(MacroKind::DefineEmits | MacroKind::DefineSlots) => {
            // Preserve the existing owned names, declaration spans, duplicate
            // ordering, signatures, annotations, and slot type argument range.
            process_call_expression(result, call, source);
        }
        Some(MacroKind::WithDefaults) => {
            // The existing dispatch recursively processes any direct inner call,
            // including malformed withDefaults(defineEmits(...), ...).
            if let Some(Argument::CallExpression(inner)) = call.arguments.first() {
                visit_call(result, inner, source);
            }
        }
        _ => {}
    }
}
