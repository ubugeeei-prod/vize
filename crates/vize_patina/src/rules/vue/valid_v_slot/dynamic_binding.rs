//! Own slot bindings cannot supply the name used to create that slot.
//!
//! Read only the parser's retained ASTs. Invalid, incomplete or unsupported
//! patterns remain on their existing routes; this syntax rule never invokes
//! the full semantic analyzer or reparses a directive.

use oxc_ast::ast::{
    ArrayExpressionElement, AssignmentTarget, Expression, ObjectPropertyKind, PropertyKind,
};
use oxc_syntax::operator::AssignmentOperator;
use vize_croquis::drawer::helpers::extract_identifier_refs_retained_only;
use vize_relief::{DirectiveNode, ExpressionNode};

pub(super) fn references_own_binding(directive: &DirectiveNode<'_>) -> bool {
    let (Some(ExpressionNode::Simple(argument)), Some(ExpressionNode::Simple(value))) =
        (&directive.arg, &directive.exp)
    else {
        return false;
    };
    if argument.is_static {
        return false;
    }
    let (Some(argument_ast), Some(value_ast)) = (&argument.js_ast, &value.js_ast) else {
        return false;
    };
    if value_ast.raw != value.content {
        return false;
    }
    let mut bindings = Vec::new();
    if !collect_bindings(value_ast.ast, &mut bindings) || bindings.is_empty() {
        return false;
    }
    let Some(reads) = extract_identifier_refs_retained_only(argument.content, argument_ast) else {
        return false;
    };
    reads
        .iter()
        .any(|read| bindings.contains(&read.name.as_str()))
}

/// An expression AST can represent these complete parameter-pattern shapes.
/// Read local values, never source property keys or default initializer reads.
fn collect_bindings<'a>(expression: &Expression<'a>, names: &mut Vec<&'a str>) -> bool {
    match expression {
        Expression::Identifier(identifier) => {
            names.push(identifier.name.as_str());
            true
        }
        Expression::ObjectExpression(object) => {
            for (index, property) in object.properties.iter().enumerate() {
                match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        if property.method
                            || property.kind != PropertyKind::Init
                            || !collect_bindings(&property.value, names)
                        {
                            return false;
                        }
                    }
                    ObjectPropertyKind::SpreadProperty(rest) => {
                        if index + 1 != object.properties.len()
                            || !matches!(&rest.argument, Expression::Identifier(_))
                            || !collect_bindings(&rest.argument, names)
                        {
                            return false;
                        }
                    }
                }
            }
            true
        }
        Expression::ArrayExpression(array) => {
            for (index, element) in array.elements.iter().enumerate() {
                match element {
                    ArrayExpressionElement::Elision(_) => {}
                    ArrayExpressionElement::SpreadElement(rest) => {
                        if index + 1 != array.elements.len()
                            || !collect_bindings(&rest.argument, names)
                        {
                            return false;
                        }
                    }
                    _ => {
                        let Some(value) = element.as_expression() else {
                            return false;
                        };
                        if !collect_bindings(value, names) {
                            return false;
                        }
                    }
                }
            }
            true
        }
        Expression::AssignmentExpression(default)
            if default.operator == AssignmentOperator::Assign =>
        {
            if let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &default.left {
                names.push(identifier.name.as_str());
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
