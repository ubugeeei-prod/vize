//! Legacy parse-based validity checks for the prefix rewrite (split out of
//! `rewrite.rs`; the parses here are counted re-parse sites until P1-9).

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::String;

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
