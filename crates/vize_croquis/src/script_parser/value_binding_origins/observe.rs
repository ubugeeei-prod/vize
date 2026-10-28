//! Hooks folded into the existing declaration and expression walk.

use super::{ValueBindingOrigins, ValueReference};
use crate::scope::ScopeId;
use oxc_ast::ast::{Expression, VariableDeclarationKind};

impl crate::script_parser::ScriptParseResult {
    pub(crate) fn note_value_declaration(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclarator<'_>,
        kind: VariableDeclarationKind,
    ) {
        let scope = self.scopes.current_id();
        if let Some(origins) = self.value_binding_origins.as_mut() {
            origins.note_declaration(scope, &declaration.id, kind, declaration.init.as_ref());
        }
    }

    pub(crate) fn note_value_function(&mut self, function: &oxc_ast::ast::Function<'_>) {
        let scope = self.scopes.current_id();
        if let Some(id) = &function.id
            && let Some(origins) = self.value_binding_origins.as_mut()
        {
            origins.note_opaque_binding(scope, id.name.as_str(), id.span);
            if let Some(key) = origins.declaration_key(scope, id.span) {
                origins.async_components.note_function(key, function);
            }
        }
    }

    pub(crate) fn note_value_class(&mut self, class: &oxc_ast::ast::Class<'_>) {
        if let Some(id) = &class.id
            && let Some(origins) = self.value_binding_origins.as_mut()
        {
            origins.refuse();
            origins.note_opaque_binding(self.scopes.current_id(), id.name.as_str(), id.span);
        }
    }
}

/// Observe writes at nodes the existing scope walker already visits.
pub(crate) fn note_expression_mutation(
    result: &mut crate::script_parser::ScriptParseResult,
    expression: &Expression<'_>,
) {
    use oxc_ast::ast::{AssignmentTarget, SimpleAssignmentTarget};
    let scope = result.scopes.current_id();
    let Some(origins) = result.value_binding_origins.as_mut() else {
        return;
    };
    match expression {
        Expression::AssignmentExpression(assignment) => match &assignment.left {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                origins.note_write(ValueReference::new(scope, id.name.as_str(), id.span), false)
            }
            AssignmentTarget::StaticMemberExpression(member) => {
                note_member_write(origins, scope, &member.object)
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                note_member_write(origins, scope, &member.object)
            }
            _ => origins.refuse(),
        },
        Expression::UpdateExpression(update) => match &update.argument {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                origins.note_write(ValueReference::new(scope, id.name.as_str(), id.span), false)
            }
            SimpleAssignmentTarget::StaticMemberExpression(member) => {
                note_member_write(origins, scope, &member.object)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                note_member_write(origins, scope, &member.object)
            }
            _ => origins.refuse(),
        },
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_syntax::operator::UnaryOperator::Delete =>
        {
            match unary.argument.get_inner_expression() {
                Expression::StaticMemberExpression(member) => {
                    note_member_write(origins, scope, &member.object)
                }
                Expression::ComputedMemberExpression(member) => {
                    note_member_write(origins, scope, &member.object)
                }
                _ => {}
            }
        }
        Expression::CallExpression(call) if matches!(call.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "eval") => {
            origins.refuse()
        }
        Expression::ClassExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::TSTypeAssertion(_)
        | Expression::TSInstantiationExpression(_)
        | Expression::YieldExpression(_)
        | Expression::PrivateInExpression(_)
        | Expression::PrivateFieldExpression(_) => origins.refuse(),
        Expression::ImportExpression(import)
            if import.options.is_some()
                || !matches!(
                    import.source.get_inner_expression(),
                    Expression::StringLiteral(_) | Expression::Identifier(_)
                ) =>
        {
            origins.refuse()
        }
        // Named function-expression identity is local to the closure; the
        // scope builder does not currently record that name as a binding.
        // Refuse this domain rather than accidentally inheriting an import.
        Expression::FunctionExpression(function) if function.id.is_some() => {
            origins.refuse();
        }
        _ => {}
    }
}

fn note_member_write(
    origins: &mut ValueBindingOrigins,
    scope: ScopeId,
    expression: &Expression<'_>,
) {
    match expression.get_inner_expression() {
        Expression::Identifier(id) => {
            origins.note_write(ValueReference::new(scope, id.name.as_str(), id.span), true)
        }
        Expression::StaticMemberExpression(member) => {
            note_member_write(origins, scope, &member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            note_member_write(origins, scope, &member.object)
        }
        _ => {}
    }
}

/// The existing for-in/of walker owns the only visit to the assignment slot.
pub(crate) fn note_loop_left(
    result: &mut crate::script_parser::ScriptParseResult,
    left: &oxc_ast::ast::ForStatementLeft<'_>,
) {
    if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(declaration) = left {
        for declarator in &declaration.declarations {
            result.note_value_declaration(declarator, declaration.kind);
        }
    } else if let Some(target) = left.as_assignment_target() {
        let scope = result.scopes.current_id();
        if let Some(origins) = result.value_binding_origins.as_mut() {
            use oxc_ast::ast::AssignmentTarget;
            match target {
                AssignmentTarget::AssignmentTargetIdentifier(id) => {
                    origins.note_write(ValueReference::new(scope, id.name.as_str(), id.span), false)
                }
                AssignmentTarget::StaticMemberExpression(member) => {
                    note_member_write(origins, scope, &member.object)
                }
                AssignmentTarget::ComputedMemberExpression(member) => {
                    note_member_write(origins, scope, &member.object)
                }
                // Destructuring/defaulted targets need ownership the current
                // walker does not encode. Refuse instead of guessing names.
                _ => origins.refuse(),
            }
        }
    }
}
