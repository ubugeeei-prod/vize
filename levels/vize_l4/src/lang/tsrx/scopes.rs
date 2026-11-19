//! The names a nested scope of a component's setup declares, so a read the
//! scope shadows is not read as a prop: a function's parameters, then its
//! body's `var`s and lexical declarations; a block's, a `switch`'s, a `for`
//! head's and a `catch` parameter's lexical declarations.

use alloc::vec::Vec;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Class, ForStatementInit, ForStatementLeft,
    FormalParameters, Function, Statement, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;

/// A function's parameters.
pub(super) fn params<'a>(params: &FormalParameters<'a>) -> Vec<&'a str> {
    let mut names = Vec::new();
    for param in &params.items {
        pattern(&param.pattern, &mut names);
    }
    if let Some(rest) = &params.rest {
        pattern(&rest.rest.argument, &mut names);
    }
    names
}

/// A function body's `var`s and lexical declarations.
pub(super) fn body<'a>(body: &[Statement<'a>]) -> Vec<&'a str> {
    let mut names = Vec::new();
    let mut vars = Vars(Vec::new());
    for statement in body {
        vars.visit_statement(statement);
    }
    names.extend(vars.0);
    names.extend(lexical(body));
    names
}

/// The lexical declarations directly in `statements`.
pub(super) fn lexical<'a>(statements: &[Statement<'a>]) -> Vec<&'a str> {
    let mut names = Vec::new();
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration)
                if declaration.kind != VariableDeclarationKind::Var =>
            {
                declarators(declaration, &mut names);
            }
            Statement::FunctionDeclaration(function) => {
                names.extend(function.id.as_ref().map(|id| id.name.as_str()));
            }
            Statement::ClassDeclaration(class) => {
                names.extend(class.id.as_ref().map(|id| id.name.as_str()));
            }
            _ => {}
        }
    }
    names
}

/// The lexical declarations of a `for` head.
pub(super) fn for_init<'a>(init: Option<&ForStatementInit<'a>>) -> Vec<&'a str> {
    let mut names = Vec::new();
    if let Some(ForStatementInit::VariableDeclaration(declaration)) = init
        && declaration.kind != VariableDeclarationKind::Var
    {
        declarators(declaration, &mut names);
    }
    names
}

/// The lexical declarations of a `for … in`/`for … of` head.
pub(super) fn for_left<'a>(left: &ForStatementLeft<'a>) -> Vec<&'a str> {
    let mut names = Vec::new();
    if let ForStatementLeft::VariableDeclaration(declaration) = left
        && declaration.kind != VariableDeclarationKind::Var
    {
        declarators(declaration, &mut names);
    }
    names
}

pub(super) fn pattern<'a>(pattern: &BindingPattern<'a>, names: &mut Vec<&'a str>) {
    names.extend(
        pattern
            .get_binding_identifiers()
            .into_iter()
            .map(|id| id.name.as_str()),
    );
}

fn declarators<'a>(declaration: &VariableDeclaration<'a>, names: &mut Vec<&'a str>) {
    for declarator in &declaration.declarations {
        pattern(&declarator.id, names);
    }
}

/// Every `var` of a function body, nested functions and classes aside.
struct Vars<'a>(Vec<&'a str>);

impl<'a> Visit<'a> for Vars<'a> {
    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        if it.kind == VariableDeclarationKind::Var {
            declarators(it, &mut self.0);
        }
        walk::walk_variable_declaration(self, it);
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _: &Class<'a>) {}
}
