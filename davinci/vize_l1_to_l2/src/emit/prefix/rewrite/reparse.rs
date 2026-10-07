use oxc_ast_visit::Visit;

use super::{
    Allocator, IdentifierCollector, Parser, PrefixScope, Retained, RewriteResult, String,
    is_generated_filter_helper, is_simple_identifier, js_module, js_module_compatible,
    parses_as_typescript, splice_insertions,
};

/// The legacy re-parse chain over already-stripped text. `original` is
/// the pre-strip text, read only by the TS-acceptance check; a retained
/// AST that passed the dialect gate already proves the original parses as
/// TypeScript, so it short-circuits that check.
pub(super) fn rewrite_reparsed(
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
        let used_is_ref = collector.used_is_ref;
        let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 1);
        return RewriteResult {
            code,
            used_unref,
            used_is_ref,
            parse_error: false,
        };
    }

    let program_allocator = Allocator::new();
    let parsed = Parser::new(program_allocator.as_oxc(), content, js_module()).parse();
    if parsed.diagnostics.is_empty() {
        let mut collector = IdentifierCollector::new(scope, content);
        collector.visit_program(&parsed.program);
        let used_unref = collector.used_unref;
        let used_is_ref = collector.used_is_ref;
        let code = splice_insertions(content, collector.rewrites, collector.suffix_rewrites, 0);
        return RewriteResult {
            code,
            used_unref,
            used_is_ref,
            parse_error: false,
        };
    }

    if is_simple_identifier(content) {
        if is_generated_filter_helper(content) {
            return RewriteResult {
                code: String::from(content),
                used_unref: false,
                used_is_ref: false,
                parse_error: false,
            };
        }
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
            used_is_ref: false,
            parse_error: false,
        };
    }
    let ts_accepts = scope.is_ts()
        && (retained.is_some_and(|js| js_module_compatible(js.ast, js.source))
            || parses_as_typescript(original));
    RewriteResult {
        code: js_content,
        used_unref: false,
        used_is_ref: false,
        parse_error: !ts_accepts,
    }
}
