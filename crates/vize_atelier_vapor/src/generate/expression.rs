use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{Visit, walk::walk_object_property};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_atelier_core::steps::expression::ExpressionScope;
use vize_atelier_core::steps::expression::is_template_global;
use vize_carton::{FxHashSet, String, ToCompactString};

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

    // Failed parses are opaque. Preserve their bytes for the caller's
    // diagnostic/tolerant mode; recursively scanning the same rejected token
    // (for example `(`) can never make progress or establish JS semantics.
    trimmed.to_compact_string()
}

pub(super) fn resolve_with_oxc(ctx: &GenerateContext<'_>, expr: &str) -> Option<String> {
    let allocator = vize_atelier_core::expr_parse_probe::parse_arena();
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

    let allocator = vize_atelier_core::expr_parse_probe::parse_arena();
    let parser = Parser::new(&allocator, expr, source_type);
    let parsed = parser.parse();
    if parsed.diagnostics.is_empty() {
        let mut collector = ExpressionRewriteCollector::new(ctx);
        collector.visit_program(&parsed.program);
        return Some(apply_rewrites(expr, collector.rewrites, 0));
    }

    None
}

pub(super) fn apply_rewrites(
    expr: &str,
    mut rewrites: std::vec::Vec<Rewrite>,
    offset: usize,
) -> String {
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

pub(super) fn is_literal_expression(expr: &str) -> bool {
    expr.parse::<f64>().is_ok()
        || matches!(expr, "true" | "false" | "null" | "undefined")
        || single_string_literal(expr)
}

/// One quoted string literal. `'[' + a + ']'` starts and ends with a quote
/// but is a concatenation whose `a` still needs resolving.
fn single_string_literal(expr: &str) -> bool {
    let Some(quote) = expr.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
        return false;
    };
    let Some(body) = expr
        .strip_prefix(quote)
        .and_then(|rest| rest.strip_suffix(quote))
    else {
        return false;
    };
    let mut escaped = false;
    for c in body.chars() {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            c if c == quote => return false,
            _ => {}
        }
    }
    !escaped
}

pub(super) fn is_simple_path_expression(expr: &str) -> bool {
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

pub(super) struct Rewrite {
    start: usize,
    end: usize,
    replacement: String,
}

pub(super) struct ExpressionRewriteCollector<'a, 'ctx> {
    ctx: &'a GenerateContext<'ctx>,
    pub(super) rewrites: std::vec::Vec<Rewrite>,
    local_scopes: std::vec::Vec<FxHashSet<String>>,
}

impl<'a, 'ctx> ExpressionRewriteCollector<'a, 'ctx> {
    pub(super) fn new(ctx: &'a GenerateContext<'ctx>) -> Self {
        Self {
            ctx,
            rewrites: std::vec::Vec::new(),
            local_scopes: std::vec::Vec::new(),
        }
    }

    pub(super) fn add_event_parameter(&mut self) {
        let mut scope = FxHashSet::default();
        scope.insert(String::from("$event"));
        self.local_scopes.push(scope);
    }

    fn is_local(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn replacement_for_identifier(&self, name: &str) -> Option<String> {
        if self.is_local(name)
            || is_template_global(name)
            || matches!(name, "_ctx" | "$props" | "$slots" | "$attrs" | "$emit")
        {
            return None;
        }

        if let Some(replacement) = self.ctx.resolve_scope_binding(name) {
            return Some(replacement);
        }

        // In JSX closure mode there is no `_ctx`: a free identifier is captured
        // from the enclosing component function, so it stays bare (matching
        // `vue-jsx-vapor`).
        if self.ctx.jsx_closure {
            return None;
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

    fn visit_program(&mut self, node: &oxc_ast_types::Program<'_>) {
        self.scoped_program(node);
    }

    fn visit_arrow_function_expression(
        &mut self,
        node: &oxc_ast_types::ArrowFunctionExpression<'_>,
    ) {
        self.scoped_arrow(node);
    }

    fn visit_function_body(&mut self, node: &oxc_ast_types::FunctionBody<'_>) {
        self.scoped_function_body(node);
    }

    fn visit_block_statement(&mut self, node: &oxc_ast_types::BlockStatement<'_>) {
        self.scoped_block(node);
    }

    fn visit_catch_clause(&mut self, node: &oxc_ast_types::CatchClause<'_>) {
        self.scoped_catch(node);
    }

    fn visit_for_statement(&mut self, node: &oxc_ast_types::ForStatement<'_>) {
        self.scoped_for(node);
    }

    fn visit_for_in_statement(&mut self, node: &oxc_ast_types::ForInStatement<'_>) {
        self.scoped_for_in(node);
    }

    fn visit_for_of_statement(&mut self, node: &oxc_ast_types::ForOfStatement<'_>) {
        self.scoped_for_of(node);
    }

    fn visit_switch_statement(&mut self, node: &oxc_ast_types::SwitchStatement<'_>) {
        self.scoped_switch(node);
    }

    fn visit_class(&mut self, node: &oxc_ast_types::Class<'_>) {
        self.scoped_class(node);
    }

    fn visit_static_block(&mut self, node: &oxc_ast_types::StaticBlock<'_>) {
        self.scoped_static_block(node);
    }

    fn visit_function(&mut self, function: &oxc_ast_types::Function<'_>, flags: ScopeFlags) {
        self.scoped_function(function, flags);
    }

    fn visit_object_property(&mut self, property: &oxc_ast_types::ObjectProperty<'_>) {
        if property.shorthand
            && let oxc_ast_types::PropertyKey::StaticIdentifier(ident) = &property.key
            && let Some(replacement) = self.replacement_for_identifier(ident.name.as_str())
        {
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

        walk_object_property(self, property);
    }
}

impl<'ast> ExpressionScope<'ast> for ExpressionRewriteCollector<'_, '_> {
    fn push_scope(&mut self) {
        self.local_scopes.push(FxHashSet::default());
    }
    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }
    fn add_local(&mut self, name: &str) {
        // An expression binding always pushes its scope first.
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(String::new(name));
        }
    }
}
