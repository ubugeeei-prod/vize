//! `codegen::slots::params::prefix_slot_defaults`, ported: default
//! values inside a slot-props destructuring pattern get `_ctx.` on their
//! free identifiers (`{ item = defaultItem }` →
//! `{ item = _ctx.defaultItem }`), in every mode — the shipped codegen
//! runs it unconditionally and knows no binding metadata there.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::{BindingPattern, Expression};
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_function, walk_object_property},
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::expression_guard::expression_is_safe_to_parse;
use vize_s0::{Allocator, String};

use super::globals::is_global_allowed;
use crate::emit::js_comment::RawJs;

/// Borrowed when nothing was rewritten: a pattern without `=` has no
/// default to prefix, so the default lane (the allocation gate's window)
/// never parses it.
pub(in crate::emit) fn prefix_slot_defaults(source: &str) -> RawJs<'_> {
    if !source.contains('=') || !expression_is_safe_to_parse(source) {
        return RawJs::Borrowed(source);
    }
    let mut wrapped = String::with_capacity(source.len() + 10);
    wrapped.push('(');
    wrapped.push_str(source);
    wrapped.push_str(") => null");

    let allocator = Allocator::new();
    let parser = Parser::new(
        allocator.as_oxc(),
        wrapped.as_str(),
        SourceType::ts().with_module(true),
    );
    let Ok(Expression::ArrowFunctionExpression(arrow)) = parser.parse_expression() else {
        return RawJs::Borrowed(source);
    };

    let mut slot_params = StdVec::new();
    for param in &arrow.params.items {
        collect_binding_names(&param.pattern, &mut slot_params);
    }
    let mut visitor = SlotDefaultPrefixVisitor {
        local_scopes: alloc::vec![slot_params],
        offset: 1,
        insertions: StdVec::new(),
    };
    for param in &arrow.params.items {
        collect_default_rewrites(&param.pattern, &mut visitor);
    }
    if visitor.insertions.is_empty() {
        return RawJs::Borrowed(source);
    }
    // The shipped loop: descending positions, `insert_str` each (an
    // insertion beyond the text is dropped by the `pos <= len` guard).
    visitor
        .insertions
        .sort_by_key(|(pos, _)| core::cmp::Reverse(*pos));
    let mut result = String::from(source);
    for (pos, text) in visitor.insertions {
        if pos <= result.len() {
            result.insert_str(pos, text.as_str());
        }
    }
    RawJs::Owned(result)
}

struct SlotDefaultPrefixVisitor {
    local_scopes: StdVec<StdVec<String>>,
    offset: u32,
    insertions: StdVec<(usize, String)>,
}

impl SlotDefaultPrefixVisitor {
    fn push_scope(&mut self) {
        self.local_scopes.push(StdVec::new());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn is_local(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.iter().any(|local| local.as_str() == name))
    }

    fn collect_function_params(&mut self, params: &oxc_ast::ast::FormalParameters<'_>) {
        if let Some(scope) = self.local_scopes.last_mut() {
            for param in &params.items {
                collect_binding_names(&param.pattern, scope);
            }
            if let Some(rest) = &params.rest {
                collect_binding_names(&rest.rest.argument, scope);
            }
        }
    }

    fn push_prefix(&mut self, span_start: u32) {
        let pos = span_start.saturating_sub(self.offset) as usize;
        self.insertions.push((pos, String::from("_ctx.")));
    }
}

impl<'a> Visit<'a> for SlotDefaultPrefixVisitor {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'a>) {
        let name = ident.name.as_str();
        if !self.is_local(name) && !is_global_allowed(name) {
            self.push_prefix(ident.span.start);
        }
    }

    fn visit_object_property(&mut self, prop: &oxc_ast::ast::ObjectProperty<'a>) {
        if prop.shorthand
            && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &prop.key
        {
            let name = ident.name.as_str();
            if !self.is_local(name) && !is_global_allowed(name) {
                let pos = ident.span.end.saturating_sub(self.offset) as usize;
                let mut suffix = String::with_capacity(name.len() + 8);
                suffix.push_str(": _ctx.");
                suffix.push_str(name);
                self.insertions.push((pos, suffix));
                return;
            }
        }
        walk_object_property(self, prop);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.push_scope();
        self.collect_function_params(&arrow.params);
        walk_arrow_function_expression(self, arrow);
        self.pop_scope();
    }

    fn visit_function(
        &mut self,
        func: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        self.push_scope();
        self.collect_function_params(&func.params);
        walk_function(self, func, flags);
        self.pop_scope();
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        collect_default_rewrites(&declarator.id, self);
        if let Some(scope) = self.local_scopes.last_mut() {
            collect_binding_names(&declarator.id, scope);
        }
    }
}

fn collect_default_rewrites(pattern: &BindingPattern<'_>, visitor: &mut SlotDefaultPrefixVisitor) {
    match pattern {
        BindingPattern::BindingIdentifier(_) => {}
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_default_rewrites(&prop.value, visitor);
            }
            if let Some(rest) = &obj.rest {
                collect_default_rewrites(&rest.argument, visitor);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_default_rewrites(elem, visitor);
            }
            if let Some(rest) = &arr.rest {
                collect_default_rewrites(&rest.argument, visitor);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            visitor.visit_expression(&assign.right);
            collect_default_rewrites(&assign.left, visitor);
        }
    }
}

fn collect_binding_names(pattern: &BindingPattern<'_>, names: &mut StdVec<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => names.push(String::from(id.name.as_str())),
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names(elem, names);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => collect_binding_names(&assign.left, names),
    }
}

#[cfg(test)]
mod tests {
    use super::prefix_slot_defaults;

    #[test]
    fn defaults_get_ctx_and_params_stay_local() {
        assert_eq!(
            prefix_slot_defaults("{ item = defaultItem, n = Math.max(a, 1) }").as_str(),
            "{ item = _ctx.defaultItem, n = Math.max(_ctx.a, 1) }"
        );
        assert_eq!(prefix_slot_defaults("{ item }").as_str(), "{ item }");
        assert_eq!(prefix_slot_defaults("props").as_str(), "props");
        assert_eq!(
            prefix_slot_defaults("{ a = { b } }").as_str(),
            "{ a = { b: _ctx.b } }"
        );
    }
}
