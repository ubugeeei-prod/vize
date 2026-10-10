//! Own slot bindings cannot supply the name used to create that slot.
//!
//! Demand the producer's complete argument Expression and slot parameter goal.
//! Refused or stale carriers never trigger a fallback parse/full analyzer.

use oxc_ast::ast::BindingPattern;
use vize_croquis::drawer::extract_identifier_refs_retained_only;
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
    let (Some(argument_ast), Some(value_ast)) = (argument.js_ast, value.js_ast) else {
        return false;
    };
    if value_ast.raw != value.content {
        return false;
    }
    let (Some(argument_ast), Some(Ok(parameters))) =
        (argument_ast.as_expression(), value_ast.as_slot_parameters())
    else {
        return false;
    };
    let mut bindings = Vec::new();
    for parameter in &parameters.items {
        collect_bindings(&parameter.pattern, &mut bindings);
    }
    if let Some(rest) = &parameters.rest {
        collect_bindings(&rest.rest.argument, &mut bindings);
    }
    if bindings.is_empty() {
        return false;
    }
    let Some(reads) = extract_identifier_refs_retained_only(argument.content, &argument_ast) else {
        return false;
    };
    reads
        .iter()
        .any(|read| bindings.contains(&read.name.as_str()))
}

/// Only declared locals: property keys and every default RHS remain reads.
fn collect_bindings<'a>(pattern: &BindingPattern<'a>, names: &mut Vec<&'a str>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => names.push(identifier.name.as_str()),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_bindings(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_bindings(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_bindings(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_bindings(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(default) => collect_bindings(&default.left, names),
    }
}
