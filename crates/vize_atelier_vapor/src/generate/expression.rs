use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{
    walk::{
        walk_arrow_function_expression, walk_function, walk_object_property,
        walk_variable_declarator,
    },
    Visit,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{FxHashSet, String, ToCompactString};
use vize_croquis::builtins::is_global_allowed;

use super::context::GenerateContext;

pub(super) fn resolve_expression(ctx: &GenerateContext<'_>, expr: &str) -> String {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return String::default();
    }

    if is_literal_expression(trimmed) {
        return trimmed.to_compact_string();
    }

    if is_simple_path_expression(trimmed) {
        return ctx.resolve_simple_reference(trimmed);
    }

    if let Some(resolved) = resolve_with_oxc(ctx, trimmed) {
        return resolved;
    }

    ctx.resolve_complex_expression_fallback(trimmed)
}

fn resolve_with_oxc(ctx: &GenerateContext<'_>, expr: &str) -> Option<String> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);

    let mut wrapped = String::with_capacity(expr.len() + 2);
    wrapped.push('(');
    wrapped.push_str(expr);
    wrapped.push(')');

    let parser = Parser::new(&allocator, wrapped.as_str(), source_type);
    if let Ok(parsed) = parser.parse_expression() {
        let mut collector = ExpressionRewriteCollector::new(ctx);
        collector.visit_expression(&parsed);
        return Some(apply_rewrites(expr, collector.rewrites, 1));
    }

    let allocator = OxcAllocator::default();
    let parser = Parser::new(&allocator, expr, source_type);
    let parsed = parser.parse();
    if parsed.errors.is_empty() {
        let mut collector = ExpressionRewriteCollector::new(ctx);
        collector.push_scope();
        collector.visit_program(&parsed.program);
        collector.pop_scope();
        return Some(apply_rewrites(expr, collector.rewrites, 0));
    }

    None
}

fn apply_rewrites(expr: &str, mut rewrites: std::vec::Vec<Rewrite>, offset: usize) -> String {
    if rewrites.is_empty() {
        return expr.to_compact_string();
    }

    rewrites.sort_by(|a, b| {
        b.start
            .cmp(&a.start)
            .then_with(|| b.end.cmp(&a.end))
            .then_with(|| b.replacement.len().cmp(&a.replacement.len()))
    });

    let mut result = expr.to_compact_string();
    for rewrite in rewrites {
        let start = rewrite.start.saturating_sub(offset);
        let end = rewrite.end.saturating_sub(offset);
        if start <= end && end <= result.len() {
            result.replace_range(start..end, rewrite.replacement.as_str());
        }
    }
    result
}

fn is_literal_expression(expr: &str) -> bool {
    expr.parse::<f64>().is_ok()
        || matches!(expr, "true" | "false" | "null" | "undefined")
        || ((expr.starts_with('"') && expr.ends_with('"'))
            || (expr.starts_with('\'') && expr.ends_with('\'')))
}

fn is_simple_path_expression(expr: &str) -> bool {
    let mut has_segment = false;
    for segment in expr.split('.') {
        if segment.is_empty() || !is_simple_identifier(segment) {
            return false;
        }
        has_segment = true;
    }
    has_segment
}

fn is_simple_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$')
}

struct Rewrite {
    start: usize,
    end: usize,
    replacement: String,
}

struct ExpressionRewriteCollector<'a, 'ctx> {
    ctx: &'a GenerateContext<'ctx>,
    rewrites: std::vec::Vec<Rewrite>,
    local_scopes: std::vec::Vec<FxHashSet<String>>,
}

impl<'a, 'ctx> ExpressionRewriteCollector<'a, 'ctx> {
    fn new(ctx: &'a GenerateContext<'ctx>) -> Self {
        Self {
            ctx,
            rewrites: std::vec::Vec::new(),
            local_scopes: std::vec::Vec::new(),
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

    fn add_binding_pattern(&mut self, pattern: &oxc_ast_types::BindingPattern<'_>) {
        match pattern {
            oxc_ast_types::BindingPattern::BindingIdentifier(ident) => {
                if let Some(scope) = self.local_scopes.last_mut() {
                    scope.insert(String::new(ident.name.as_str()));
                }
            }
            oxc_ast_types::BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.add_binding_pattern(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.add_binding_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.add_binding_pattern(element);
                }
                if let Some(rest) = &array.rest {
                    self.add_binding_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::AssignmentPattern(assign) => {
                self.add_binding_pattern(&assign.left);
            }
        }
    }

    fn replacement_for_identifier(&self, name: &str) -> Option<String> {
        if self.is_local(name)
            || is_global_allowed(name)
            || matches!(name, "_ctx" | "$props" | "$slots" | "$attrs" | "$emit")
        {
            return None;
        }

        if let Some(replacement) = self.ctx.resolve_scope_binding(name) {
            return Some(replacement);
        }

        let mut resolved = String::with_capacity(name.len() + 5);
        resolved.push_str("_ctx.");
        resolved.push_str(name);
        Some(resolved)
    }

    fn push_identifier_rewrite(&mut self, ident: &oxc_ast_types::IdentifierReference<'_>) {
        if let Some(replacement) = self.replacement_for_identifier(ident.name.as_str()) {
            self.rewrites.push(Rewrite {
                start: ident.span.start as usize,
                end: ident.span.end as usize,
                replacement,
            });
        }
    }
}

impl<'a, 'ctx> Visit<'_> for ExpressionRewriteCollector<'a, 'ctx> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast_types::IdentifierReference<'_>) {
        self.push_identifier_rewrite(ident);
    }

    fn visit_member_expression(&mut self, expr: &oxc_ast_types::MemberExpression<'_>) {
        match expr {
            oxc_ast_types::MemberExpression::ComputedMemberExpression(computed) => {
                self.visit_expression(&computed.object);
                self.visit_expression(&computed.expression);
            }
            oxc_ast_types::MemberExpression::StaticMemberExpression(static_member) => {
                self.visit_expression(&static_member.object);
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
            self.add_binding_pattern(&param.pattern);
        }
        walk_arrow_function_expression(self, arrow);
        self.pop_scope();
    }

    fn visit_function(&mut self, func: &oxc_ast_types::Function<'_>, flags: ScopeFlags) {
        self.push_scope();
        for param in &func.params.items {
            self.add_binding_pattern(&param.pattern);
        }
        walk_function(self, func, flags);
        self.pop_scope();
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast_types::VariableDeclarator<'_>) {
        walk_variable_declarator(self, declarator);
        if let Some(scope) = self.local_scopes.last_mut() {
            match &declarator.id {
                oxc_ast_types::BindingPattern::BindingIdentifier(ident) => {
                    scope.insert(String::new(ident.name.as_str()));
                }
                _ => self.add_binding_pattern(&declarator.id),
            }
        }
    }

    fn visit_object_property(&mut self, property: &oxc_ast_types::ObjectProperty<'_>) {
        if property.shorthand {
            if let oxc_ast_types::PropertyKey::StaticIdentifier(ident) = &property.key {
                if let Some(replacement) = self.replacement_for_identifier(ident.name.as_str()) {
                    let key = ident.name.as_str();
                    let mut expanded = String::with_capacity(key.len() + replacement.len() + 2);
                    expanded.push_str(key);
                    expanded.push_str(": ");
                    expanded.push_str(replacement.as_str());
                    self.rewrites.push(Rewrite {
                        start: property.span.start as usize,
                        end: property.span.end as usize,
                        replacement: expanded,
                    });
                    return;
                }
            }
        }

        walk_object_property(self, property);
    }
}
