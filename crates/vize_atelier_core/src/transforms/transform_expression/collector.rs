//! AST visitor for collecting identifiers that need prefixing.
//!
//! Uses the OXC `Visit` trait to walk parsed expressions and determine
//! which identifiers need `_ctx.`, `$setup.`, or `.value` rewrites.

use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{
    walk::{walk_assignment_expression, walk_object_property, walk_update_expression},
    Visit,
};
use vize_carton::FxHashSet;
use vize_carton::String;

use vize_croquis::builtins::is_global_allowed;

use crate::transform::TransformContext;

use super::prefix::get_identifier_prefix;

/// Visitor to collect identifiers that need prefixing
pub(crate) struct IdentifierCollector<'a, 'ctx> {
    pub(crate) ctx: &'a TransformContext<'ctx>,
    /// The wrapped source text (for scanning paren positions)
    pub(crate) source: &'a str,
    /// Identifiers that are being declared (e.g., in arrow function params)
    pub(crate) local_scope: FxHashSet<String>,
    /// (position, prefix) pairs for rewrites
    pub(crate) rewrites: FxHashSet<(usize, String)>,
    /// (position, suffix) pairs for suffix rewrites (e.g., .value for refs)
    pub(crate) suffix_rewrites: Vec<(usize, String)>,
    /// Assignment target identifier positions (for .value on LHS)
    pub(crate) assignment_targets: FxHashSet<usize>,
    /// Whether _unref helper was used
    pub(crate) used_unref: bool,
}

impl<'a, 'ctx> IdentifierCollector<'a, 'ctx> {
    pub(crate) fn new(ctx: &'a TransformContext<'ctx>, source: &'a str) -> Self {
        Self {
            ctx,
            source,
            local_scope: FxHashSet::default(),
            rewrites: FxHashSet::default(),
            suffix_rewrites: Vec::new(),
            assignment_targets: FxHashSet::default(),
            used_unref: false,
        }
    }

    /// Check if an identifier is a ref that needs .value suffix
    fn is_ref_binding(&self, name: &str) -> bool {
        // Skip if in local scope
        if self.local_scope.contains(name) {
            return false;
        }

        // Check if this is an inline mode ref binding
        if self.ctx.options.inline {
            // Croquis first: use ReactiveKind for precise determination
            if let Some(kind) = self.ctx.get_reactive_kind(name) {
                return kind.needs_value_access();
            }
            // Fallback: binding_metadata
            if let Some(bindings) = &self.ctx.options.binding_metadata {
                if let Some(binding_type) = bindings.bindings.get(name) {
                    return matches!(binding_type, crate::options::BindingType::SetupRef);
                }
            }
        }
        false
    }

    /// Check if an identifier needs _unref() wrapping.
    ///
    /// This applies to let/var declarations and maybe-ref bindings.
    fn needs_unref(&self, name: &str) -> bool {
        // Skip if in local scope
        if self.local_scope.contains(name) {
            return false;
        }

        // If Croquis has ReactiveKind info, type is known -- no _unref() needed
        if self.ctx.get_reactive_kind(name).is_some() {
            return false;
        }

        // Fallback: SetupLet/SetupMaybeRef have unknown type, need _unref()
        if let Some(bindings) = &self.ctx.options.binding_metadata {
            if let Some(binding_type) = bindings.bindings.get(name) {
                return matches!(
                    binding_type,
                    crate::options::BindingType::SetupLet
                        | crate::options::BindingType::SetupMaybeRef
                );
            }
        }
        false
    }

    pub(crate) fn collect_binding_pattern(&mut self, pattern: &oxc_ast_types::BindingPattern<'_>) {
        match pattern {
            oxc_ast_types::BindingPattern::BindingIdentifier(id) => {
                self.local_scope.insert(String::new(id.name.as_str()));
            }
            oxc_ast_types::BindingPattern::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    self.collect_binding_pattern(&prop.value);
                }
                if let Some(rest) = &obj.rest {
                    self.collect_binding_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::ArrayPattern(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.collect_binding_pattern(elem);
                }
                if let Some(rest) = &arr.rest {
                    self.collect_binding_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::AssignmentPattern(assign) => {
                self.collect_binding_pattern(&assign.left);
            }
        }
    }

    fn collect_assignment_targets(&mut self, target: &oxc_ast_types::AssignmentTarget<'_>) {
        use oxc_ast_types::{AssignmentTarget, AssignmentTargetProperty};

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
        target: &oxc_ast_types::AssignmentTargetMaybeDefault<'_>,
    ) {
        use oxc_ast_types::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};

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
        target: &oxc_ast_types::SimpleAssignmentTarget<'_>,
    ) {
        use oxc_ast_types::SimpleAssignmentTarget;

        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
            self.assignment_targets.insert(ident.span.start as usize);
        }
    }
}

impl<'a, 'ctx> Visit<'_> for IdentifierCollector<'a, 'ctx> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast_types::IdentifierReference<'_>) {
        let name = ident.name.as_str();
        // Skip if in local scope
        if self.local_scope.contains(name) {
            return;
        }

        let needs_unref = self.needs_unref(name);
        let is_assignment_target = self
            .assignment_targets
            .contains(&(ident.span.start as usize));

        if is_assignment_target {
            if let Some(prefix) = get_identifier_prefix(name, self.ctx) {
                self.rewrites
                    .insert((ident.span.start as usize, String::new(prefix)));
            }
            if self.is_ref_binding(name) || needs_unref {
                // For assignment targets wrapped in parens like ((model) = $event),
                // we need to place .value after the closing paren: ((model).value = $event)
                // Scan forward from ident.span.end to skip past ')' characters
                let mut pos = ident.span.end as usize;
                let source_bytes = self.source.as_bytes();
                while pos < source_bytes.len() && source_bytes[pos] == b')' {
                    pos += 1;
                }
                self.suffix_rewrites.push((pos, String::new(".value")));
            }
            return;
        }

        if let Some(prefix) = get_identifier_prefix(name, self.ctx) {
            // In function mode, SetupLet bindings need both $setup. prefix and _unref() wrapper
            // Result: _unref($setup.b) instead of just $setup.b
            if needs_unref && prefix == "$setup." {
                self.rewrites
                    .insert((ident.span.start as usize, String::new("_unref($setup.")));
                self.suffix_rewrites
                    .push((ident.span.end as usize, String::new(")")));
                self.used_unref = true;
            } else {
                self.rewrites
                    .insert((ident.span.start as usize, String::new(prefix)));
            }
        } else if self.is_ref_binding(name) {
            // Add .value suffix for refs in inline mode
            self.suffix_rewrites
                .push((ident.span.end as usize, String::new(".value")));
        } else if needs_unref {
            // Wrap with _unref() for let/var bindings (inline mode)
            self.rewrites
                .insert((ident.span.start as usize, String::new("_unref(")));
            self.suffix_rewrites
                .push((ident.span.end as usize, String::new(")")));
            self.used_unref = true;
        }
    }

    fn visit_member_expression(&mut self, expr: &oxc_ast_types::MemberExpression<'_>) {
        // Visit the object part, but skip .value addition if already accessing .value
        match expr {
            oxc_ast_types::MemberExpression::ComputedMemberExpression(computed) => {
                self.visit_expression(&computed.object);
                // For computed access [expr], visit the expression normally
                self.visit_expression(&computed.expression);
            }
            oxc_ast_types::MemberExpression::StaticMemberExpression(static_expr) => {
                // If this is `ref.value`, don't add another .value to the ref object
                let property_name = static_expr.property.name.as_str();
                if property_name == "value" {
                    // Check if object is a simple identifier that is a ref
                    if let oxc_ast_types::Expression::Identifier(ident) = &static_expr.object {
                        let name = ident.name.as_str();
                        if self.is_ref_binding(name) {
                            // Skip adding .value - it's already accessed via .value
                            // But still add _ctx. prefix if needed
                            if let Some(prefix) = get_identifier_prefix(name, self.ctx) {
                                self.rewrites
                                    .insert((ident.span.start as usize, String::new(prefix)));
                            }
                            return;
                        }
                    }
                }
                self.visit_expression(&static_expr.object);
                // Don't visit the property - it's a static name, not a reference
            }
            oxc_ast_types::MemberExpression::PrivateFieldExpression(private) => {
                self.visit_expression(&private.object);
                // Private field name shouldn't be prefixed
            }
        }
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast_types::ArrowFunctionExpression<'_>,
    ) {
        // Add params to local scope
        for param in &arrow.params.items {
            self.collect_binding_pattern(&param.pattern);
        }
        if let Some(rest) = &arrow.params.rest {
            self.collect_binding_pattern(&rest.rest.argument);
        }

        // Visit body
        self.visit_function_body(&arrow.body);
    }

    fn visit_assignment_expression(&mut self, expr: &oxc_ast_types::AssignmentExpression<'_>) {
        self.collect_assignment_targets(&expr.left);
        walk_assignment_expression(self, expr);
    }

    fn visit_update_expression(&mut self, expr: &oxc_ast_types::UpdateExpression<'_>) {
        self.collect_simple_assignment_targets(&expr.argument);
        walk_update_expression(self, expr);
    }

    fn visit_object_property(&mut self, prop: &oxc_ast_types::ObjectProperty<'_>) {
        if prop.shorthand {
            if let oxc_ast_types::PropertyKey::StaticIdentifier(ident) = &prop.key {
                let name = ident.name.as_str();
                if self.local_scope.contains(name) || is_global_allowed(name) {
                    return;
                }
                if self.ctx.is_in_scope(name) {
                    return;
                }

                let prefix = get_identifier_prefix(name, self.ctx);
                let is_ref = self.is_ref_binding(name);
                let needs_unref = self.needs_unref(name);

                // Expand shorthand if identifier needs a prefix, is a ref binding,
                // or needs _unref() wrapping.
                // In inline mode, refs have no prefix but need .value, so shorthand
                // { hasForm } must become { hasForm: hasForm.value } (not { hasForm.value }).
                // Similarly, SetupLet/SetupMaybeRef bindings need _unref():
                // { paddingBottom } must become { paddingBottom: _unref(paddingBottom) }
                if prefix.is_some_and(|p| !p.is_empty()) || is_ref || needs_unref {
                    let p = prefix.unwrap_or("");
                    let (value_prefix, value_suffix) = if needs_unref && p.is_empty() {
                        // Inline mode: wrap with _unref()
                        ("_unref(", ")")
                    } else if needs_unref && p == "$setup." {
                        // Function mode: wrap with _unref($setup.)
                        ("_unref($setup.", ")")
                    } else if is_ref {
                        ("", ".value")
                    } else {
                        ("", "")
                    };
                    let mut suffix = String::with_capacity(
                        2 + value_prefix.len() + p.len() + name.len() + value_suffix.len(),
                    );
                    suffix.push_str(": ");
                    suffix.push_str(value_prefix);
                    if !needs_unref {
                        suffix.push_str(p);
                    }
                    suffix.push_str(name);
                    suffix.push_str(value_suffix);
                    self.suffix_rewrites.push((ident.span.end as usize, suffix));
                    if needs_unref {
                        self.used_unref = true;
                    }
                    return;
                }
            }
        }

        walk_object_property(self, prop);
    }
}
