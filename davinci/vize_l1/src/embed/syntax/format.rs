//! The formatter parse profile: whole-Program syntax a code printer may print.
//!
//! The compiler profile ([`super::parse_program_once`]) retains parentheses and
//! never admits formatter options. A printer instead needs the parse its
//! formatter was written against: no retained parentheses, no regular
//! expression parsing, `return` outside a function and V8 intrinsics accepted,
//! and JSX enabled for every JavaScript source type. The compiler profile hands
//! out `EmbedSyntax`, never a bare parser result, so the two parses are never
//! confused for each other.

use oxc_allocator::Allocator;
/// The parser's result, as a printer receives it.
pub use oxc_parser::ParserReturn;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

/// Parser options of the formatter profile.
///
/// Identifier hashes stay off: printers read syntax, not semantic identity.
pub const FORMAT_PARSE_OPTIONS: ParseOptions = ParseOptions {
    parse_regular_expression: false,
    allow_return_outside_function: true,
    allow_v8_intrinsics: true,
    preserve_parens: false,
    enable_ident_hashes: false,
};

/// The parse a JavaScript source type receives under the formatter profile.
///
/// JSX has no syntax conflict with JavaScript, so it is always enabled there.
/// TypeScript keeps its explicit variant: `<T>value` is a type assertion in
/// `.ts` and an element in `.tsx`.
#[must_use]
#[inline]
pub fn format_source_type(source_type: SourceType) -> SourceType {
    if source_type.is_javascript() {
        source_type.with_jsx(true)
    } else {
        source_type
    }
}

/// Parse `source` once under the formatter profile into `allocator`.
///
/// A printer may only print a clean parse: it must refuse the result on any
/// diagnostic, warning or error, because a recovered AST can be an unfaithful
/// repair of the authored source and printing it would silently rewrite it.
/// The parse result is returned as the parser built it, without a move into
/// another owner, because printers call this once per embedded expression.
///
/// The arena is the printer's own: a formatter prints into the arena it parsed
/// into, and a `vize_l0::Allocator` dereferences to it. File and container
/// language resolution remain the caller's: the source type is taken as given,
/// apart from the JSX rule of [`format_source_type`].
#[must_use]
#[inline]
pub fn parse_program_for_format<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> ParserReturn<'a> {
    Parser::new(allocator, source, format_source_type(source_type))
        .with_options(FORMAT_PARSE_OPTIONS)
        .parse()
}

#[cfg(test)]
mod tests;
