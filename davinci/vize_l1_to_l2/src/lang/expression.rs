// Expression admission for the lowering. Module documentation remains on the
// compatibility owner while language policy lives here.

use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::expression_guard::scan::line_comment_terminator;
use vize_l0::{Allocator, Span, String, cstr};
use vize_l2::expr::{ExprRef, ForeignExpr, OpaqueExpr, OpaqueReason, VueFilterExpr};
use vize_l2::provenance::ProvenanceSink;

use super::cx::Cx;

/// Trim `text` and return the trimmed slice with its authored span.
/// Trimming keeps the slice a source slice, so spans stay exact.
pub(crate) fn trimmed<'a>(cx: &Cx<'a, impl ProvenanceSink>, text: &'a str) -> (&'a str, Span) {
    let slice = text.trim();
    (slice, cx.span_of(slice))
}

fn trimmed_attribute<'a>(cx: &Cx<'a, impl ProvenanceSink>, text: &'a str) -> (&'a str, Span) {
    let slice = trimmed_attribute_expression_text(cx.allocator, text);
    (slice, cx.span_of(slice))
}

/// Trim expression trivia without discarding the line break that terminates a
/// trailing line comment. The retained break is part of Vue's expression
/// grammar and keeps the shared L2 parse authoritative for lint diagnostics.
/// Only an expression that ends inside a `//` comment keeps it; a `//` on an
/// earlier line, in a string or in a block comment leaves plain trimming.
fn trimmed_attribute_expression_text<'t>(allocator: &Allocator, text: &'t str) -> &'t str {
    let text = text.trim_start();
    let trimmed = text.trim_end();
    if !trimmed.contains("//") {
        return trimmed;
    }
    let suffix = text.get(trimmed.len()..).unwrap_or_default();
    let terminator = line_comment_terminator(suffix.as_bytes(), 0);
    if terminator.is_empty() || !ends_in_line_comment(allocator, trimmed) {
        trimmed
    } else {
        text.get(..trimmed.len() + terminator.end)
            .unwrap_or(trimmed)
    }
}

/// Whether `source` ends inside a `//` comment, decided by the OXC lexer so
/// strings, templates, regexes and block comments never count.
fn ends_in_line_comment(allocator: &Allocator, source: &str) -> bool {
    let oxc = allocator.as_oxc();
    let wrapped = oxc.alloc_concat_strs_array(["(", source, "\n)"]);
    let parsed = Parser::new(oxc, wrapped, SourceType::ts()).parse();
    parsed
        .program
        .comments
        .last()
        .is_some_and(|comment| comment.is_line() && comment.span.end as usize == source.len() + 1)
}

#[cfg(test)]
mod trim_tests {
    use super::trimmed_attribute_expression_text;
    use vize_l0::Allocator;

    #[test]
    fn trailing_line_comments_keep_exactly_their_authored_terminator() {
        let allocator = Allocator::new();
        let trim = |text| trimmed_attribute_expression_text(&allocator, text);
        assert_eq!(trim("  id // note\n  "), "id // note\n");
        assert_eq!(trim("id // note\r\n\t"), "id // note\r\n");
        assert_eq!(trim("id // note"), "id // note");
        assert_eq!(trim("a + 1\n  "), "a + 1");
    }

    #[test]
    fn line_comments_before_the_last_token_keep_plain_trimming() {
        let allocator = Allocator::new();
        let trim = |text| trimmed_attribute_expression_text(&allocator, text);
        assert_eq!(
            trim("\n  // https://example.com/a\n  $attrs.style as CSSProperties\n"),
            "// https://example.com/a\n  $attrs.style as CSSProperties"
        );
        assert_eq!(
            trim("cn(\n  // @ts-ignore\n  a?.[b],\n)\n  "),
            "cn(\n  // @ts-ignore\n  a?.[b],\n)"
        );
        assert_eq!(trim("'http://x'\n"), "'http://x'");
        assert_eq!(trim("a /* // b */\n"), "a /* // b */");
    }
}

/// Lower one expression position: trim, then admit through the shared
/// JS rule. Total — refused text comes back as the classified escape.
pub(crate) fn expr_at<'a>(cx: &Cx<'a, impl ProvenanceSink>, text: &'a str) -> ExprRef<'a> {
    let (slice, span) = trimmed_attribute(cx, text);
    coordinates::expression(cx, slice, span, false)
}

/// The same dialect selection for synthesized shorthand text at an authored
/// argument span. Such text may be arena-owned rather than an L0 slice.
pub(crate) fn expression_in<'a>(
    cx: &Cx<'a, impl ProvenanceSink>,
    slice: &'a str,
    span: Span,
) -> ExprRef<'a> {
    if let Some(dialect) = cx.foreign_dialect {
        return ExprRef::Foreign(cx.allocator.alloc(ForeignExpr {
            dialect: dialect.name,
            source: slice,
            span,
            facts: vize_l0::Vec::new_in(&cx.allocator),
        }));
    }
    ExprRef::parse_js_in(cx.allocator, slice, span)
}

/// `v-on` handler admission: a lone expression keeps the retained JS
/// payload, while a valid statement body that cannot be represented by one
/// expression is classified explicitly as `Opaque(MultiStatement)`.
pub(crate) fn handler_expr_at<'a>(cx: &Cx<'a, impl ProvenanceSink>, text: &'a str) -> ExprRef<'a> {
    let expr = expr_at(cx, text);
    let ExprRef::Opaque(opaque) = expr else {
        return expr;
    };
    if opaque.reason != OpaqueReason::ParseRejected || !parses_as_handler_body(cx, opaque.source) {
        return expr;
    }
    opaque_at(cx, OpaqueReason::MultiStatement, opaque.source, opaque.span)
}

fn parses_as_handler_body(cx: &Cx<'_, impl ProvenanceSink>, source: &str) -> bool {
    if contains_module_declaration(cx, source) {
        return false;
    }
    let mut wrapped = String::with_capacity(source.len() + 39);
    wrapped.push_str("const __vize_handler__ = $event => {\n");
    wrapped.push_str(source);
    wrapped.push_str("\n};");
    let parsed = Parser::new(
        cx.allocator.as_oxc(),
        wrapped.as_str(),
        SourceType::ts().with_module(true),
    )
    .parse();
    if !parsed.diagnostics.is_empty() {
        return false;
    }
    let semantic = SemanticBuilder::new_compiler().build(&parsed.program);
    semantic.diagnostics.is_empty()
}

fn contains_module_declaration(cx: &Cx<'_, impl ProvenanceSink>, source: &str) -> bool {
    let parsed = Parser::new(
        cx.allocator.as_oxc(),
        source,
        SourceType::ts().with_module(true),
    )
    .parse();
    if !parsed.diagnostics.is_empty() {
        return false;
    }
    parsed.program.body.iter().any(|statement| {
        matches!(
            statement,
            Statement::ImportDeclaration(_)
                | Statement::ExportAllDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
                | Statement::ExportNamedDeclaration(_)
        )
    })
}

/// Interpolation / `v-bind` value admission: Vue 2 pipe filters first
/// when the dialect asks, so `|` is not bitwise-OR. Other directive
/// expressions stay on [`expr_at`].
pub(crate) fn filter_expr_at<'a>(cx: &Cx<'a, impl ProvenanceSink>, text: &'a str) -> ExprRef<'a> {
    if cx.foreign_dialect.is_some() {
        return expr_at(cx, text);
    }
    let (slice, span) = trimmed_attribute(cx, text);
    coordinates::expression(cx, slice, span, cx.caps.supports_filters)
}

/// Interpolations are trimmed by Vue before it wraps and parses them, so a
/// final line break cannot terminate a trailing `//` comment there.
pub(crate) fn interpolation_expr_at<'a>(
    cx: &Cx<'a, impl ProvenanceSink>,
    text: &'a str,
) -> ExprRef<'a> {
    if cx.foreign_dialect.is_some() {
        return expression_in(cx, text.trim(), cx.span_of(text.trim()));
    }
    let (slice, span) = trimmed(cx, text);
    coordinates::expression(cx, slice, span, cx.caps.supports_filters)
}

/// A position-classified escape at an exact place: the reasons only the
/// lowering can assign, because they are facts about where the text came
/// from, never recoverable from the text (P2-5b).
pub(crate) fn opaque_at<'a>(
    cx: &Cx<'a, impl ProvenanceSink>,
    reason: OpaqueReason,
    slice: &'a str,
    span: Span,
) -> ExprRef<'a> {
    ExprRef::Opaque(cx.allocator.alloc(OpaqueExpr {
        reason,
        source: slice,
        span,
    }))
}

/// The provenance spelling of an admitted reference: `js`, `foreign`, or
/// `opaque(<reason>)` — the mnemonic vocabulary the folio uses.
pub(crate) fn desc(expr: &ExprRef<'_>) -> String {
    match expr {
        ExprRef::Js(_) | ExprRef::Foreign(_) => String::from(expr.mnemonic()),
        ExprRef::Filter(_) => String::from("vue.filter"),
        ExprRef::Opaque(opaque) => cstr!("opaque({})", opaque.reason.mnemonic()),
    }
}

/// The identifier an admitted expression binds, when it is exactly one:
/// the only name enumeration P2-8 does. Patterns return `None` — the
/// single identifier-extraction implementation is the
/// `ExprDialect::enumerate_bindings` seam (#4365), not a scanner here.
pub(crate) fn simple_identifier<'a>(expr: &ExprRef<'a>) -> Option<&'a str> {
    match expr {
        ExprRef::Js(js) => match js.ast {
            oxc_ast::ast::Expression::Identifier(_) => Some(js.source),
            _ => None,
        },
        ExprRef::Foreign(_) | ExprRef::Filter(_) | ExprRef::Opaque(_) => None,
    }
}
