//! The codegen identifier-prefix visitor (split from `prefix_context.rs`
//! under the source budget). Callback binding ownership is shared by the
//! retained and reparsed expression paths.

use crate::options::BindingType;
use crate::steps::expression::is_template_global;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_assignment_expression, walk_object_property, walk_update_expression,
};
use oxc_syntax::scope::ScopeFlags;
use vize_relief::ExpressionScope;
use vize_s0::FxHashSet;
use vize_s0::String;
use vize_s0::ToCompactString;

use super::super::context::CodegenContext;

// Visitor to collect identifiers and rewrite them with appropriate prefixes / .value
pub(super) struct IdentifierVisitor<'a, 'b> {
    pub(super) rewrites: &'a mut Vec<(usize, usize, String)>,
    pub(super) local_scopes: Vec<FxHashSet<String>>,
    pub(super) assignment_targets: &'a mut FxHashSet<usize>,
    pub(super) ctx: &'b CodegenContext,
    pub(super) offset: u32,
}

impl<'a, 'b> Visit<'_> for IdentifierVisitor<'a, 'b> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'_>) {
        let name = ident.name.as_str();

        // Skip if local variable
        if self.is_local(name) {
            return;
        }

        // Skip globals
        if is_template_global(name) {
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
            if self.is_local(name) || is_template_global(name) || self.ctx.is_slot_param(name) {
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

    fn visit_program(&mut self, node: &oxc_ast::ast::Program<'_>) {
        self.scoped_program(node);
    }

    fn visit_arrow_function_expression(
        &mut self,
        node: &oxc_ast::ast::ArrowFunctionExpression<'_>,
    ) {
        self.scoped_arrow(node);
    }

    fn visit_function_body(&mut self, node: &oxc_ast::ast::FunctionBody<'_>) {
        self.scoped_function_body(node);
    }

    fn visit_block_statement(&mut self, node: &oxc_ast::ast::BlockStatement<'_>) {
        self.scoped_block(node);
    }

    fn visit_catch_clause(&mut self, node: &oxc_ast::ast::CatchClause<'_>) {
        self.scoped_catch(node);
    }

    fn visit_for_statement(&mut self, node: &oxc_ast::ast::ForStatement<'_>) {
        self.scoped_for(node);
    }

    fn visit_for_in_statement(&mut self, node: &oxc_ast::ast::ForInStatement<'_>) {
        self.scoped_for_in(node);
    }

    fn visit_for_of_statement(&mut self, node: &oxc_ast::ast::ForOfStatement<'_>) {
        self.scoped_for_of(node);
    }

    fn visit_switch_statement(&mut self, node: &oxc_ast::ast::SwitchStatement<'_>) {
        self.scoped_switch(node);
    }

    fn visit_class(&mut self, node: &oxc_ast::ast::Class<'_>) {
        self.scoped_class(node);
    }

    fn visit_static_block(&mut self, node: &oxc_ast::ast::StaticBlock<'_>) {
        self.scoped_static_block(node);
    }

    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'_>, flags: ScopeFlags) {
        self.scoped_function(function, flags);
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
