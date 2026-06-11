//! petite-vue `v-scope` object-literal key extraction.
//!
//! `v-scope="{ count: 0, msg: 'x' }"` introduces the object's **top-level**
//! keys as in-scope names for the element's subtree. This module parses the
//! object expression and returns each key together with the byte offset of its
//! key token (relative to the expression source), so the binding can carry an
//! accurate declaration offset for go-to-definition.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, PropertyKey};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, SmallVec};

/// A v-scope binding: the introduced name and the offset of its key token
/// relative to the start of the v-scope expression source.
pub type VScopeBinding = (CompactString, u32);

/// Extract the top-level keys of a `v-scope` object expression.
///
/// Returns an empty list when the expression is not an object literal (e.g.
/// `v-scope` with no value, or a non-object expression), so non-object usages
/// simply introduce no bindings rather than erroring.
pub fn extract_v_scope_bindings(expr: &str) -> SmallVec<[VScopeBinding; 4]> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return SmallVec::new();
    }

    // Wrap as the initializer of a declaration so the object literal parses as
    // an expression (a bare leading `{` would be a block statement). The prefix
    // length is fixed, so subtracting it recovers offsets within `trimmed`.
    const PREFIX: &str = "const __vize_scope = ";
    #[allow(clippy::disallowed_macros)]
    let wrapped = format!("{PREFIX}{trimmed}");
    // Offset of `trimmed` within `wrapped`, plus the original expression's own
    // leading whitespace that `trim` removed (callers pass absolute offsets
    // relative to the raw expression source).
    let leading_ws = (expr.len() - expr.trim_start().len()) as u32;
    let base = PREFIX.len() as u32 - leading_ws;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let ret = Parser::new(&allocator, &wrapped, source_type).parse();

    let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = ret.program.body.first()
    else {
        return SmallVec::new();
    };
    let Some(init) = var_decl.declarations.first().and_then(|d| d.init.as_ref()) else {
        return SmallVec::new();
    };

    // Unwrap any parentheses the source carried (`v-scope="({ ... })"`).
    let mut expr_node = init;
    while let Expression::ParenthesizedExpression(paren) = expr_node {
        expr_node = &paren.expression;
    }

    let Expression::ObjectExpression(obj) = expr_node else {
        return SmallVec::new();
    };

    let mut bindings: SmallVec<[VScopeBinding; 4]> = SmallVec::new();
    for property in obj.properties.iter() {
        let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) = property else {
            // Spread elements (`...rest`) do not introduce statically known keys.
            continue;
        };

        // Computed keys (`[expr]: ...`) are not statically resolvable names.
        if prop.computed {
            continue;
        }

        let (name, span) = match &prop.key {
            PropertyKey::StaticIdentifier(ident) => {
                (CompactString::new(ident.name.as_str()), ident.span)
            }
            PropertyKey::StringLiteral(lit) => {
                let name = lit.value.as_str();
                if !is_identifier(name) {
                    continue;
                }
                (CompactString::new(name), lit.span)
            }
            _ => continue,
        };

        // `span.start` is relative to `wrapped`; subtracting `base` maps it
        // back onto the original (untrimmed) expression source.
        let key_offset = span.start.saturating_sub(base);
        bindings.push((name, key_offset));
    }

    bindings
}

/// Whether `name` is a valid JavaScript identifier (so a quoted key like
/// `"count"` can become a scope binding, but `"a-b"` cannot).
fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '_' || c == '$' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::extract_v_scope_bindings;

    fn names(expr: &str) -> Vec<vize_carton::CompactString> {
        extract_v_scope_bindings(expr)
            .into_iter()
            .map(|(n, _)| n)
            .collect()
    }

    fn assert_names(expr: &str, expected: &[&str]) {
        let got = names(expr);
        let got_strs: Vec<&str> = got.iter().map(|n| n.as_str()).collect();
        assert_eq!(got_strs, expected, "for expression `{expr}`");
    }

    #[test]
    fn extracts_top_level_keys_with_offsets() {
        let bindings = extract_v_scope_bindings("{ count: 0, msg: 'x' }");
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0.as_str(), "count");
        assert_eq!(bindings[0].1, 2); // `count` starts at byte 2
        assert_eq!(bindings[1].0.as_str(), "msg");
        assert_eq!(bindings[1].1, 12); // `msg` starts at byte 12
    }

    #[test]
    fn handles_quoted_shorthand_and_methods() {
        assert_names(
            "{ 'count': 0, inc() { this.count++ }, total }",
            &["count", "inc", "total"],
        );
    }

    #[test]
    fn ignores_spread_and_computed_keys() {
        assert_names("{ ...state, [key]: 1, ok: true }", &["ok"]);
    }

    #[test]
    fn non_object_yields_no_bindings() {
        assert!(names("count").is_empty());
        assert!(names("").is_empty());
        assert!(names("[1, 2, 3]").is_empty());
    }

    #[test]
    fn tolerates_leading_whitespace_offsets() {
        let bindings = extract_v_scope_bindings("  { count: 0 }");
        assert_eq!(bindings.len(), 1);
        // `count` is at byte 4 in the untrimmed source.
        assert_eq!(bindings[0].1, 4);
    }
}
