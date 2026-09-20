mod walk;

use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, profile};

use super::IdentifierRef;

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
    let mut identifiers = Vec::with_capacity(4);
    profile!(
        "croquis.helpers.identifiers.walk_expr",
        walk::walk_expr(ast, &mut identifiers)
    );
    identifiers
        .into_iter()
        .map(|identifier| identifier.name)
        .collect()
}

#[inline]
pub(super) fn extract_identifier_refs_oxc_ast(expr: &str) -> Vec<IdentifierRef> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("expr.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.helpers.identifiers.oxc_parse",
        Parser::new(&allocator, expr, source_type).parse_expression()
    );
    let parsed_expr = match ret {
        Ok(expr) => expr,
        Err(_) => {
            return extract_identifier_refs_oxc_program(expr, source_type).unwrap_or_default();
        }
    };

    if !expression_consumes_source(expr, &parsed_expr)
        && let Some(identifiers) = extract_identifier_refs_oxc_program(expr, source_type)
    {
        return identifiers;
    }

    let mut identifiers = Vec::with_capacity(4);
    profile!(
        "croquis.helpers.identifiers.walk_expr",
        walk::walk_expr(&parsed_expr, &mut identifiers)
    );
    identifiers
}

fn expression_consumes_source(expr: &str, parsed_expr: &Expression<'_>) -> bool {
    let end = parsed_expr.span().end as usize;
    expr.get(end..)
        .is_none_or(|tail| tail.chars().all(char::is_whitespace))
}

fn extract_identifier_refs_oxc_program(
    expr: &str,
    source_type: SourceType,
) -> Option<Vec<IdentifierRef>> {
    let allocator = Allocator::default();
    let ret = profile!(
        "croquis.helpers.identifiers.oxc_parse_program",
        crate::script_parser::parse_program_for_analysis(&allocator, expr, source_type)
    );
    if ret.panicked {
        return None;
    }

    // A missing member name still reads its receiver. Reuse the same bounded,
    // byte-preserving analysis view as scripts so incomplete template edits
    // cannot mark `$event` unused and erase its native type context.

    // A statement body owns lexical bindings (loops, catches, functions,
    // classes, enums). Only unresolved value references reach template scope.
    let built = SemanticBuilder::new()
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
    Some(identifiers)
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
}
