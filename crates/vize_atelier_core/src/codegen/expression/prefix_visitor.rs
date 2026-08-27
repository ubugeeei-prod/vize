//! The codegen identifier-prefix visitor (split from `prefix_context.rs`
//! under the source budget). Behavior is byte-identical to the pre-split
//! `helpers.rs` visitor; only visibility changed for the cross-file
//! construction.

use crate::options::BindingType;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_assignment_expression, walk_object_property, walk_update_expression,
};
use vize_croquis::builtins::is_global_allowed;
use vize_s0::FxHashSet;
use vize_s0::String;
use vize_s0::ToCompactString;

use super::super::context::CodegenContext;

// Visitor to collect identifiers and rewrite them with appropriate prefixes / .value
pub(super) struct IdentifierVisitor<'a, 'b> {
    pub(super) rewrites: &'a mut Vec<(usize, usize, String)>,
    pub(super) local_vars: &'a mut FxHashSet<String>,
    pub(super) assignment_targets: &'a mut FxHashSet<usize>,
    pub(super) ctx: &'b CodegenContext,
    pub(super) offset: u32,
}

impl<'a, 'b> Visit<'_> for IdentifierVisitor<'a, 'b> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'_>) {
        let name = ident.name.as_str();

        // Skip if local variable
        if self.local_vars.contains(name) {
            return;
        }

        // Skip globals
        if is_global_allowed(name) {
            return;
        }

        // Skip slot params
        if self.ctx.is_slot_param(name) {
            return;
        }

        let is_assignment_target = self
            .assignment_targets
            .contains(&(ident.span.start as usize));

        // Determine prefix based on binding metadata
        let mut binding_type: Option<BindingType> = None;
        let prefix = if let Some(ref metadata) = self.ctx.options.binding_metadata {
            if let Some(binding) = metadata.bindings.get(name) {
                binding_type = Some(*binding);
                if self.ctx.options.inline {
                    match binding {
                        BindingType::Props | BindingType::PropsAliased => "$props.",
                        _ => "",
                    }
                } else {
                    binding.non_inline_template_prefix()
                }
            } else {
                "_ctx."
            }
        } else {
            "_ctx."
        };

        if is_assignment_target {
            let needs_value = self.ctx.options.inline
                && matches!(
                    binding_type,
                    Some(
                        BindingType::SetupLet | BindingType::SetupMaybeRef | BindingType::SetupRef
                    )
                );
            let replacement = if needs_value {
                let mut out = String::with_capacity(prefix.len() + name.len() + 6);
                out.push_str(prefix);
                out.push_str(name);
                out.push_str(".value");
                out
            } else if !prefix.is_empty() {
                let mut out = String::with_capacity(prefix.len() + name.len());
                out.push_str(prefix);
                out.push_str(name);
                out
            } else {
                name.to_compact_string()
            };
            if replacement != name {
                let start = (ident.span.start - self.offset) as usize;
                let end = (ident.span.end - self.offset) as usize;
                self.rewrites.push((start, end, replacement));
            }
            return;
        }

        if !prefix.is_empty() {
            let start = (ident.span.start - self.offset) as usize;
            let end = (ident.span.end - self.offset) as usize;
            let mut replacement = String::with_capacity(prefix.len() + name.len());
            replacement.push_str(prefix);
            replacement.push_str(name);
            self.rewrites.push((start, end, replacement));
        }
    }

    fn visit_assignment_expression(&mut self, expr: &oxc_ast::ast::AssignmentExpression<'_>) {
        self.collect_assignment_targets(&expr.left);
        walk_assignment_expression(self, expr);
    }

    fn visit_update_expression(&mut self, expr: &oxc_ast::ast::UpdateExpression<'_>) {
        self.collect_simple_assignment_targets(&expr.argument);
        walk_update_expression(self, expr);
    }

    fn visit_object_property(&mut self, prop: &oxc_ast::ast::ObjectProperty<'_>) {
        if prop.shorthand
            && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &prop.key
        {
            let name = ident.name.as_str();

            // Skip if local variable, global, or slot param
            if self.local_vars.contains(name)
                || is_global_allowed(name)
                || self.ctx.is_slot_param(name)
            {
                return;
            }

            let mut is_ref = false;
            let mut needs_unref = false;
            let prefix = if let Some(ref metadata) = self.ctx.options.binding_metadata {
                if let Some(binding_type) = metadata.bindings.get(name) {
                    is_ref =
                        self.ctx.options.inline && matches!(binding_type, BindingType::SetupRef);
                    needs_unref = self.ctx.options.inline
                        && matches!(
                            binding_type,
                            BindingType::SetupLet | BindingType::SetupMaybeRef
                        );
                    if self.ctx.options.inline {
                        match binding_type {
                            BindingType::Props | BindingType::PropsAliased => "$props.",
                            _ => "",
                        }
                    } else {
                        binding_type.non_inline_template_prefix()
                    }
                } else {
                    "_ctx."
                }
            } else {
                "_ctx."
            };

            if !prefix.is_empty() || is_ref || needs_unref {
                let start = (prop.span.start - self.offset) as usize;
                let end = (prop.span.end - self.offset) as usize;
                let (value_prefix, value_suffix) = if needs_unref {
                    ("_unref(", ")")
                } else if is_ref {
                    ("", ".value")
                } else {
                    ("", "")
                };
                let mut replacement = String::with_capacity(
                    name.len()
                        + 2
                        + value_prefix.len()
                        + prefix.len()
                        + name.len()
                        + value_suffix.len(),
                );
                replacement.push_str(name);
                replacement.push_str(": ");
                replacement.push_str(value_prefix);
                if !needs_unref {
                    replacement.push_str(prefix);
                }
                replacement.push_str(name);
                replacement.push_str(value_suffix);
                self.rewrites.push((start, end, replacement));
                return;
            }
        }

        walk_object_property(self, prop);
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast::ast::VariableDeclarator<'_>) {
        // Add local var names to skip list
        if let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &declarator.id {
            self.local_vars.insert(ident.name.to_compact_string());
        }
        // Visit init expression
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'_>,
    ) {
        // Add arrow function params to local vars
        for param in &arrow.params.items {
            if let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &param.pattern {
                self.local_vars.insert(ident.name.to_compact_string());
            }
        }
        // Visit body
        self.visit_function_body(&arrow.body);
    }
}

impl<'a, 'b> IdentifierVisitor<'a, 'b> {
    fn collect_assignment_targets(&mut self, target: &oxc_ast::ast::AssignmentTarget<'_>) {
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

    fn collect_simple_assignment_targets(
        &mut self,
        target: &oxc_ast::ast::SimpleAssignmentTarget<'_>,
    ) {
        use oxc_ast::ast::SimpleAssignmentTarget;

        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
            self.assignment_targets.insert(ident.span.start as usize);
        }
    }
}

/// Apply collected rewrites to content and return the result
pub(super) fn apply_rewrites(content: &str, mut rewrites: Vec<(usize, usize, String)>) -> String {
    if rewrites.is_empty() {
        return content.to_compact_string();
    }
    rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));
    let mut result = content.to_compact_string();
    for (start, end, replacement) in rewrites {
        if start < result.len() && end <= result.len() {
            result.replace_range(start..end, &replacement);
        }
    }
    result
}
