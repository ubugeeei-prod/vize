//! `rewrite_expression` (`steps::expression::rewrite` + `reparse` +
//! `retained_rewrite`), ported for the no-binding, non-TS lane: the
//! retained AST drives a span splice when the dialect gate admits it;
//! everything else re-parses through the legacy chain — wrapped
//! expression parse, whole-program parse, simple-identifier fallback —
//! whose byte behavior is the shipped lane's.

use oxc_ast::ast::Expression;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::expression_guard::{expression_exceeds_max_depth, expression_has_balanced_delimiters};
use vize_s0::{Allocator, String};

use super::collector::IdentifierCollector;
use super::compat::js_module_compatible;
use super::globals::is_simple_identifier;
use super::scope::PrefixScope;
use super::splice::splice_insertions;
use super::strip_typescript_from_expression;

/// A retained AST beside the text it describes: `ast` was parsed from
/// `text[offset..offset + len]`, so its spans shift by `offset`.
#[derive(Clone, Copy)]
pub(super) struct Retained<'r, 'a> {
    pub(super) ast: &'r Expression<'a>,
    pub(super) source: &'a str,
    pub(super) offset: usize,
}

pub(super) struct RewriteResult {
    pub(super) code: String,
    /// Set when a binding was read through `_unref(…)`; the emit marks
    /// that helper once, after the body, the way the shipped lane appends
    /// it after every used helper.
    pub(super) used_unref: bool,
    /// The shipped lane reports `X_INVALID_EXPRESSION` here; the emit
    /// refuses instead (the diagnostic is not recoverable, so the corpus
    /// lane never compares such a template).
    pub(super) parse_error: bool,
}

fn js_module() -> SourceType {
    SourceType::default().with_module(true)
}

fn ts_module() -> SourceType {
    SourceType::ts().with_module(true)
}

pub(super) fn rewrite_expression(
    content: &str,
    retained: Option<Retained<'_, '_>>,
    scope: &PrefixScope<'_>,
    as_params: bool,
) -> RewriteResult {
    if !as_params
        && let Some(js) = retained
        && js_module_compatible(js.ast, js.source)
    {
        if !scope.is_ts() {
            return project_aliases(rewrite_retained(content, js, scope), scope);
        }
        // TS lanes strip first, always: the detection scan can false-positive
        // on TS-free text (` as ` inside a string literal) and rewrite bytes
        // through the codegen round-trip. Only the identity outcome keeps the
        // retained byte proof; changed bytes rejoin the legacy chain.
        let js_content = strip_typescript_from_expression(content);
        if js_content.as_str() == content {
            return project_aliases(rewrite_retained(content, js, scope), scope);
        }
        return project_aliases(
            rewrite_reparsed(js_content, content, retained, scope),
            scope,
        );
    }
    let overflows = expression_exceeds_max_depth(content);
    if overflows || !expression_has_balanced_delimiters(content) {
        return RewriteResult {
            code: String::from(content),
            used_unref: false,
            parse_error: !overflows,
        };
    }
    let js_content = if scope.is_ts() {
        strip_typescript_from_expression(content)
    } else {
        String::from(content)
    };
    if as_params {
        // The original text is re-checked as TypeScript: the official
        // compiler accepts params the stripping fallback could not lower.
        let accepted = parses_as_params(js_content.as_str(), js_module())
            || (scope.is_ts() && parses_as_params(content, ts_module()));
        return RewriteResult {
            code: js_content,
            used_unref: false,
            parse_error: !accepted,
        };
    }
    project_aliases(
        rewrite_reparsed(js_content, content, retained, scope),
        scope,
    )
}

/// The transform's `rewrite_props_aliases` post-pass over both prop
/// objects; a parse failure passes the raw text through untouched.
fn project_aliases(result: RewriteResult, scope: &PrefixScope<'_>) -> RewriteResult {
    if result.parse_error {
        return result;
    }
    RewriteResult {
        code: super::aliases::rewrite_props_aliases(
            result.code,
            scope.bindings(),
            &["__props", "$props"],
        ),
        used_unref: result.used_unref,
        parse_error: false,
    }
}

fn rewrite_retained(
    content: &str,
    retained: Retained<'_, '_>,
    scope: &PrefixScope<'_>,
) -> RewriteResult {
    let mut collector = IdentifierCollector::new_unwrapped(scope, content, retained.offset);
    collector.visit_expression(retained.ast);
    let used_unref = collector.used_unref;
    let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 0);
    RewriteResult {
        code,
        used_unref,
        parse_error: false,
    }
}

/// The legacy re-parse chain over already-stripped text. `original` is
/// the pre-strip text, read only by the TS-acceptance check; a retained
/// AST that passed the dialect gate already proves the original parses as
/// TypeScript, so it short-circuits that check.
fn rewrite_reparsed(
    js_content: String,
    original: &str,
    retained: Option<Retained<'_, '_>>,
    scope: &PrefixScope<'_>,
) -> RewriteResult {
    let content = js_content.as_str();
    let allocator = Allocator::new();
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    if let Ok(expr) =
        Parser::new(allocator.as_oxc(), wrapped.as_str(), js_module()).parse_expression()
    {
        let mut collector = IdentifierCollector::new(scope, wrapped.as_str());
        collector.visit_expression(&expr);
        let used_unref = collector.used_unref;
        let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 1);
        return RewriteResult {
            code,
            used_unref,
            parse_error: false,
        };
    }

    let program_allocator = Allocator::new();
    let parsed = Parser::new(program_allocator.as_oxc(), content, js_module()).parse();
    if parsed.diagnostics.is_empty() {
        let mut collector = IdentifierCollector::new(scope, content);
        collector.visit_program(&parsed.program);
        let used_unref = collector.used_unref;
        let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 0);
        return RewriteResult {
            code,
            used_unref,
            parse_error: false,
        };
    }

    if is_simple_identifier(content) {
        // The same three-way read the collector makes: prefix, `.value`
        // for an inline ref, `_unref(…)` for an inline `let`.
        let needs_unref = scope.needs_unref(content);
        let code = match scope.identifier_prefix(content) {
            Some(prefix) => {
                let mut code = String::with_capacity(prefix.len() + content.len());
                code.push_str(prefix);
                code.push_str(content);
                code
            }
            None if scope.is_ref_binding(content) => {
                let mut code = String::with_capacity(content.len() + 6);
                code.push_str(content);
                code.push_str(".value");
                code
            }
            None if needs_unref => {
                let mut code = String::with_capacity(content.len() + 8);
                code.push_str("_unref(");
                code.push_str(content);
                code.push(')');
                code
            }
            None => String::from(content),
        };
        return RewriteResult {
            code,
            used_unref: needs_unref,
            parse_error: false,
        };
    }
    let ts_accepts = scope.is_ts()
        && (retained.is_some_and(|js| js_module_compatible(js.ast, js.source))
            || parses_as_typescript(original));
    RewriteResult {
        code: js_content,
        used_unref: false,
        parse_error: !ts_accepts,
    }
}

/// `parse_checks::parse_as_params`: the synthesized `(content) => null` parse.
fn parses_as_params(content: &str, source_type: SourceType) -> bool {
    let allocator = Allocator::new();
    let mut wrapped = String::with_capacity(content.len() + 12);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push_str(") => null");
    Parser::new(allocator.as_oxc(), wrapped.as_str(), source_type)
        .parse_expression()
        .is_ok()
}

/// `parse_checks::parses_as_typescript`: the wrapped expression parse,
/// then the whole-program parse, both as TypeScript.
fn parses_as_typescript(content: &str) -> bool {
    let expr_allocator = Allocator::new();
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    if Parser::new(expr_allocator.as_oxc(), wrapped.as_str(), ts_module())
        .parse_expression()
        .is_ok()
    {
        return true;
    }
    let program_allocator = Allocator::new();
    Parser::new(program_allocator.as_oxc(), content, ts_module())
        .parse()
        .diagnostics
        .is_empty()
}

/// The legacy prefix parse: `Parser::parse_expression` over the bare
/// text, which accepts a leading complete expression and ignores the rest.
pub(super) fn with_prefix_parse<T>(
    content: &str,
    decide: impl FnOnce(&Expression<'_>) -> T,
) -> Option<T> {
    if !vize_s0::expression_guard::expression_is_safe_to_parse(content) {
        return None;
    }
    let allocator = Allocator::new();
    Parser::new(allocator.as_oxc(), content, js_module())
        .parse_expression()
        .ok()
        .map(|expr| decide(&expr))
}
