//! Legacy parse-based validity checks for the prefix rewrite (split out of
//! `rewrite.rs`; the parses here are counted re-parse sites until P1-9).

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::String;

/// Validate an authored expression that has no retained AST.
///
/// Retained expressions already prove that parsing succeeded. This cold path
/// is only for opaque Vapor inputs: accept either one expression or a valid
/// statement body, while preserving the shared depth refusal as a tolerant
/// passthrough.
pub(super) fn validate_unretained_expression(
    content: &str,
    is_ts: bool,
) -> Result<(), Option<String>> {
    if super::expression_exceeds_max_depth(content) {
        return Err(None);
    }
    if !super::expression_has_balanced_delimiters(content) {
        return Err(Some(String::new("mismatched expression delimiters")));
    }

    let source_type = if is_ts {
        SourceType::ts().with_module(true)
    } else {
        SourceType::default().with_module(true)
    };
    let expression_allocator = crate::expr_parse_probe::parse_arena();
    let mut wrapped = String::with_capacity(content.len() + 3);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push('\n');
    wrapped.push(')');
    let expression = Parser::new(&expression_allocator, &wrapped, source_type).parse_expression();
    if expression.is_ok() {
        return Ok(());
    }

    let program_allocator = crate::expr_parse_probe::parse_arena();
    let program = Parser::new(&program_allocator, content, source_type).parse();
    if program.diagnostics.is_empty() {
        return Ok(());
    }

    Err(Some(
        expression
            .err()
            .and_then(|errors| {
                errors
                    .first()
                    .map(|error| String::new(error.message.as_ref()))
            })
            .unwrap_or_else(|| String::new("invalid expression")),
    ))
}

/// Returns true when `content` parses as a TypeScript expression or program.
///
/// Only consulted on the parse-failure path for `is_ts` templates: when the
/// TypeScript-stripping step falls back to the original source, the plain-JS
/// parse below can fail even though the expression is valid TypeScript that
/// the official compiler (babel with the `typescript` plugin) accepts. The
/// parity rule is that vize must not reject what the official compiler
/// accepts, so such expressions keep the silent passthrough behavior.
pub(super) fn parses_as_typescript(content: &str) -> bool {
    let source_type = SourceType::ts().with_module(true);

    let expr_allocator = crate::expr_parse_probe::parse_arena();
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    if Parser::new(&expr_allocator, &wrapped, source_type)
        .parse_expression()
        .is_ok()
    {
        return true;
    }

    let program_allocator = crate::expr_parse_probe::parse_arena();
    Parser::new(&program_allocator, content, source_type)
        .parse()
        .diagnostics
        .is_empty()
}

/// Validate `content` as a parameter list by parsing the synthesized arrow
/// `(content) => null`. Synthesized text: no retained AST corresponds, so
/// this parse stays (P1-7 fallback class — binding patterns).
pub(super) fn parse_as_params(content: &str, source_type: SourceType) -> Result<(), String> {
    let allocator = crate::expr_parse_probe::parse_arena();
    let mut wrapped = String::with_capacity(content.len() + 12);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push_str(") => null");

    let parser = Parser::new(&allocator, &wrapped, source_type);
    parser.parse_expression().map(|_| ()).map_err(|errors| {
        errors
            .first()
            .map(|error| String::new(error.message.as_ref()))
            .unwrap_or_else(|| String::new("invalid parameters"))
    })
}
