use oxc_ast::ast::{
    ArrayExpressionElement, BindingPattern, Expression, ObjectPropertyKind, PropertyKey,
};

use super::super::IdentifierRef;

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
        Expression::UpdateExpression(update) => match &update.argument {
            oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                identifiers.push(IdentifierRef::new(id.name.as_str(), id.span.start));
            }
            oxc_ast::ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
                walk_expr(&member.object, identifiers);
            }
            oxc_ast::ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                walk_expr(&member.object, identifiers);
                walk_expr(&member.expression, identifiers);
            }
            oxc_ast::ast::SimpleAssignmentTarget::PrivateFieldExpression(field) => {
                walk_expr(&field.object, identifiers);
            }
            _ => {}
        },
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
            let mut param_names: Vec<&str> = Vec::new();
            for param in arrow.params.items.iter() {
                collect_binding_names(&param.pattern, &mut param_names);
            }

            if arrow.expression
                && let Some(oxc_ast::ast::Statement::ExpressionStatement(expr_stmt)) =
                    arrow.body.statements.first()
            {
                let mut body_idents = Vec::new();
                walk_expr(&expr_stmt.expression, &mut body_idents);
                for ident in body_idents {
                    if !param_names.contains(&ident.name.as_str()) {
                        identifiers.push(ident);
                    }
                }
            }
        }
        Expression::SequenceExpression(seq) => {
            for e in seq.expressions.iter() {
                walk_expr(e, identifiers);
            }
        }
        Expression::AssignmentExpression(assign) => {
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
