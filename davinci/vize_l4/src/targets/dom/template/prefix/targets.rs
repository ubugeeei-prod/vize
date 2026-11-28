//! Assignment-target position collection for [`IdentifierCollector`]
//! (`steps::expression::collector_targets`, ported).

use oxc_ast::ast as oxc_ast_types;

use super::collector::IdentifierCollector;

impl IdentifierCollector<'_, '_> {
    pub(super) fn collect_assignment_targets(
        &mut self,
        target: &oxc_ast_types::AssignmentTarget<'_>,
    ) {
        use oxc_ast_types::AssignmentTarget;
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.push_assignment_target(ident.span.start);
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                self.collect_object_assignment_target(obj);
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.collect_assignment_targets_maybe_default(elem);
                }
                if let Some(rest) = &arr.rest {
                    self.collect_assignment_targets(&rest.target);
                }
            }
            _ => {}
        }
    }

    fn collect_object_assignment_target(
        &mut self,
        obj: &oxc_ast_types::ObjectAssignmentTarget<'_>,
    ) {
        use oxc_ast_types::AssignmentTargetProperty;
        for prop in &obj.properties {
            match prop {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(prop_ident) => {
                    self.push_assignment_target(prop_ident.binding.span.start);
                }
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop_prop) => {
                    self.collect_assignment_targets_maybe_default(&prop_prop.binding);
                }
            }
        }
        if let Some(rest) = &obj.rest {
            self.collect_assignment_targets(&rest.target);
        }
    }

    fn collect_assignment_targets_maybe_default(
        &mut self,
        target: &oxc_ast_types::AssignmentTargetMaybeDefault<'_>,
    ) {
        use oxc_ast_types::AssignmentTargetMaybeDefault;
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(def) => {
                self.collect_assignment_targets(&def.binding);
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                self.push_assignment_target(ident.span.start);
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                self.collect_object_assignment_target(obj);
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.collect_assignment_targets_maybe_default(elem);
                }
                if let Some(rest) = &arr.rest {
                    self.collect_assignment_targets(&rest.target);
                }
            }
            _ => {}
        }
    }

    pub(super) fn collect_simple_assignment_targets(
        &mut self,
        target: &oxc_ast_types::SimpleAssignmentTarget<'_>,
    ) {
        if let oxc_ast_types::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
            self.push_assignment_target(ident.span.start);
        }
    }
}
