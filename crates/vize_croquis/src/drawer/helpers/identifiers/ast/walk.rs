mod assignment_target;

use oxc_ast::ast::{
    ArrayExpressionElement, BindingPattern, Expression, FunctionBody, ObjectPropertyKind,
    PropertyKey, Statement,
};

use super::super::IdentifierRef;
use assignment_target::{walk_assignment_target, walk_simple_assignment_target};

pub(super) fn walk_expr(expr: &Expression<'_>, identifiers: &mut Vec<IdentifierRef>) {
    match expr {
        Expression::Identifier(id) => {
            identifiers.push(IdentifierRef::new(id.name.as_str(), id.span.start));
        }
        Expression::StaticMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
        }
        Expression::ComputedMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
            walk_expr(&member.expression, identifiers);
        }
        Expression::PrivateFieldExpression(field) => {
            walk_expr(&field.object, identifiers);
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    ObjectPropertyKind::ObjectProperty(p) => {
                        if p.computed
                            && let Some(key_expr) = p.key.as_expression()
                        {
                            walk_expr(key_expr, identifiers);
                        }
                        if p.shorthand {
                            if let PropertyKey::StaticIdentifier(id) = &p.key {
                                identifiers
                                    .push(IdentifierRef::new(id.name.as_str(), id.span.start));
                            }
                        } else {
                            walk_expr(&p.value, identifiers);
                        }
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        walk_expr(&spread.argument, identifiers);
                    }
                }
            }
        }
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                match elem {
                    ArrayExpressionElement::SpreadElement(spread) => {
                        walk_expr(&spread.argument, identifiers);
                    }
                    ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(e) = elem.as_expression() {
                            walk_expr(e, identifiers);
                        }
                    }
                }
            }
        }
        Expression::BinaryExpression(binary) => {
            walk_expr(&binary.left, identifiers);
            walk_expr(&binary.right, identifiers);
        }
        Expression::LogicalExpression(logical) => {
            walk_expr(&logical.left, identifiers);
            walk_expr(&logical.right, identifiers);
        }
        Expression::ConditionalExpression(cond) => {
            walk_expr(&cond.test, identifiers);
            walk_expr(&cond.consequent, identifiers);
            walk_expr(&cond.alternate, identifiers);
        }
        Expression::UnaryExpression(unary) => {
            walk_expr(&unary.argument, identifiers);
        }
        Expression::UpdateExpression(update) => {
            walk_simple_assignment_target(&update.argument, identifiers);
        }
        Expression::CallExpression(call) => {
            walk_expr(&call.callee, identifiers);
            for arg in call.arguments.iter() {
                if let Some(e) = arg.as_expression() {
                    walk_expr(e, identifiers);
                }
            }
        }
        Expression::NewExpression(new_expr) => {
            walk_expr(&new_expr.callee, identifiers);
            for arg in new_expr.arguments.iter() {
                if let Some(e) = arg.as_expression() {
                    walk_expr(e, identifiers);
                }
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            let mut locals: Vec<&str> = Vec::new();
            for param in arrow.params.items.iter() {
                collect_binding_names(&param.pattern, &mut locals);
            }
            walk_function_body(&arrow.body, &mut locals, identifiers);
        }
        Expression::FunctionExpression(function) => {
            let mut locals: Vec<&str> = Vec::new();
            if let Some(id) = &function.id {
                locals.push(id.name.as_str());
            }
            for param in function.params.items.iter() {
                collect_binding_names(&param.pattern, &mut locals);
            }
            if let Some(body) = &function.body {
                walk_function_body(body, &mut locals, identifiers);
            }
        }
        Expression::SequenceExpression(seq) => {
            for e in seq.expressions.iter() {
                walk_expr(e, identifiers);
            }
        }
        Expression::AssignmentExpression(assign) => {
            walk_assignment_target(&assign.left, identifiers);
            walk_expr(&assign.right, identifiers);
        }
        Expression::TemplateLiteral(template) => {
            for expr in template.expressions.iter() {
                walk_expr(expr, identifiers);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            walk_expr(&tagged.tag, identifiers);
            for expr in tagged.quasi.expressions.iter() {
                walk_expr(expr, identifiers);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            walk_expr(&paren.expression, identifiers);
        }
        Expression::AwaitExpression(await_expr) => {
            walk_expr(&await_expr.argument, identifiers);
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(arg) = &yield_expr.argument {
                walk_expr(arg, identifiers);
            }
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                walk_expr(&call.callee, identifiers);
                for arg in call.arguments.iter() {
                    if let Some(e) = arg.as_expression() {
                        walk_expr(e, identifiers);
                    }
                }
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(non_null) => {
                walk_expr(&non_null.expression, identifiers);
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                walk_expr(&member.object, identifiers);
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                walk_expr(&member.object, identifiers);
                walk_expr(&member.expression, identifiers);
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                walk_expr(&field.object, identifiers);
            }
        },
        Expression::TSAsExpression(as_expr) => {
            walk_expr(&as_expr.expression, identifiers);
        }
        Expression::TSSatisfiesExpression(satisfies) => {
            walk_expr(&satisfies.expression, identifiers);
        }
        Expression::TSNonNullExpression(non_null) => {
            walk_expr(&non_null.expression, identifiers);
        }
        Expression::TSTypeAssertion(assertion) => {
            walk_expr(&assertion.expression, identifiers);
        }
        Expression::TSInstantiationExpression(inst) => {
            walk_expr(&inst.expression, identifiers);
        }
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::RegExpLiteral(_) => {}
        _ => {}
    }
}

/// Walk a function body, reporting only the references that escape it.
///
/// A concise arrow body is a single expression statement; a block body
/// (`@click="() => { $router.replace(to) }"`) declares its own lexical
/// bindings, which are collected first so a `const` shadowing a template name
/// stays local while `$router` and `to` still reach template scope. Statement
/// kinds outside the handled set contribute nothing, as before.
fn walk_function_body<'a>(
    body: &'a FunctionBody<'a>,
    locals: &mut Vec<&'a str>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    collect_statement_declarations(&body.statements, locals);
    let mut body_idents = Vec::new();
    for statement in body.statements.iter() {
        walk_statement(statement, &mut body_idents);
    }
    for ident in body_idents {
        if !locals.contains(&ident.name.as_str()) {
            identifiers.push(ident);
        }
    }
}

/// Names a statement list declares (`const`/`let`/`var`, functions, classes),
/// including those of nested plain blocks, so the body walk can drop them.
fn collect_statement_declarations<'a>(statements: &'a [Statement<'a>], locals: &mut Vec<&'a str>) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration) => {
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
            Statement::BlockStatement(block) => {
                collect_statement_declarations(&block.body, locals);
            }
            _ => {}
        }
    }
}

/// Walk the expressions a body statement evaluates. Loops, `try`, `switch` and
/// other statement kinds are not descended into, matching the previous
/// behavior for those shapes.
fn walk_statement(statement: &Statement<'_>, identifiers: &mut Vec<IdentifierRef>) {
    match statement {
        Statement::ExpressionStatement(expression) => {
            walk_expr(&expression.expression, identifiers);
        }
        Statement::ReturnStatement(ret) => {
            if let Some(argument) = &ret.argument {
                walk_expr(argument, identifiers);
            }
        }
        Statement::ThrowStatement(throw) => {
            walk_expr(&throw.argument, identifiers);
        }
        Statement::VariableDeclaration(declaration) => {
            for declarator in declaration.declarations.iter() {
                if let Some(init) = &declarator.init {
                    walk_expr(init, identifiers);
                }
            }
        }
        Statement::IfStatement(if_statement) => {
            walk_expr(&if_statement.test, identifiers);
            walk_statement(&if_statement.consequent, identifiers);
            if let Some(alternate) = &if_statement.alternate {
                walk_statement(alternate, identifiers);
            }
        }
        Statement::BlockStatement(block) => {
            for statement in block.body.iter() {
                walk_statement(statement, identifiers);
            }
        }
        _ => {}
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
