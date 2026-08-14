mod walk;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, profile};

use super::IdentifierRef;

/// OXC-based identifier extraction for expressions with object literals.
#[inline]
pub(super) fn extract_identifiers_oxc_slow(expr: &str) -> Vec<CompactString> {
    extract_identifier_refs_oxc_slow(expr)
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
/// [`extract_identifiers_oxc_slow`] until P1-8 deletes the split.
#[inline]
pub(super) fn extract_identifiers_retained_slow(
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
pub(super) fn extract_identifier_refs_oxc_slow(expr: &str) -> Vec<IdentifierRef> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("expr.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.helpers.identifiers.oxc_parse",
        Parser::new(&allocator, expr, source_type).parse_expression()
    );
    let parsed_expr = match ret {
        Ok(expr) => expr,
        Err(_) => return Vec::new(),
    };

    let mut identifiers = Vec::with_capacity(4);
    profile!(
        "croquis.helpers.identifiers.walk_expr",
        walk::walk_expr(&parsed_expr, &mut identifiers)
    );
    identifiers
}
