//! Slot parameter helpers (scoped slot props parsing and prefixing).

use crate::codegen::context::CodegenContext;
use crate::options::BindingType;
use crate::{DirectiveNode, ExpressionNode};
use oxc_ast::ast::{BindingPattern, FormalParameters};
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_function, walk_object_property},
};
use vize_croquis::builtins::is_global_allowed;
use vize_l0::{FxHashSet, String, ToCompactString};

/// Get slot props expression as raw source (not transformed)
pub(super) fn get_slot_props(dir: &DirectiveNode<'_>, source: &str) -> Option<String> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(s) => String::new(s.loc.span.slice(source)),
        ExpressionNode::Compound(c) => String::new(c.loc.span.slice(source)),
    })
}

/// The original slot carrier is the sole declaration/default-RHS authority.
/// A refused or stale carrier emits the original text without synthesizing
/// bindings or reparsing a parameter wrapper.
pub(super) fn slot_parameters(
    dir: &DirectiveNode<'_>,
    ctx: &CodegenContext,
) -> Option<(String, Vec<String>)> {
    let raw = get_slot_props(dir, &ctx.source)?;
    let Some(ExpressionNode::Simple(node)) = &dir.exp else {
        return Some((raw, vec![]));
    };
    let Some(retained) = node.js_ast else {
        return Some((raw, vec![]));
    };
    if retained.raw != node.content {
        return Some((raw, vec![]));
    }
    let Some(Ok(parameters)) = retained.as_slot_parameters() else {
        return Some((raw, vec![]));
    };
    Some(prefix_retained_defaults(retained.raw, parameters, ctx))
}

#[cfg(test)]
pub(super) fn prefix_slot_defaults(source: &str) -> String {
    prefix_slot_defaults_with_context(source, &CodegenContext::new(Default::default()))
}

#[cfg(test)]
fn prefix_slot_defaults_with_context(source: &str, ctx: &CodegenContext) -> String {
    let allocator = oxc_allocator::Allocator::default();
    let parameters = oxc_parser::Parser::new(
        &allocator,
        source,
        oxc_span::SourceType::ts().with_module(true),
    )
    .parse_slot_parameters()
    .expect("authored parameter grammar");
    prefix_retained_defaults(source, &parameters, ctx).0
}

fn prefix_retained_defaults(
    source: &str,
    parameters: &FormalParameters<'_>,
    ctx: &CodegenContext,
) -> (String, Vec<String>) {
    let mut slot_params = FxHashSet::default();
    for parameter in &parameters.items {
        collect_binding_names(&parameter.pattern, &mut slot_params);
    }
    if let Some(rest) = &parameters.rest {
        collect_binding_names(&rest.rest.argument, &mut slot_params);
    }
    let mut names: Vec<_> = slot_params.iter().cloned().collect();
    names.sort();
    let mut visitor = SlotDefaultPrefixVisitor::new(slot_params, 0, ctx);
    for parameter in &parameters.items {
        collect_default_rewrites(&parameter.pattern, &mut visitor);
        if let Some(initializer) = &parameter.initializer {
            visitor.visit_expression(initializer);
        }
    }
    if let Some(rest) = &parameters.rest {
        collect_default_rewrites(&rest.rest.argument, &mut visitor);
    }

    if visitor.insertions.is_empty() {
        return (String::new(source), names);
    }

    visitor
        .insertions
        .sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));

    let mut result = String::new(source);
    for (pos, text) in visitor.insertions {
        if pos <= result.len() {
            result.insert_str(pos, &text);
        }
    }
    (result, names)
}

struct SlotDefaultPrefixVisitor<'ctx> {
    ctx: &'ctx CodegenContext,
    local_scopes: std::vec::Vec<FxHashSet<String>>,
    offset: u32,
    insertions: std::vec::Vec<(usize, String)>,
}

impl<'ctx> SlotDefaultPrefixVisitor<'ctx> {
    fn new(slot_params: FxHashSet<String>, offset: u32, ctx: &'ctx CodegenContext) -> Self {
        Self {
            ctx,
            local_scopes: vec![slot_params],
            offset,
            insertions: std::vec::Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(FxHashSet::default());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn is_local(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn should_prefix(&self, name: &str) -> bool {
        if self.is_local(name) || is_global_allowed(name) {
            return false;
        }
        let inline_constant = self.ctx.options.inline
            && self
                .ctx
                .options
                .binding_metadata
                .as_ref()
                .and_then(|metadata| metadata.bindings.get(name))
                .is_some_and(|kind| {
                    matches!(kind, BindingType::LiteralConst | BindingType::SetupConst)
                });
        !inline_constant
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
        self.insertions.push((pos, String::new("_ctx.")));
    }
}

impl<'a> Visit<'a> for SlotDefaultPrefixVisitor<'_> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'a>) {
        let name = ident.name.as_str();
        if self.should_prefix(name) {
            self.push_prefix(ident.span.start);
        }
    }

    fn visit_object_property(&mut self, prop: &oxc_ast::ast::ObjectProperty<'a>) {
        if prop.shorthand
            && let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &prop.key
        {
            let name = ident.name.as_str();
            if self.should_prefix(name) {
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

fn collect_default_rewrites(
    pattern: &BindingPattern<'_>,
    visitor: &mut SlotDefaultPrefixVisitor<'_>,
) {
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

fn collect_binding_names(pattern: &BindingPattern<'_>, names: &mut FxHashSet<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.insert(id.name.as_str().to_compact_string());
        }
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
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names(&assign.left, names);
        }
    }
}

#[cfg(test)]
mod setup_bindings_tests;
