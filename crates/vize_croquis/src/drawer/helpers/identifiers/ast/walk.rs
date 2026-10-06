mod assignment_target;
pub(super) mod facts;
mod scopes;
mod type_reads;

use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind, PropertyKey};

use super::super::IdentifierRef;
use assignment_target::{walk_assignment_target, walk_simple_assignment_target};
use facts::IdentifierWalk;
use scopes::{walk_function_body, walk_parameters};

pub(super) fn walk_expr(expr: &Expression<'_>, identifiers: &mut IdentifierWalk) {
    type_reads::refuse_expression_types(expr, identifiers);
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
            if matches!(&call.callee, Expression::Identifier(id) if id.name == "eval") {
                identifiers.refuse();
            }
            walk_expr(&call.callee, identifiers);
            for arg in call.arguments.iter() {
                if let Some(e) = arg.as_expression() {
                    walk_expr(e, identifiers);
                } else {
                    identifiers.refuse();
                }
            }
        }
        Expression::NewExpression(new_expr) => {
            walk_expr(&new_expr.callee, identifiers);
            for arg in new_expr.arguments.iter() {
                if let Some(e) = arg.as_expression() {
                    walk_expr(e, identifiers);
                } else {
                    identifiers.refuse();
                }
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            let mut locals: Vec<&str> = Vec::new();
            walk_parameters(&arrow.params, &mut locals, identifiers);
            walk_function_body(&arrow.body, &mut locals, identifiers);
        }
        Expression::FunctionExpression(function) => {
            let mut locals: Vec<&str> = Vec::new();
            if let Some(id) = &function.id {
                locals.push(id.name.as_str());
            }
            walk_parameters(&function.params, &mut locals, identifiers);
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
                if matches!(&call.callee, Expression::Identifier(id) if id.name == "eval") {
                    identifiers.refuse();
                }
                walk_expr(&call.callee, identifiers);
                for arg in call.arguments.iter() {
                    if let Some(e) = arg.as_expression() {
                        walk_expr(e, identifiers);
                    } else {
                        identifiers.refuse();
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
        _ => identifiers.refuse(),
    }
}
