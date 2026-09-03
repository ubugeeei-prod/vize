//! The codegen-time prefix visitor (`codegen::expression::prefix_visitor`
//! and `prefix_context`), which the shipped lane runs over dynamic
//! directive arguments: whole-span replacements, locals from declarators
//! and plain arrow params only, slot params skipped.

use alloc::vec::Vec as StdVec;

use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_assignment_expression, walk_object_property, walk_update_expression,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{Allocator, String};

use super::compat::js_module_compatible;
use super::globals::is_global_allowed;
use super::rewrite::Retained;
use super::scope::PrefixScope;
use super::splice::splice_replacements;

struct IdentifierVisitor<'s> {
    scope: &'s PrefixScope,
    rewrites: StdVec<(usize, usize, String)>,
    local_vars: StdVec<String>,
    assignment_targets: StdVec<usize>,
    offset: u32,
}

impl IdentifierVisitor<'_> {
    fn is_local(&self, name: &str) -> bool {
        self.local_vars.iter().any(|local| local.as_str() == name)
    }

    fn collect_assignment_targets(&mut self, target: &oxc_ast::ast::AssignmentTarget<'_>) {
        use oxc_ast::ast::{AssignmentTarget, AssignmentTargetProperty};
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.assignment_targets.push(ident.span.start as usize);
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                            prop_ident,
                        ) => {
                            self.assignment_targets
                                .push(prop_ident.binding.span.start as usize);
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
                self.assignment_targets.push(ident.span.start as usize);
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                            prop_ident,
                        ) => {
                            self.assignment_targets
                                .push(prop_ident.binding.span.start as usize);
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
}

impl Visit<'_> for IdentifierVisitor<'_> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'_>) {
        let name = ident.name.as_str();
        if self.is_local(name) || is_global_allowed(name) || self.scope.is_slot_param(name) {
            return;
        }
        let start = (ident.span.start - self.offset) as usize;
        let end = (ident.span.end - self.offset) as usize;
        let mut replacement = String::with_capacity(5 + name.len());
        replacement.push_str("_ctx.");
        replacement.push_str(name);
        self.rewrites.push((start, end, replacement));
    }

    fn visit_assignment_expression(&mut self, expr: &oxc_ast::ast::AssignmentExpression<'_>) {
        self.collect_assignment_targets(&expr.left);
        walk_assignment_expression(self, expr);
    }

    fn visit_update_expression(&mut self, expr: &oxc_ast::ast::UpdateExpression<'_>) {
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) =
            &expr.argument
        {
            self.assignment_targets.push(ident.span.start as usize);
        }
        walk_update_expression(self, expr);
    }

    fn visit_object_property(&mut self, prop: &oxc_ast::ast::ObjectProperty<'_>) {
        if prop.shorthand
            && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &prop.key
        {
            let name = ident.name.as_str();
            if self.is_local(name) || is_global_allowed(name) || self.scope.is_slot_param(name) {
                return;
            }
            let start = (prop.span.start - self.offset) as usize;
            let end = (prop.span.end - self.offset) as usize;
            let mut replacement = String::with_capacity(name.len() * 2 + 7);
            replacement.push_str(name);
            replacement.push_str(": _ctx.");
            replacement.push_str(name);
            self.rewrites.push((start, end, replacement));
            return;
        }
        walk_object_property(self, prop);
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast::ast::VariableDeclarator<'_>) {
        if let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &declarator.id {
            self.local_vars.push(String::from(ident.name.as_str()));
        }
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'_>,
    ) {
        for param in &arrow.params.items {
            if let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &param.pattern {
                self.local_vars.push(String::from(ident.name.as_str()));
            }
        }
        self.visit_function_body(&arrow.body);
    }
}

fn prefix_via_expr(
    expr: &oxc_ast::ast::Expression<'_>,
    offset: u32,
    content: &str,
    scope: &PrefixScope,
) -> String {
    let mut visitor = IdentifierVisitor {
        scope,
        rewrites: StdVec::new(),
        local_vars: StdVec::new(),
        assignment_targets: StdVec::new(),
        offset,
    };
    visitor.visit_expression(expr);
    splice_replacements(content, visitor.rewrites)
}

/// `prefix_identifiers_with_context`: wrapped expression parse, then a
/// program parse, then the raw text.
pub(super) fn prefix_identifiers_with_context(content: &str, scope: &PrefixScope) -> String {
    let source_type = SourceType::default().with_module(true);
    let allocator = Allocator::new();
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    if let Ok(expr) =
        Parser::new(allocator.as_oxc(), wrapped.as_str(), source_type).parse_expression()
    {
        return prefix_via_expr(&expr, 1, content, scope);
    }
    let program_allocator = Allocator::new();
    let parsed = Parser::new(program_allocator.as_oxc(), content, source_type).parse();
    if !parsed.diagnostics.is_empty() {
        return String::from(content);
    }
    let mut visitor = IdentifierVisitor {
        scope,
        rewrites: StdVec::new(),
        local_vars: StdVec::new(),
        assignment_targets: StdVec::new(),
        offset: 0,
    };
    visitor.visit_program(&parsed.program);
    splice_replacements(content, visitor.rewrites)
}

/// `prefix_identifiers_with_context_node`: the retained AST when the
/// dialect gate holds, the string entry otherwise.
pub(super) fn prefix_identifiers_with_context_node(
    content: &str,
    retained: Option<Retained<'_, '_>>,
    scope: &PrefixScope,
) -> String {
    if let Some(retained) = retained
        && retained.offset == 0
        && js_module_compatible(retained.ast, retained.source)
    {
        return prefix_via_expr(retained.ast, 0, content, scope);
    }
    prefix_identifiers_with_context(content, scope)
}
