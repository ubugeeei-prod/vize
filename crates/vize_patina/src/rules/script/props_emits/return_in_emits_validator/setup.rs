//! Compiler-owned runtime validators in actual SFC script-setup blocks.
//!
//! This rule-specific resolver deliberately leaves the shared Options resolver
//! and standalone ScriptRule API unchanged. Both programs use the existing
//! script pass's parse; a malformed sibling never proves absence of shadowing.

use oxc_ast::ast::{
    BindingPattern, CallExpression, Declaration, Expression, Program, Statement,
    VariableDeclarationKind,
};

use super::super::compiler_macro_bindings::{
    binds, declaration, declaration_binds, default_export_binds, hoisted_var_binds, import_binds,
    shadowed,
};
use super::{ScriptLintResult, check_emits_object};

/// Ownership evidence for the normal script block, distinct from its absence.
#[derive(Clone, Copy)]
pub enum EmitsModuleScript<'a> {
    Absent,
    Unavailable,
    Parsed {
        program: &'a Program<'a>,
        offset: usize,
    },
}

const MACRO: &str = "defineEmits";

/// Inspect the already parsed setup AST before requesting sibling evidence.
pub(crate) fn has_candidate(program: &Program<'_>) -> bool {
    !shadowed(program, MACRO)
        && program.body.iter().any(|statement| {
            for_each_call(statement, |call| {
                is_macro_call(call) && call.type_arguments.is_none() && !call.arguments.is_empty()
            })
        })
}

pub(super) fn check<'a>(
    program: &'a Program<'a>,
    offset: usize,
    module: EmitsModuleScript<'a>,
    result: &mut ScriptLintResult,
) {
    if shadowed(program, MACRO) {
        return;
    }
    match module {
        EmitsModuleScript::Unavailable => return,
        EmitsModuleScript::Parsed { program, .. } if shadowed(program, MACRO) => return,
        _ => {}
    }
    for statement in &program.body {
        for_each_call(statement, |call| {
            if is_macro_call(call)
                && call.type_arguments.is_none()
                && let Some(argument) = call.arguments.first().and_then(|a| a.as_expression())
            {
                check_object(argument, program, offset, module, 0, result);
            }
            false
        });
    }
}

// Vue's compiler examines direct expression statements and non-ambient
// variable initializers, unwrapping parentheses and TypeScript assertions.
fn for_each_call<'a>(
    statement: &'a Statement<'a>,
    mut inspect: impl FnMut(&'a CallExpression<'a>) -> bool,
) -> bool {
    match statement {
        Statement::ExpressionStatement(statement) => {
            match statement.expression.get_inner_expression() {
                Expression::CallExpression(call) => inspect(call),
                _ => false,
            }
        }
        Statement::VariableDeclaration(declaration) if !declaration.declare => {
            declaration.declarations.iter().any(|declarator| {
                match declarator
                    .init
                    .as_ref()
                    .map(Expression::get_inner_expression)
                {
                    Some(Expression::CallExpression(call)) => inspect(call),
                    _ => false,
                }
            })
        }
        _ => false,
    }
}

fn is_macro_call(call: &CallExpression<'_>) -> bool {
    !call.optional && matches!(&call.callee, Expression::Identifier(id) if id.name == MACRO)
}

fn check_object<'a>(
    expression: &'a Expression<'a>,
    program: &'a Program<'a>,
    offset: usize,
    module: EmitsModuleScript<'a>,
    depth: usize,
    result: &mut ScriptLintResult,
) {
    // A fixed bound terminates cycles without allocating a name set. Resolution
    // never evaluates calls, follows mutable bindings, or imports dependencies.
    if depth >= 16 {
        return;
    }
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => check_emits_object(object, offset, result),
        Expression::Identifier(identifier) => {
            match binding(program, identifier.name.as_str()) {
                Binding::Constant(initializer) => {
                    check_object(initializer, program, offset, module, depth + 1, result);
                }
                Binding::Absent => {
                    if let EmitsModuleScript::Parsed { program, offset } = module
                        && let Binding::Constant(initializer) =
                            binding(program, identifier.name.as_str())
                    {
                        // A normal-script initializer resolves in its own
                        // module scope and retains its original SFC offset.
                        check_object(
                            initializer,
                            program,
                            offset,
                            EmitsModuleScript::Absent,
                            depth + 1,
                            result,
                        );
                    }
                }
                Binding::Other => {}
            }
        }
        _ => {}
    }
}

enum Binding<'a> {
    Absent,
    Constant(&'a Expression<'a>),
    Other,
}

fn binding<'a>(program: &'a Program<'a>, name: &str) -> Binding<'a> {
    for statement in &program.body {
        if let Some(declaration) = declaration(statement) {
            if let Declaration::VariableDeclaration(declaration) = declaration {
                if !declaration.declare {
                    for declarator in &declaration.declarations {
                        if binds(&declarator.id, name) {
                            return match (&declarator.id, &declarator.init) {
                                (BindingPattern::BindingIdentifier(_), Some(initializer))
                                    if declaration.kind == VariableDeclarationKind::Const =>
                                {
                                    Binding::Constant(initializer)
                                }
                                _ => Binding::Other,
                            };
                        }
                    }
                }
            } else if declaration_binds(declaration, name) {
                return Binding::Other;
            }
        }
        if import_binds(statement, name, false) || default_export_binds(statement, name) {
            return Binding::Other;
        }
    }
    if hoisted_var_binds(program, name) {
        Binding::Other
    } else {
        Binding::Absent
    }
}
