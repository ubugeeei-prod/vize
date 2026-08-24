//! Assignment-target position collection for [`IdentifierCollector`]
//! (split from `collector.rs` under the source budget).

use oxc_ast::ast as oxc_ast_types;

use super::collector::IdentifierCollector;

impl<'a, 'ctx> IdentifierCollector<'a, 'ctx> {
    pub(super) fn collect_object_assignment_target(
        &mut self,
        obj: &oxc_ast_types::ObjectAssignmentTarget<'_>,
    ) {
        use oxc_ast_types::AssignmentTargetProperty;

        for prop in &obj.properties {
            match prop {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(prop_ident) => {
                    self.assignment_targets
                        .insert(prop_ident.binding.span.start as usize);
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

    pub(super) fn collect_assignment_targets_maybe_default(
        &mut self,
        target: &oxc_ast_types::AssignmentTargetMaybeDefault<'_>,
    ) {
        use oxc_ast_types::AssignmentTargetMaybeDefault;

        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(def) => {
                self.collect_assignment_targets(&def.binding);
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                self.assignment_targets.insert(ident.span.start as usize);
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
        use oxc_ast_types::SimpleAssignmentTarget;

        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
            self.assignment_targets.insert(ident.span.start as usize);
        }
    }
}
