//! Scope storage and assignment binding facts for codegen prefixing.

use super::prefix_visitor::IdentifierVisitor;
use vize_relief::ExpressionScope;
use vize_s0::{FxHashSet, String};

impl<'ast> ExpressionScope<'ast> for IdentifierVisitor<'_, '_> {
    fn push_scope(&mut self) {
        self.local_scopes.push(FxHashSet::default());
    }
    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }
    fn add_local(&mut self, name: &str) {
        self.local_scopes
            .last_mut()
            .expect("expression binding owns a scope")
            .insert(String::new(name));
    }
}

impl IdentifierVisitor<'_, '_> {
    pub(super) fn is_local(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }
}

impl<'a, 'b> IdentifierVisitor<'a, 'b> {
    pub(super) fn collect_assignment_targets(
        &mut self,
        target: &oxc_ast::ast::AssignmentTarget<'_>,
    ) {
        use oxc_ast::ast::{AssignmentTarget, AssignmentTargetProperty};

        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.assignment_targets.insert(ident.span.start as usize);
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                            prop_ident,
                        ) => {
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

    fn collect_assignment_targets_maybe_default(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetMaybeDefault<'_>,
    ) {
        use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};

        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(def) => {
                self.collect_assignment_targets(&def.binding);
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                self.assignment_targets.insert(ident.span.start as usize);
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                            prop_ident,
                        ) => {
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
        target: &oxc_ast::ast::SimpleAssignmentTarget<'_>,
    ) {
        use oxc_ast::ast::SimpleAssignmentTarget;

        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
            self.assignment_targets.insert(ident.span.start as usize);
        }
    }
}
