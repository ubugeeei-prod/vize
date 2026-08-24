use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty,
    SimpleAssignmentTarget,
};

use super::super::super::IdentifierRef;
use super::walk_expr;

pub(super) fn walk_assignment_target(
    target: &AssignmentTarget<'_>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => {
            identifiers.push(IdentifierRef::new(id.name.as_str(), id.span.start));
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
            walk_expr(&member.expression, identifiers);
        }
        AssignmentTarget::PrivateFieldExpression(field) => {
            walk_expr(&field.object, identifiers);
        }
        AssignmentTarget::TSAsExpression(as_expr) => {
            walk_expr(&as_expr.expression, identifiers);
        }
        AssignmentTarget::TSSatisfiesExpression(satisfies) => {
            walk_expr(&satisfies.expression, identifiers);
        }
        AssignmentTarget::TSNonNullExpression(non_null) => {
            walk_expr(&non_null.expression, identifiers);
        }
        AssignmentTarget::TSTypeAssertion(assertion) => {
            walk_expr(&assertion.expression, identifiers);
        }
        AssignmentTarget::ObjectAssignmentTarget(obj) => {
            walk_object_assignment_target(obj, identifiers);
        }
        AssignmentTarget::ArrayAssignmentTarget(arr) => {
            for elem in arr.elements.iter().flatten() {
                walk_assignment_target_maybe_default(elem, identifiers);
            }
            if let Some(rest) = &arr.rest {
                walk_assignment_target(&rest.target, identifiers);
            }
        }
    }
}

fn walk_assignment_target_maybe_default(
    target: &AssignmentTargetMaybeDefault<'_>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => {
            walk_assignment_target(&default.binding, identifiers);
            walk_expr(&default.init, identifiers);
        }
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
            identifiers.push(IdentifierRef::new(id.name.as_str(), id.span.start));
        }
        AssignmentTargetMaybeDefault::StaticMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
        }
        AssignmentTargetMaybeDefault::ComputedMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
            walk_expr(&member.expression, identifiers);
        }
        AssignmentTargetMaybeDefault::PrivateFieldExpression(field) => {
            walk_expr(&field.object, identifiers);
        }
        AssignmentTargetMaybeDefault::TSAsExpression(as_expr) => {
            walk_expr(&as_expr.expression, identifiers);
        }
        AssignmentTargetMaybeDefault::TSSatisfiesExpression(satisfies) => {
            walk_expr(&satisfies.expression, identifiers);
        }
        AssignmentTargetMaybeDefault::TSNonNullExpression(non_null) => {
            walk_expr(&non_null.expression, identifiers);
        }
        AssignmentTargetMaybeDefault::TSTypeAssertion(assertion) => {
            walk_expr(&assertion.expression, identifiers);
        }
        AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
            walk_object_assignment_target(obj, identifiers);
        }
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(arr) => {
            for elem in arr.elements.iter().flatten() {
                walk_assignment_target_maybe_default(elem, identifiers);
            }
            if let Some(rest) = &arr.rest {
                walk_assignment_target(&rest.target, identifiers);
            }
        }
    }
}

fn walk_object_assignment_target(
    obj: &oxc_ast::ast::ObjectAssignmentTarget<'_>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    for prop in obj.properties.iter() {
        match prop {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(prop_ident) => {
                identifiers.push(IdentifierRef::new(
                    prop_ident.binding.name.as_str(),
                    prop_ident.binding.span.start,
                ));
                if let Some(init) = &prop_ident.init {
                    walk_expr(init, identifiers);
                }
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop_prop) => {
                if prop_prop.computed
                    && let Some(key_expr) = prop_prop.name.as_expression()
                {
                    walk_expr(key_expr, identifiers);
                }
                walk_assignment_target_maybe_default(&prop_prop.binding, identifiers);
            }
        }
    }
    if let Some(rest) = &obj.rest {
        walk_assignment_target(&rest.target, identifiers);
    }
}

pub(super) fn walk_simple_assignment_target(
    target: &SimpleAssignmentTarget<'_>,
    identifiers: &mut Vec<IdentifierRef>,
) {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
            identifiers.push(IdentifierRef::new(id.name.as_str(), id.span.start));
        }
        SimpleAssignmentTarget::StaticMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
        }
        SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            walk_expr(&member.object, identifiers);
            walk_expr(&member.expression, identifiers);
        }
        SimpleAssignmentTarget::PrivateFieldExpression(field) => {
            walk_expr(&field.object, identifiers);
        }
        SimpleAssignmentTarget::TSAsExpression(as_expr) => {
            walk_expr(&as_expr.expression, identifiers);
        }
        SimpleAssignmentTarget::TSSatisfiesExpression(satisfies) => {
            walk_expr(&satisfies.expression, identifiers);
        }
        SimpleAssignmentTarget::TSNonNullExpression(non_null) => {
            walk_expr(&non_null.expression, identifiers);
        }
        SimpleAssignmentTarget::TSTypeAssertion(assertion) => {
            walk_expr(&assertion.expression, identifiers);
        }
    }
}
