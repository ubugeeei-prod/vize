//! The transform-time identifier collector
//! (`vize_atelier_core::steps::expression::collector::IdentifierCollector`),
//! ported without binding metadata: every free identifier outside the
//! transform scope and the allowlist gets `_ctx.`, shorthand properties
//! expand, assignment targets prefix in place. `offset` shifts the walked
//! AST's spans into `source` (0 when the AST was parsed from `source`
//! itself), which is how a retained trimmed-text AST rewrites the padded
//! attribute value the shipped lane held as the node's content.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_assignment_expression, walk_block_statement,
        walk_catch_clause, walk_function, walk_object_property, walk_update_expression,
        walk_variable_declarator,
    },
};
use oxc_syntax::scope::ScopeFlags;
use vize_s0::String;

use super::globals::is_global_allowed;
use super::scope::PrefixScope;

pub(super) struct IdentifierCollector<'s, 'a> {
    scope: &'s PrefixScope<'s>,
    /// The walked text; the inline-mode assignment scan reads it to place
    /// `.value` after the closing parens of `((model) = $event)`.
    pub(super) source: &'a str,
    /// Whether `source` is the wrapped `(content)` parse. The shipped
    /// lane drops a suffix that lands at the very end of *unwrapped*
    /// text, because there the apply loop saw it as out of range.
    wrapped: bool,
    /// Set when a binding was read through `_unref(…)`.
    pub(super) used_unref: bool,
    offset: usize,
    local_scopes: StdVec<StdVec<String>>,
    pub(super) rewrites: StdVec<(usize, String)>,
    pub(super) suffix_rewrites: StdVec<(usize, String)>,
    pub(super) assignment_targets: StdVec<usize>,
}

impl<'s, 'a> IdentifierCollector<'s, 'a> {
    /// The legacy wrapped-parse collector: spans count the synthetic `(`.
    pub(super) fn new(scope: &'s PrefixScope<'s>, source: &'a str) -> Self {
        Self {
            scope,
            source,
            wrapped: true,
            used_unref: false,
            offset: 0,
            local_scopes: alloc::vec![StdVec::new()],
            rewrites: StdVec::new(),
            suffix_rewrites: StdVec::new(),
            assignment_targets: StdVec::new(),
        }
    }

    /// The retained-AST collector over the bare content, whose AST spans
    /// are `offset` bytes before their position in `source`.
    pub(super) fn new_unwrapped(
        scope: &'s PrefixScope<'s>,
        source: &'a str,
        offset: usize,
    ) -> Self {
        Self {
            offset,
            wrapped: false,
            ..Self::new(scope, source)
        }
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(StdVec::new());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn add_local(&mut self, name: &str) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.push(String::from(name));
        }
    }

    fn is_local(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.iter().any(|local| local.as_str() == name))
    }

    fn at(&self, position: u32) -> usize {
        position as usize + self.offset
    }

    /// An assignment target wrapped in parens — `((model) = $event)` —
    /// takes its `.value` after the closing parens, so the scan walks
    /// past them. A suffix landing at the very end of unwrapped text is
    /// dropped: the shipped apply loop saw that position as out of range.
    fn push_assignment_value_suffix(&mut self, end: usize) {
        let bytes = self.source.as_bytes();
        let mut position = end;
        while position < bytes.len() && bytes[position] == b')' {
            position += 1;
        }
        if self.wrapped || position < bytes.len() {
            self.suffix_rewrites
                .push((position, String::from(".value")));
        }
    }

    fn push_prefix(&mut self, position: usize, prefix: &'static str) {
        let entry = (position, String::from(prefix));
        if !self.rewrites.contains(&entry) {
            self.rewrites.push(entry);
        }
    }

    pub(super) fn push_assignment_target(&mut self, position: u32) {
        let position = self.at(position);
        if !self.assignment_targets.contains(&position) {
            self.assignment_targets.push(position);
        }
    }

    pub(super) fn collect_binding_pattern(&mut self, pattern: &oxc_ast_types::BindingPattern<'_>) {
        match pattern {
            oxc_ast_types::BindingPattern::BindingIdentifier(id) => {
                self.add_local(id.name.as_str());
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
}

impl<'s, 'a> Visit<'_> for IdentifierCollector<'s, 'a> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast_types::IdentifierReference<'_>) {
        let name = ident.name.as_str();
        if self.is_local(name) {
            return;
        }
        let start = self.at(ident.span.start);
        let end = self.at(ident.span.end);
        let needs_unref = self.scope.needs_unref(name);
        let is_assignment_target = self.assignment_targets.contains(&start);
        if is_assignment_target {
            if let Some(prefix) = self.scope.identifier_prefix(name) {
                self.push_prefix(start, prefix);
            }
            if self.scope.inline() && (self.scope.is_ref_binding(name) || needs_unref) {
                self.push_assignment_value_suffix(end);
            }
            return;
        }
        match self.scope.identifier_prefix(name) {
            // `_unref($setup.x)` is unreachable today — `needs_unref`
            // implies inline, where the prefix is never `$setup.` — but
            // the shipped branch order is what decides the other two.
            Some(prefix) if needs_unref && prefix == "$setup." => {
                self.push_prefix(start, "_unref($setup.");
                self.suffix_rewrites.push((end, String::from(")")));
                self.used_unref = true;
            }
            Some(prefix) => self.push_prefix(start, prefix),
            None if self.scope.is_ref_binding(name) => {
                self.suffix_rewrites.push((end, String::from(".value")));
            }
            None if needs_unref => {
                self.push_prefix(start, "_unref(");
                self.suffix_rewrites.push((end, String::from(")")));
                self.used_unref = true;
            }
            None => {}
        }
    }

    fn visit_member_expression(&mut self, expr: &oxc_ast_types::MemberExpression<'_>) {
        match expr {
            oxc_ast_types::MemberExpression::ComputedMemberExpression(computed) => {
                self.visit_expression(&computed.object);
                self.visit_expression(&computed.expression);
            }
            oxc_ast_types::MemberExpression::StaticMemberExpression(static_expr) => {
                self.visit_expression(&static_expr.object);
            }
            oxc_ast_types::MemberExpression::PrivateFieldExpression(private) => {
                self.visit_expression(&private.object);
            }
        }
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast_types::ArrowFunctionExpression<'_>,
    ) {
        self.push_scope();
        for param in &arrow.params.items {
            self.collect_binding_pattern(&param.pattern);
        }
        if let Some(rest) = &arrow.params.rest {
            self.collect_binding_pattern(&rest.rest.argument);
        }
        walk_arrow_function_expression(self, arrow);
        self.pop_scope();
    }

    fn visit_function(&mut self, func: &oxc_ast_types::Function<'_>, flags: ScopeFlags) {
        if func.r#type == oxc_ast_types::FunctionType::FunctionDeclaration
            && let Some(id) = &func.id
        {
            self.add_local(id.name.as_str());
        }
        self.push_scope();
        if let Some(id) = &func.id {
            self.add_local(id.name.as_str());
        }
        for param in &func.params.items {
            self.collect_binding_pattern(&param.pattern);
        }
        if let Some(rest) = &func.params.rest {
            self.collect_binding_pattern(&rest.rest.argument);
        }
        walk_function(self, func, flags);
        self.pop_scope();
    }

    fn visit_block_statement(&mut self, block: &oxc_ast_types::BlockStatement<'_>) {
        self.push_scope();
        walk_block_statement(self, block);
        self.pop_scope();
    }

    fn visit_catch_clause(&mut self, catch_clause: &oxc_ast_types::CatchClause<'_>) {
        self.push_scope();
        if let Some(param) = &catch_clause.param {
            self.collect_binding_pattern(&param.pattern);
        }
        walk_catch_clause(self, catch_clause);
        self.pop_scope();
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast_types::VariableDeclarator<'_>) {
        walk_variable_declarator(self, declarator);
        self.collect_binding_pattern(&declarator.id);
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
        if prop.shorthand
            && let oxc_ast_types::PropertyKey::StaticIdentifier(ident) = &prop.key
        {
            let name = ident.name.as_str();
            if self.is_local(name) || is_global_allowed(name) {
                return;
            }
            if self.scope.is_in_transform_scope(name) {
                return;
            }
            let prefix = self.scope.identifier_prefix(name);
            let is_ref = self.scope.is_ref_binding(name);
            let needs_unref = self.scope.needs_unref(name);
            // Inline mode reads a ref through `.value` and a `let` through
            // `_unref(…)` with no prefix at all, so the shorthand still has
            // to expand: `{ n }` becomes `{ n: n.value }`, never `{ n.value }`.
            if prefix.is_some_and(|prefix| !prefix.is_empty()) || is_ref || needs_unref {
                let prefix = prefix.unwrap_or("");
                let (value_prefix, value_suffix) = if needs_unref && prefix.is_empty() {
                    ("_unref(", ")")
                } else if is_ref {
                    ("", ".value")
                } else {
                    ("", "")
                };
                let mut suffix = String::with_capacity(
                    2 + value_prefix.len() + prefix.len() + name.len() + value_suffix.len(),
                );
                suffix.push_str(": ");
                suffix.push_str(value_prefix);
                if !needs_unref {
                    suffix.push_str(prefix);
                }
                suffix.push_str(name);
                suffix.push_str(value_suffix);
                self.suffix_rewrites.push((self.at(ident.span.end), suffix));
                if needs_unref {
                    self.used_unref = true;
                }
                return;
            }
        }
        walk_object_property(self, prop);
    }
}
