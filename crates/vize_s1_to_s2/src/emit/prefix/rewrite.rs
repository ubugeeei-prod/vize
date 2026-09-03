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
    /// The shipped lane reports `X_INVALID_EXPRESSION` here; the emit
    /// refuses instead (the diagnostic is not recoverable, so the corpus
    /// lane never compares such a template).
    pub(super) parse_error: bool,
}

fn js_module() -> SourceType {
    SourceType::default().with_module(true)
}

pub(super) fn rewrite_expression(
    content: &str,
    retained: Option<Retained<'_, '_>>,
    scope: &PrefixScope<'_>,
    as_params: bool,
) -> RewriteResult {
    if !as_params
        && let Some(retained) = retained
        && js_module_compatible(retained.ast, retained.source)
    {
        return project_aliases(rewrite_retained(content, retained, scope), scope);
    }
    let overflows = expression_exceeds_max_depth(content);
    if overflows || !expression_has_balanced_delimiters(content) {
        return RewriteResult {
            code: String::from(content),
            parse_error: !overflows,
        };
    }
    if as_params {
        return RewriteResult {
            code: String::from(content),
            parse_error: !parses_as_params(content),
        };
    }
    project_aliases(rewrite_reparsed(content, scope), scope)
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
    let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 0);
    RewriteResult {
        code,
        parse_error: false,
    }
}

fn rewrite_reparsed(content: &str, scope: &PrefixScope<'_>) -> RewriteResult {
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
        let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 1);
        return RewriteResult {
            code,
            parse_error: false,
        };
    }

    let program_allocator = Allocator::new();
    let parsed = Parser::new(program_allocator.as_oxc(), content, js_module()).parse();
    if parsed.diagnostics.is_empty() {
        let mut collector = IdentifierCollector::new(scope, content);
        collector.visit_program(&parsed.program);
        let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 0);
        return RewriteResult {
            code,
            parse_error: false,
        };
    }

    if is_simple_identifier(content) {
        let code = match scope.identifier_prefix(content) {
            Some(prefix) => {
                let mut code = String::with_capacity(prefix.len() + content.len());
                code.push_str(prefix);
                code.push_str(content);
                code
            }
            None => String::from(content),
        };
        return RewriteResult {
            code,
            parse_error: false,
        };
    }
    RewriteResult {
        code: String::from(content),
        parse_error: true,
    }
}

/// `parse_checks::parse_as_params`: the synthesized `(content) => null` parse.
fn parses_as_params(content: &str) -> bool {
    let allocator = Allocator::new();
    let mut wrapped = String::with_capacity(content.len() + 12);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push_str(") => null");
    Parser::new(allocator.as_oxc(), wrapped.as_str(), js_module())
        .parse_expression()
        .is_ok()
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
