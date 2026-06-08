mod walk;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, profile};

/// OXC-based identifier extraction for expressions with object literals.
#[inline]
pub(super) fn extract_identifiers_oxc_slow(expr: &str) -> Vec<CompactString> {
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
