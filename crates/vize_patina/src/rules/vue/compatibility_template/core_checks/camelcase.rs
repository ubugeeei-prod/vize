//! `camelcase` placement over OXC, following ESLint's core rule selectors.

use super::{predicates::is_underscored_identifier, visitor::CoreVisitor};
use oxc_ast::ast::{
    Argument, AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, Expression,
    ObjectExpression, ObjectPropertyKind, PropertyKey,
};
use oxc_span::GetSpan;

impl CoreVisitor<'_, '_> {
    /// Member targets of an assignment or of any destructuring pattern; a bare
    /// `for (a.b_c of x)` or `a.b_c++` is a read for the core rule.
    pub(super) fn camelcase_assignment_target(
        &mut self,
        target: &AssignmentTarget<'_>,
        report_member: bool,
    ) {
        match target {
            AssignmentTarget::StaticMemberExpression(member)
                if report_member && is_underscored_identifier(&member.property.name) =>
            {
                self.report(member.property.span);
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                for property in &object.properties {
                    if let AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) =
                        property
                    {
                        self.camelcase_maybe_default(&property.binding);
                    }
                }
                if let Some(rest) = &object.rest {
                    self.camelcase_assignment_target(&rest.target, true);
                }
            }
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                for element in array.elements.iter().flatten() {
                    self.camelcase_maybe_default(element);
                }
                if let Some(rest) = &array.rest {
                    self.camelcase_assignment_target(&rest.target, true);
                }
            }
            _ => {}
        }
    }

    fn camelcase_maybe_default(&mut self, target: &AssignmentTargetMaybeDefault<'_>) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                self.camelcase_assignment_target(&target.binding, true);
            }
            target => {
                if let Some(target) = target.as_assignment_target() {
                    self.camelcase_assignment_target(target, true);
                }
            }
        }
    }

    /// A reference the core rule skips "for backward compatibility": the
    /// callee or an argument of a call or `new`, or a default value.
    pub(super) fn camelcase_skip_reference(&mut self, expression: &Expression<'_>) {
        if let Expression::Identifier(identifier) = expression.without_parentheses() {
            self.covered_spans
                .insert((identifier.span.start, identifier.span.end));
        }
    }

    pub(super) fn camelcase_skip_call_operands(
        &mut self,
        callee: &Expression<'_>,
        arguments: &[Argument<'_>],
    ) {
        self.camelcase_skip_reference(callee);
        for argument in arguments {
            if let Some(argument) = argument.as_expression() {
                self.camelcase_skip_reference(argument);
            }
        }
    }

    /// Keys of an object literal, except the attribute keys of `import()`.
    pub(super) fn camelcase_object_keys(&mut self, object: &ObjectExpression<'_>) {
        let import_attributes = self
            .covered_spans
            .contains(&(object.span.start, object.span.end));
        for entry in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                continue;
            };
            if property.computed {
                continue;
            }
            if import_attributes {
                if let Expression::ObjectExpression(nested) = property.value.without_parentheses() {
                    self.covered_spans
                        .insert((nested.span.start, nested.span.end));
                }
                if property.shorthand {
                    self.camelcase_skip_reference(&property.value);
                }
                continue;
            }
            if let PropertyKey::StaticIdentifier(key) = &property.key
                && is_underscored_identifier(&key.name)
            {
                self.report(key.span);
            }
        }
    }

    /// `import(source, { with_key: ... })`: the options object holds attribute keys.
    pub(super) fn camelcase_import_options(&mut self, options: Option<&Expression<'_>>) {
        if let Some(Expression::ObjectExpression(object)) =
            options.map(Expression::without_parentheses)
        {
            self.covered_spans
                .insert((object.span.start, object.span.end));
        }
    }

    /// Non-computed class member names and private names.
    pub(super) fn camelcase_class_key(&mut self, key: &PropertyKey<'_>, computed: bool) {
        let name = match key {
            PropertyKey::StaticIdentifier(key) if !computed => key.name.as_str(),
            PropertyKey::PrivateIdentifier(key) => key.name.as_str(),
            _ => return,
        };
        if is_underscored_identifier(name) {
            self.report(key.span());
        }
    }
}
