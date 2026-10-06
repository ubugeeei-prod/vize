//! Script execution uses AST references, not occurrences of browser names.
//! Function bodies are deferred unless called directly or registered with a
//! witnessed Vue server/initial synchronous-effect API. This is a bounded SSR
//! heuristic, not interprocedural analysis of external calls or Options API.

use super::{BROWSER_GLOBALS, META, NoBrowserGlobalsInSsr, script_symbols, typeof_guard};
use crate::{context::LintContext, diagnostic::LintDiagnostic};
use oxc_allocator::Allocator;
use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{SourceType, Span};
use oxc_syntax::{operator::UnaryOperator, scope::ScopeFlags, symbol::SymbolId};
use vize_l0::String;

pub(super) fn check(ctx: &mut LintContext<'_>) {
    if !ctx.is_ssr_enabled() || !ctx.is_rule_enabled(META.name) {
        return;
    }
    let reads = {
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        let blocks: Vec<_> = descriptor
            .script
            .iter()
            .chain(descriptor.script_setup.iter())
            .collect();
        if blocks
            .iter()
            .all(|block| !might_read_browser_global(&block.content))
        {
            return;
        }
        // Keep both authored scripts in one lexical Program: normal-script
        // imports/declarations are visible to setup. Padding preserves exact
        // physical SFC byte offsets without exposing template/style as JS.
        let mut bytes = vec![b' '; descriptor.source.len()];
        let mut extension = "js";
        for block in blocks {
            if block.src.is_some() {
                return;
            }
            let lang = block.lang.as_deref().unwrap_or("js");
            if !matches!(lang, "js" | "ts" | "jsx" | "tsx") {
                return;
            }
            extension = match (extension, lang) {
                ("tsx", _) | (_, "tsx") | ("ts", "jsx") | ("jsx", "ts") => "tsx",
                (_, "ts") | ("ts", _) => "ts",
                (_, "jsx") | ("jsx", _) => "jsx",
                _ => "js",
            };
            let start = block.loc.start;
            let Some(end) = start.checked_add(block.content.len()) else {
                return;
            };
            if end != block.loc.end
                || descriptor.source.get(start..end) != Some(block.content.as_ref())
            {
                return;
            }
            let Some(target) = bytes.get_mut(start..end) else {
                return;
            };
            target.copy_from_slice(block.content.as_bytes());
        }
        let Ok(source) = String::from_utf8(bytes) else {
            return;
        };
        let path = match extension {
            "ts" => "component.ts",
            "tsx" => "component.tsx",
            "jsx" => "component.jsx",
            _ => "component.js",
        };
        find_reads(&source, SourceType::from_path(path).unwrap())
    };
    for (name, span) in reads {
        let mut diagnostic = LintDiagnostic::warn(
            META.name,
            ctx.t_fmt("ssr/no-browser-globals-in-ssr.message", &[("name", &name)]),
            span.start,
            span.end,
        );
        if let Some(help) = ctx
            .help_level()
            .process(&ctx.t("ssr/no-browser-globals-in-ssr.help"))
        {
            diagnostic = diagnostic.with_help(help);
        }
        ctx.report_in_sfc(diagnostic);
    }
}

fn might_read_browser_global(source: &str) -> bool {
    source.contains('\\') || BROWSER_GLOBALS.iter().any(|name| source.contains(name))
}

pub(super) fn find_reads(source: &str, source_type: SourceType) -> Vec<(String, Span)> {
    if !might_read_browser_global(source) {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let result = SemanticBuilder::new().build(&parsed.program);
    if !result.diagnostics.is_empty() {
        return Vec::new();
    }
    let semantic = result.semantic;
    let mut callbacks = Vec::new();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.source.value != "vue" || import.import_kind.is_type() {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                continue;
            };
            let name = specifier.imported.name();
            if !specifier.import_kind.is_type()
                && matches!(
                    name.as_str(),
                    "onServerPrefetch" | "watchEffect" | "watchSyncEffect"
                )
                && let Some(symbol) = specifier.local.symbol_id.get()
            {
                callbacks.push((symbol, name.as_str() == "watchEffect"));
            }
        }
    }
    let mut visitor = RuntimeReads {
        semantic: &semantic,
        callbacks,
        active: Vec::new(),
        guarded: Vec::new(),
        reads: Vec::new(),
    };
    visitor.visit_program(&parsed.program);
    visitor
        .reads
        .sort_unstable_by_key(|(_, span)| (span.start, span.end));
    visitor
        .reads
        .dedup_by_key(|(_, span)| (span.start, span.end));
    visitor.reads
}

struct RuntimeReads<'s, 'a> {
    semantic: &'s Semantic<'a>,
    // watchEffect is synchronous initially only with its default options.
    callbacks: Vec<(SymbolId, bool)>,
    active: Vec<Span>,
    guarded: Vec<String>,
    reads: Vec<(String, Span)>,
}

impl<'a> RuntimeReads<'_, 'a> {
    fn symbol(&self, id: &IdentifierReference<'_>) -> Option<SymbolId> {
        id.reference_id
            .get()
            .and_then(|id| self.semantic.scoping().get_reference(id).symbol_id())
    }

    fn execute(&mut self, expression: &Expression<'a>) {
        match expression.get_inner_expression() {
            Expression::ArrowFunctionExpression(arrow) => self.body(&arrow.body, arrow.span),
            Expression::FunctionExpression(function) => {
                if !function.generator
                    && let Some(body) = &function.body
                {
                    self.body(body, function.span);
                }
            }
            Expression::Identifier(id) => {
                let Some(symbol) = self.symbol(id) else {
                    return;
                };
                if self
                    .semantic
                    .symbol_references(symbol)
                    .any(|reference| reference.is_write())
                {
                    return;
                }
                match self.semantic.symbol_declaration(symbol).kind() {
                    AstKind::Function(function) => {
                        if !function.generator
                            && let Some(body) = &function.body
                        {
                            self.body(body, function.span);
                        }
                    }
                    AstKind::VariableDeclarator(variable) => {
                        if let Some(init) = &variable.init {
                            // Follow only function initializers, never repeat
                            // effects of evaluating an ordinary initializer.
                            if matches!(
                                init.get_inner_expression(),
                                Expression::ArrowFunctionExpression(_)
                                    | Expression::FunctionExpression(_)
                            ) {
                                self.execute(init);
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn body(&mut self, body: &FunctionBody<'a>, span: Span) {
        if self.active.contains(&span) {
            return;
        }
        self.active.push(span);
        self.visit_function_body(body);
        self.active.pop();
    }
}

impl<'a> Visit<'a> for RuntimeReads<'_, 'a> {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        if NoBrowserGlobalsInSsr::is_browser_global_static(id.name.as_str())
            && !self
                .guarded
                .iter()
                .any(|name| name.as_str() == id.name.as_str())
            && let Some(reference) = id.reference_id.get()
            && self.semantic.scoping().get_reference(reference).is_value()
            && !self
                .symbol(id)
                .is_some_and(|symbol| script_symbols::runtime_shadow(self.semantic, symbol))
        {
            self.reads.push((String::from(id.name.as_str()), id.span));
        }
    }

    fn visit_ts_type(&mut self, _: &TSType<'a>) {}
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        self.visit_decorators(&property.decorators);
        self.visit_property_key(&property.key);
        if property.r#static
            && let Some(value) = &property.value
        {
            self.visit_expression(value);
        }
    }

    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Typeof
            && matches!(
                expression.argument.get_inner_expression(),
                Expression::Identifier(_)
            )
        {
            return;
        }
        self.visit_expression(&expression.argument);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // Callee/ordinary arguments are evaluated now; function arguments
        // remain deferred. A direct local/IIFE call executes its own body.
        walk_call_expression(self, call);
        self.execute(&call.callee);
        if let Expression::Identifier(id) = call.callee.get_inner_expression()
            && let Some(symbol) = self.symbol(id)
            && let Some((_, default_only)) = self.callbacks.iter().find(|(id, _)| *id == symbol)
            && (!*default_only || call.arguments.len() == 1)
            && let Some(argument) = call.arguments.first().and_then(Argument::as_expression)
        {
            self.execute(argument);
        }
    }

    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
        self.visit_expression(&statement.test);
        let depth = self.guarded.len();
        self.guarded
            .extend(typeof_guard::defined_name(&statement.test, true));
        self.visit_statement(&statement.consequent);
        self.guarded.truncate(depth);
        if let Some(alternate) = &statement.alternate {
            self.guarded
                .extend(typeof_guard::defined_name(&statement.test, false));
            self.visit_statement(alternate);
            self.guarded.truncate(depth);
        }
    }

    fn visit_conditional_expression(&mut self, expression: &ConditionalExpression<'a>) {
        self.visit_expression(&expression.test);
        let depth = self.guarded.len();
        self.guarded
            .extend(typeof_guard::defined_name(&expression.test, true));
        self.visit_expression(&expression.consequent);
        self.guarded.truncate(depth);
        self.guarded
            .extend(typeof_guard::defined_name(&expression.test, false));
        self.visit_expression(&expression.alternate);
        self.guarded.truncate(depth);
    }

    fn visit_logical_expression(&mut self, expression: &LogicalExpression<'a>) {
        self.visit_expression(&expression.left);
        let depth = self.guarded.len();
        match expression.operator {
            oxc_syntax::operator::LogicalOperator::And => self
                .guarded
                .extend(typeof_guard::defined_name(&expression.left, true)),
            oxc_syntax::operator::LogicalOperator::Or => self
                .guarded
                .extend(typeof_guard::defined_name(&expression.left, false)),
            _ => {}
        }
        self.visit_expression(&expression.right);
        self.guarded.truncate(depth);
    }
}
