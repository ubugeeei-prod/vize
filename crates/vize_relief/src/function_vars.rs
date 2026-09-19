//! Read function-owned `var` bindings from an existing JavaScript body.

use oxc_ast::ast::{
    BindingPattern, ForStatementInit, ForStatementLeft, FunctionBody, Statement,
    VariableDeclaration, VariableDeclarationKind,
};

/// Visit every `var` binding owned by this function before rewriting its body.
///
/// Blocks, loops and catch clauses do not own `var`; nested functions and class
/// bodies do. Walk only statement containers, without entering expressions or
/// nested declaration bodies. Parameter defaults are outside this body's scope.
/// This consumes the retained AST and does not create another parse or AST.
pub fn for_each_function_var<'a>(
    body: &FunctionBody<'a>,
    mut visit: impl FnMut(&BindingPattern<'a>),
) {
    let mut pending: vize_s0::SmallVec<[_; 8]> = body.statements.iter().collect();
    while let Some(statement) = pending.pop() {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                declaration_vars(declaration, &mut visit)
            }
            Statement::BlockStatement(block) => pending.extend(block.body.iter()),
            Statement::IfStatement(branch) => {
                pending.push(&branch.consequent);
                pending.extend(branch.alternate.iter());
            }
            Statement::ForStatement(loop_) => {
                if let Some(ForStatementInit::VariableDeclaration(declaration)) = &loop_.init {
                    declaration_vars(declaration, &mut visit);
                }
                pending.push(&loop_.body);
            }
            Statement::ForInStatement(loop_) => {
                if let ForStatementLeft::VariableDeclaration(declaration) = &loop_.left {
                    declaration_vars(declaration, &mut visit);
                }
                pending.push(&loop_.body);
            }
            Statement::ForOfStatement(loop_) => {
                if let ForStatementLeft::VariableDeclaration(declaration) = &loop_.left {
                    declaration_vars(declaration, &mut visit);
                }
                pending.push(&loop_.body);
            }
            Statement::WhileStatement(loop_) => pending.push(&loop_.body),
            Statement::DoWhileStatement(loop_) => pending.push(&loop_.body),
            Statement::LabeledStatement(labeled) => pending.push(&labeled.body),
            Statement::WithStatement(with) => pending.push(&with.body),
            Statement::SwitchStatement(switch) => {
                pending.extend(switch.cases.iter().flat_map(|case| case.consequent.iter()));
            }
            Statement::TryStatement(try_) => {
                pending.extend(try_.block.body.iter());
                if let Some(handler) = &try_.handler {
                    pending.extend(handler.body.body.iter());
                }
                if let Some(finalizer) = &try_.finalizer {
                    pending.extend(finalizer.body.iter());
                }
            }
            _ => {}
        }
    }
}

fn declaration_vars<'a>(
    declaration: &VariableDeclaration<'a>,
    visit: &mut impl FnMut(&BindingPattern<'a>),
) {
    if declaration.kind == VariableDeclarationKind::Var {
        for declarator in &declaration.declarations {
            visit(&declarator.id);
        }
    }
}
