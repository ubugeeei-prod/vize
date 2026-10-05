use super::super::super::IdentifierRef;
use super::walk_expr;
use oxc_ast::ast::{BindingPattern, Expression, FormalParameters, FunctionBody, Statement};

/// Bind a parameter list, the rest parameter included, and report the
/// references its default values and computed keys make to the outside.
pub(super) fn walk_parameters<'a>(
    params: &'a FormalParameters<'a>,
    locals: &mut Vec<&'a str>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    for param in params.items.iter() {
        collect_binding_names(&param.pattern, locals);
    }
    if let Some(rest) = &params.rest {
        collect_binding_names(&rest.rest.argument, locals);
    }
    let mut found = Vec::new();
    for param in params.items.iter() {
        walk_binding_pattern_expressions(&param.pattern, &mut found);
        if let Some(initializer) = &param.initializer {
            walk_expr(initializer, &mut found);
        }
    }
    if let Some(rest) = &params.rest {
        walk_binding_pattern_expressions(&rest.rest.argument, &mut found);
    }
    push_escaping(found, locals, identifiers);
}

/// Walk a function body, reporting only the references that escape it.
///
/// A concise arrow body is a single expression statement; a block body
/// (`@click="() => { $router.replace(to) }"`) declares its own bindings.
/// `var` hoists to the function, while `const`/`let`, functions and classes
/// belong to the block that declares them, so a `const` shadowing a template
/// name stays local to its block while `$router` and `to` still reach
/// template scope. Statement kinds outside the handled set contribute
/// nothing, as before.
pub(super) fn walk_function_body<'a>(
    body: &'a FunctionBody<'a>,
    locals: &mut Vec<&'a str>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    for statement in body.statements.iter() {
        collect_var_declarations(statement, locals);
    }
    walk_block(&body.statements, locals, identifiers);
}

/// Walk a statement list as one lexical scope: the `const`/`let`, function
/// and class names it declares shadow outer names for this walk only.
fn walk_block<'a>(
    statements: &'a [Statement<'a>],
    locals: &mut Vec<&'a str>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    let depth = locals.len();
    collect_lexical_declarations(statements, locals);
    for statement in statements {
        walk_statement(statement, locals, identifiers);
    }
    locals.truncate(depth);
}

/// `var` names a statement declares, looking through the blocks and `if`
/// branches the walk descends into: they hoist to the enclosing function.
fn collect_var_declarations<'a>(statement: &'a Statement<'a>, locals: &mut Vec<&'a str>) {
    match statement {
        Statement::VariableDeclaration(declaration) if declaration.kind.is_var() => {
            for declarator in declaration.declarations.iter() {
                collect_binding_names(&declarator.id, locals);
            }
        }
        Statement::BlockStatement(block) => {
            for statement in block.body.iter() {
                collect_var_declarations(statement, locals);
            }
        }
        Statement::IfStatement(if_statement) => {
            collect_var_declarations(&if_statement.consequent, locals);
            if let Some(alternate) = &if_statement.alternate {
                collect_var_declarations(alternate, locals);
            }
        }
        _ => {}
    }
}

/// Block-scoped names a statement list declares: `const`/`let` (and `using`),
/// functions and classes. Nested blocks bind their own when walked.
fn collect_lexical_declarations<'a>(statements: &'a [Statement<'a>], locals: &mut Vec<&'a str>) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration) if !declaration.kind.is_var() => {
                for declarator in declaration.declarations.iter() {
                    collect_binding_names(&declarator.id, locals);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    locals.push(id.name.as_str());
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    locals.push(id.name.as_str());
                }
            }
            _ => {}
        }
    }
}

/// Walk the expressions a body statement evaluates. Loops, `try`, `switch` and
/// other statement kinds are not descended into, matching the previous
/// behavior for those shapes.
fn walk_statement<'a>(
    statement: &'a Statement<'a>,
    locals: &mut Vec<&'a str>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    match statement {
        Statement::ExpressionStatement(expression) => {
            walk_scoped_expr(&expression.expression, locals, identifiers);
        }
        Statement::ReturnStatement(ret) => {
            if let Some(argument) = &ret.argument {
                walk_scoped_expr(argument, locals, identifiers);
            }
        }
        Statement::ThrowStatement(throw) => {
            walk_scoped_expr(&throw.argument, locals, identifiers);
        }
        Statement::VariableDeclaration(declaration) => {
            let mut found = Vec::new();
            for declarator in declaration.declarations.iter() {
                walk_binding_pattern_expressions(&declarator.id, &mut found);
                if let Some(init) = &declarator.init {
                    walk_expr(init, &mut found);
                }
            }
            push_escaping(found, locals, identifiers);
        }
        Statement::IfStatement(if_statement) => {
            walk_scoped_expr(&if_statement.test, locals, identifiers);
            walk_statement(&if_statement.consequent, locals, identifiers);
            if let Some(alternate) = &if_statement.alternate {
                walk_statement(alternate, locals, identifiers);
            }
        }
        Statement::BlockStatement(block) => {
            walk_block(&block.body, locals, identifiers);
        }
        _ => {}
    }
}

/// Walk one expression and keep the references no enclosing scope declares.
fn walk_scoped_expr(expr: &Expression<'_>, locals: &[&str], identifiers: &mut Vec<IdentifierRef>) {
    let mut found = Vec::new();
    walk_expr(expr, &mut found);
    push_escaping(found, locals, identifiers);
}

/// Keep the references that name no local of the enclosing scopes.
fn push_escaping(found: Vec<IdentifierRef>, locals: &[&str], identifiers: &mut Vec<IdentifierRef>) {
    identifiers.extend(
        found
            .into_iter()
            .filter(|identifier| !locals.contains(&identifier.name.as_str())),
    );
}

/// Walk the expressions a binding pattern evaluates rather than declares:
/// default values and computed keys read the surrounding scope.
fn walk_binding_pattern_expressions(
    pattern: &BindingPattern<'_>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(_) => {}
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                if prop.computed
                    && let Some(key_expr) = prop.key.as_expression()
                {
                    walk_expr(key_expr, identifiers);
                }
                walk_binding_pattern_expressions(&prop.value, identifiers);
            }
            if let Some(rest) = &obj.rest {
                walk_binding_pattern_expressions(&rest.argument, identifiers);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                walk_binding_pattern_expressions(elem, identifiers);
            }
            if let Some(rest) = &arr.rest {
                walk_binding_pattern_expressions(&rest.argument, identifiers);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            walk_binding_pattern_expressions(&assign.left, identifiers);
            walk_expr(&assign.right, identifiers);
        }
    }
}

fn collect_binding_names<'a>(pattern: &'a BindingPattern<'a>, names: &mut Vec<&'a str>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.push(id.name.as_str());
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                collect_binding_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names(elem, names);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names(&assign.left, names);
        }
    }
}
