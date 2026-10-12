//! Inline compiler-owned runtime props, using the existing setup block parse.

use oxc_ast::ast::{CallExpression, Expression, Program, Statement};

use super::super::compiler_macro_bindings::shadowed;
use super::{ScriptLintResult, check_props_object};

const MACRO: &str = "defineProps";

pub(super) fn check(program: &Program<'_>, offset: usize, result: &mut ScriptLintResult) {
    let mut compiler_owned = None;
    for statement in &program.body {
        match statement {
            Statement::ExpressionStatement(statement) => {
                check_expression(
                    &statement.expression,
                    program,
                    &mut compiler_owned,
                    offset,
                    result,
                );
            }
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    if let Some(initializer) = &declarator.init {
                        check_expression(initializer, program, &mut compiler_owned, offset, result);
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_expression(
    expression: &Expression<'_>,
    program: &Program<'_>,
    compiler_owned: &mut Option<bool>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return;
    };
    check_call(call, program, compiler_owned, offset, result);
}

fn check_call(
    call: &CallExpression<'_>,
    program: &Program<'_>,
    compiler_owned: &mut Option<bool>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    if call.optional
        || call.type_arguments.is_some()
        || !matches!(&call.callee, Expression::Identifier(id) if id.name == MACRO)
    {
        return;
    }
    let Some(Expression::ObjectExpression(props)) = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
        .map(Expression::get_inner_expression)
    else {
        return;
    };
    // Existing typed calls never pay for a second ownership scan. A runtime
    // object establishes the need once and all direct calls share the result.
    if *compiler_owned.get_or_insert_with(|| !shadowed(program, MACRO)) {
        check_props_object(props, true, offset, result);
    }
}
