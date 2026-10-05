mod walk;

use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, profile};

use super::IdentifierRef;
use walk::facts::IdentifierWalk;

/// OXC-based identifier extraction for expressions with object literals.
#[inline]
pub(super) fn extract_identifiers_oxc_ast(expr: &str) -> Vec<CompactString> {
    extract_identifier_refs_oxc_ast(expr)
        .into_iter()
        .map(|identifier| identifier.name)
        .collect()
}

/// Identifier walk over a retained (parse-once, Davinci P1-5) expression AST.
///
/// The walk is the same [`walk`] the re-parse path runs; the oxc parse and
/// its throwaway `Allocator::default()` die here for nodes that carry
/// [`vize_relief::JsExpression`] (Davinci P1-6). The fallback classes —
/// shapes without a retained AST (v-for sub-expressions, v-on statement
/// bodies, guard-refused or invalid text, compound expressions) and
/// comment-carrying text (see `extract_identifiers_retained`) — stay on
/// [`extract_identifiers_oxc_ast`].
#[inline]
pub(super) fn extract_identifiers_retained_ast(
    ast: &oxc_ast::ast::Expression<'_>,
) -> Vec<CompactString> {
    extract_identifier_refs_retained_ast(ast)
        .into_iter()
        .map(|identifier| identifier.name)
        .collect()
}

pub(super) fn extract_identifier_refs_retained_ast(
    ast: &oxc_ast::ast::Expression<'_>,
) -> Vec<IdentifierRef> {
    retained_references(ast, false).0
}

pub(super) fn retained_references(
    ast: &Expression<'_>,
    demanded: bool,
) -> (Vec<IdentifierRef>, bool) {
    let mut identifiers = IdentifierWalk::new(demanded);
    profile!(
        "croquis.helpers.identifiers.walk_expr",
        walk::walk_expr(ast, &mut identifiers)
    );
    (identifiers.values, identifiers.complete)
}

#[inline]
pub(super) fn extract_identifier_refs_oxc_ast(expr: &str) -> Vec<IdentifierRef> {
    parsed_references(expr, false).0
}

pub(super) fn parsed_references(expr: &str, demanded: bool) -> (Vec<IdentifierRef>, bool) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("expr.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.helpers.identifiers.oxc_parse",
        Parser::new(&allocator, expr, source_type).parse_expression()
    );
    let parsed_expr = match ret {
        Ok(expr) => expr,
        Err(_) => {
            return extract_identifier_refs_oxc_program(expr, source_type, demanded)
                .unwrap_or_default();
        }
    };

    if !expression_consumes_source(expr, &parsed_expr)
        && let Some(identifiers) = extract_identifier_refs_oxc_program(expr, source_type, demanded)
    {
        return identifiers;
    }

    let mut references = retained_references(&parsed_expr, demanded);
    references.1 &= !demanded || expression_consumes_source(expr, &parsed_expr);
    references
}

fn expression_consumes_source(expr: &str, parsed_expr: &Expression<'_>) -> bool {
    let end = parsed_expr.span().end as usize;
    expr.get(end..)
        .is_none_or(|tail| tail.chars().all(char::is_whitespace))
}

fn extract_identifier_refs_oxc_program(
    expr: &str,
    source_type: SourceType,
    demanded: bool,
) -> Option<(Vec<IdentifierRef>, bool)> {
    let allocator = Allocator::default();
    let ret = profile!(
        "croquis.helpers.identifiers.oxc_parse_program",
        if demanded {
            Parser::new(&allocator, expr, source_type).parse()
        } else {
            crate::script_parser::parse_program_for_analysis(&allocator, expr, source_type)
        }
    );
    if ret.panicked {
        return None;
    }
    let syntax_complete = ret.diagnostics.is_empty();

    // A missing member name still reads its receiver. Reuse the same bounded,
    // byte-preserving analysis view as scripts so incomplete template edits
    // cannot mark `$event` unused and erase its native type context.

    // A statement body owns lexical bindings (loops, catches, functions,
    // classes, enums). Only unresolved value references reach template scope.
    let built = SemanticBuilder::new()
        .with_check_syntax_error(demanded)
        .with_build_nodes(true)
        .build(&ret.program);
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    let mut identifiers: Vec<_> = scoping
        .root_unresolved_references_ids()
        .flatten()
        .filter_map(|id| {
            let reference = scoping.get_reference(id);
            (reference.flags().is_value() || reference.flags().is_value_as_type()).then(|| {
                IdentifierRef::new(
                    semantic.reference_name(reference),
                    semantic.reference_span(reference).start,
                )
            })
        })
        .collect();
    identifiers.sort_unstable_by_key(|reference| reference.offset);
    let complete = !demanded
        || (syntax_complete
            && built.diagnostics.is_empty()
            && !scoping
                .scope_descendants_from_root()
                .any(|scope| scoping.scope_flags(scope).contains_direct_eval()));
    Some((identifiers, complete))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_members_retain_only_lexically_unresolved_receivers() {
        for source in [
            "$event.",
            "$event?.",
            "const 雪 = '🌸'; $event.",
            "(function ($event) { $event.; })(); $event?.",
        ] {
            assert_eq!(
                extract_identifier_refs_oxc_ast(source),
                vec![IdentifierRef {
                    name: "$event".into(),
                    offset: source.rfind("$event").unwrap() as u32,
                }],
                "{source}"
            );
        }
        for source in [
            "'$event.'",
            "1.",
            "(() => { const $event = {}; $event.; })()",
        ] {
            assert!(
                extract_identifier_refs_oxc_ast(source).is_empty(),
                "{source}"
            );
        }
    }

    /// A block-bodied handler (`@click="() => { ... }"`) reports the
    /// references that escape it, in source order, and keeps its own
    /// parameters and lexical declarations local.
    #[test]
    fn block_bodied_functions_report_references_that_escape_them() {
        let source = "() => { const local = $router; $router.replace(local); return other }";
        let refs = extract_identifier_refs_oxc_ast(source);
        let names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["$router", "$router", "other"], "{source}");
        assert_eq!(refs[0].offset, source.find("$router").unwrap() as u32);
        assert_eq!(refs[1].offset, source.rfind("$router").unwrap() as u32);

        let source = "function (event) { if (event) { handle(event, $el) } }";
        let names: Vec<_> = extract_identifier_refs_oxc_ast(source)
            .into_iter()
            .map(|r| r.name)
            .collect();
        assert_eq!(names, vec!["handle", "$el"], "{source}");
    }

    #[test]
    fn block_bodied_functions_scope_their_declarations_lexically() {
        for (source, expected) in [
            // Rest parameters bind like the others.
            ("(...args) => { consume(args) }", vec!["consume"]),
            (
                "function (a, ...rest) { return a + rest.length + b }",
                vec!["b"],
            ),
            // `const` / `let` belong to their block; `var` hoists.
            ("() => { { const local = 1 } return local }", vec!["local"]),
            (
                "() => { if (flag) { const inner = make(); use(inner) } else { use(inner) } }",
                vec!["flag", "make", "use", "use", "inner"],
            ),
            (
                "() => { var a = 1; if (x) { var b = 2 } return a + b }",
                vec!["x"],
            ),
            (
                "() => { function helper() {} class Local {} return [helper, Local] }",
                vec![],
            ),
            // Default values and computed keys read the surrounding scope.
            (
                "() => { const { value = fallback, [key]: picked } = options; return value + picked }",
                vec!["fallback", "key", "options"],
            ),
            (
                "(a, b = a, { c = d } = e) => b + c + f",
                vec!["d", "e", "f"],
            ),
            ("(value = fallback) => value", vec!["fallback"]),
            (
                "function ({ value } = options) { return value }",
                vec!["options"],
            ),
            ("function named() { return named }", vec![]),
        ] {
            let names: Vec<_> = extract_identifier_refs_oxc_ast(source)
                .into_iter()
                .map(|r| r.name)
                .collect();
            assert_eq!(names, expected, "{source}");
        }
    }
}
