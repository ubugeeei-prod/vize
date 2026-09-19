//! Lexical binding ownership for the context-aware codegen prefix visitor.

use oxc_ast::ast;
use oxc_ast_visit::walk;
use oxc_syntax::scope::ScopeFlags;
use vize_s0::{FxHashSet, String};

use super::prefix_visitor::IdentifierVisitor;

impl IdentifierVisitor<'_, '_> {
    pub(super) fn add_local(&mut self, name: &str) {
        self.local_scopes
            .last_mut()
            .unwrap()
            .insert(String::new(name));
    }

    pub(super) fn is_local(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn collect_binding_pattern(&mut self, pattern: &ast::BindingPattern<'_>) {
        // OXC visits nested object/array/default/rest bindings without allocating
        // another identifier list. Always continue through the entire pattern.
        pattern.all_binding_identifiers(&mut |identifier| {
            self.add_local(identifier.name.as_str());
            true
        });
    }

    pub(super) fn visit_arrow_with_scope(&mut self, arrow: &ast::ArrowFunctionExpression<'_>) {
        self.local_scopes.push(FxHashSet::default());
        for pattern in arrow.params.iter_bindings() {
            self.collect_binding_pattern(pattern);
        }
        walk::walk_arrow_function_expression(self, arrow);
        self.local_scopes.pop();
    }

    pub(super) fn visit_function_with_scope(
        &mut self,
        function: &ast::Function<'_>,
        flags: ScopeFlags,
    ) {
        if function.r#type == ast::FunctionType::FunctionDeclaration
            && let Some(id) = &function.id
        {
            self.add_local(id.name.as_str());
        }
        self.local_scopes.push(FxHashSet::default());
        if let Some(id) = &function.id {
            self.add_local(id.name.as_str());
        }
        for pattern in function.params.iter_bindings() {
            self.collect_binding_pattern(pattern);
        }
        walk::walk_function(self, function, flags);
        self.local_scopes.pop();
    }

    pub(super) fn visit_body_with_scope(&mut self, body: &ast::FunctionBody<'_>) {
        vize_relief::for_each_function_var(body, |pattern| self.collect_binding_pattern(pattern));
        walk::walk_function_body(self, body);
    }

    pub(super) fn visit_block_with_scope(&mut self, block: &ast::BlockStatement<'_>) {
        self.local_scopes.push(FxHashSet::default());
        walk::walk_block_statement(self, block);
        self.local_scopes.pop();
    }

    pub(super) fn visit_catch_with_scope(&mut self, clause: &ast::CatchClause<'_>) {
        self.local_scopes.push(FxHashSet::default());
        if let Some(parameter) = &clause.param {
            self.collect_binding_pattern(&parameter.pattern);
        }
        walk::walk_catch_clause(self, clause);
        self.local_scopes.pop();
    }

    pub(super) fn visit_declarator_with_scope(&mut self, declarator: &ast::VariableDeclarator<'_>) {
        self.collect_binding_pattern(&declarator.id);
        walk::walk_variable_declarator(self, declarator);
    }
}
