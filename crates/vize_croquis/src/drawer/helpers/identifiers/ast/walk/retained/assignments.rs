//! Value occurrences in assignment targets; binding targets never declare locals.

use super::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty,
    SimpleAssignmentTarget, TokenWalk,
};

impl TokenWalk {
    pub(super) fn target(&mut self, target: &AssignmentTarget<'_>) -> Option<()> {
        self.within(|walk| {
            if let Some(simple) = target.as_simple_assignment_target() {
                return walk.simple_target(simple);
            }
            match target {
                AssignmentTarget::ObjectAssignmentTarget(object) => {
                    for property in &object.properties {
                        walk.tick()?;
                        match property {
                            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                                property,
                            ) => {
                                walk.reference(
                                    property.binding.name.as_str(),
                                    property.binding.span,
                                )?;
                                if let Some(initializer) = &property.init {
                                    walk.expression(initializer)?;
                                }
                            }
                            AssignmentTargetProperty::AssignmentTargetPropertyProperty(
                                property,
                            ) => {
                                if property.computed {
                                    walk.expression(property.name.as_expression()?)?;
                                }
                                walk.target_default(&property.binding)?;
                            }
                        }
                    }
                    if let Some(rest) = &object.rest {
                        walk.target(&rest.target)?;
                    }
                    Some(())
                }
                AssignmentTarget::ArrayAssignmentTarget(array) => {
                    for element in &array.elements {
                        walk.tick()?;
                        if let Some(element) = element {
                            walk.target_default(element)?;
                        }
                    }
                    if let Some(rest) = &array.rest {
                        walk.target(&rest.target)?;
                    }
                    Some(())
                }
                _ => None,
            }
        })
    }

    fn target_default(&mut self, target: &AssignmentTargetMaybeDefault<'_>) -> Option<()> {
        self.within(|walk| match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => {
                walk.target(&default.binding)?;
                walk.expression(&default.init)
            }
            _ => walk.target(target.as_assignment_target()?),
        })
    }

    pub(super) fn simple_target(&mut self, target: &SimpleAssignmentTarget<'_>) -> Option<()> {
        self.within(|walk| match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                walk.reference(id.name.as_str(), id.span)
            }
            SimpleAssignmentTarget::StaticMemberExpression(member) => {
                walk.expression(&member.object)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                walk.expression(&member.object)?;
                walk.expression(&member.expression)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(field) => walk.expression(&field.object),
            SimpleAssignmentTarget::TSAsExpression(expression) => {
                walk.expression(&expression.expression)
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
                walk.expression(&expression.expression)
            }
            SimpleAssignmentTarget::TSNonNullExpression(expression) => {
                walk.expression(&expression.expression)
            }
            SimpleAssignmentTarget::TSTypeAssertion(expression) => {
                walk.expression(&expression.expression)
            }
        })
    }
}
