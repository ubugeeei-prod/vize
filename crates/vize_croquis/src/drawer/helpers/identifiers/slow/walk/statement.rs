use super::super::super::IdentifierRef;
use super::walk_expr;

pub(in crate::drawer::helpers::identifiers::slow) fn walk_program(
    program: &oxc_ast::ast::Program<'_>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    for statement in program.body.iter() {
        walk_statement(statement, identifiers);
    }
}

fn walk_statement(statement: &oxc_ast::ast::Statement<'_>, identifiers: &mut Vec<IdentifierRef>) {
    match statement {
        oxc_ast::ast::Statement::BlockStatement(block) => {
            for statement in block.body.iter() {
                walk_statement(statement, identifiers);
            }
        }
        oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) => {
            walk_expr(&expr_stmt.expression, identifiers);
        }
        oxc_ast::ast::Statement::IfStatement(if_stmt) => {
            walk_expr(&if_stmt.test, identifiers);
            walk_statement(&if_stmt.consequent, identifiers);
            if let Some(alternate) = &if_stmt.alternate {
                walk_statement(alternate, identifiers);
            }
        }
        oxc_ast::ast::Statement::VariableDeclaration(var_decl) => {
            walk_variable_declaration(var_decl, identifiers);
        }
        oxc_ast::ast::Statement::WhileStatement(while_stmt) => {
            walk_expr(&while_stmt.test, identifiers);
            walk_statement(&while_stmt.body, identifiers);
        }
        oxc_ast::ast::Statement::DoWhileStatement(do_while) => {
            walk_statement(&do_while.body, identifiers);
            walk_expr(&do_while.test, identifiers);
        }
        oxc_ast::ast::Statement::ForStatement(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                if let oxc_ast::ast::ForStatementInit::VariableDeclaration(var_decl) = init {
                    walk_variable_declaration(var_decl, identifiers);
                } else if let Some(expr) = init.as_expression() {
                    walk_expr(expr, identifiers);
                }
            }
            if let Some(test) = &for_stmt.test {
                walk_expr(test, identifiers);
            }
            if let Some(update) = &for_stmt.update {
                walk_expr(update, identifiers);
            }
            walk_statement(&for_stmt.body, identifiers);
        }
        oxc_ast::ast::Statement::ForInStatement(for_in) => {
            walk_expr(&for_in.right, identifiers);
            walk_statement(&for_in.body, identifiers);
        }
        oxc_ast::ast::Statement::ForOfStatement(for_of) => {
            walk_expr(&for_of.right, identifiers);
            walk_statement(&for_of.body, identifiers);
        }
        oxc_ast::ast::Statement::ReturnStatement(return_stmt) => {
            if let Some(argument) = &return_stmt.argument {
                walk_expr(argument, identifiers);
            }
        }
        oxc_ast::ast::Statement::SwitchStatement(switch_stmt) => {
            walk_expr(&switch_stmt.discriminant, identifiers);
            for case in switch_stmt.cases.iter() {
                if let Some(test) = &case.test {
                    walk_expr(test, identifiers);
                }
                for statement in case.consequent.iter() {
                    walk_statement(statement, identifiers);
                }
            }
        }
        oxc_ast::ast::Statement::ThrowStatement(throw_stmt) => {
            walk_expr(&throw_stmt.argument, identifiers);
        }
        oxc_ast::ast::Statement::TryStatement(try_stmt) => {
            for statement in try_stmt.block.body.iter() {
                walk_statement(statement, identifiers);
            }
            if let Some(handler) = &try_stmt.handler {
                for statement in handler.body.body.iter() {
                    walk_statement(statement, identifiers);
                }
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for statement in finalizer.body.iter() {
                    walk_statement(statement, identifiers);
                }
            }
        }
        oxc_ast::ast::Statement::WithStatement(with_stmt) => {
            walk_expr(&with_stmt.object, identifiers);
            walk_statement(&with_stmt.body, identifiers);
        }
        oxc_ast::ast::Statement::LabeledStatement(labeled) => {
            walk_statement(&labeled.body, identifiers);
        }
        _ => {}
    }
}

fn walk_variable_declaration(
    var_decl: &oxc_ast::ast::VariableDeclaration<'_>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    for declarator in var_decl.declarations.iter() {
        if let Some(init) = &declarator.init {
            walk_expr(init, identifiers);
        }
    }
}
