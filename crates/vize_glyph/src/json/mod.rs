//! JSON / JSONC formatting for non-SFC sources (e.g. `package.json`,
//! `tsconfig.json`).
//!
//! This is the formatter path that lets a project replace Prettier on its
//! project config files. Two entry points share one parser + printer:
//!
//! - [`format_json_source`] — strict JSON. Comments and trailing commas are
//!   errors, exactly as before.
//! - [`format_jsonc_source`] — JSON-with-comments (`.jsonc`, and the comment /
//!   trailing-comma dialect TypeScript accepts in `tsconfig.json`). Comments
//!   are preserved and trailing commas are tolerated on input and dropped on
//!   output (#2249, the follow-up to the strict-JSON pass).
//!
//! Both paths parse into a small value tree ([`ast`]), then print it
//! ([`printer`]). Object key order and scalar token text are preserved
//! verbatim; only structural whitespace is rewritten to the indent/newline
//! configured in [`FormatOptions`]. The output is idempotent: formatting
//! already-formatted source is a no-op.

use crate::error::FormatError;
use crate::options::FormatOptions;
use vize_carton::String;

mod ast;
mod parser;
mod printer;

use parser::Parser;
use printer::Printer;

/// Format a strict JSON source string.
///
/// Comments and trailing commas are rejected. The output ends with the
/// configured line terminator so it round-trips through `vize fmt --check`.
pub fn format_json_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    format_document(source, options, false)
}

/// Format a JSONC source string (JSON with `//` and `/* */` comments and
/// optional trailing commas).
///
/// Comments are preserved in source order. A comment that trails a value on the
/// same source line stays a trailing comment; a comment on its own line stays on
/// its own line. Trailing commas are accepted on input and removed on output.
pub fn format_jsonc_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    format_document(source, options, true)
}

fn format_document(
    source: &str,
    options: &FormatOptions,
    jsonc: bool,
) -> Result<String, FormatError> {
    if source.trim().is_empty() {
        return Ok(String::default());
    }

    let mut parser = Parser::new(source, jsonc);
    let leading = parser.collect_comments()?;
    let value = parser.parse_value()?;
    let trailing = parser.collect_comments()?;

    parser.skip_whitespace();
    if parser.peek().is_some() {
        return Err(json_error("trailing content after JSON value"));
    }

    let newline = options.newline_string();
    let indent = options.indent_string();
    let printer = Printer {
        indent: indent.as_str(),
        newline,
    };

    let mut output = String::with_capacity(source.len() + 32);
    for comment in &leading {
        printer.write_comment(&mut output, comment);
        output.push_str(newline);
    }
    printer.write_value(&mut output, &value, 0);
    for comment in &trailing {
        output.push_str(newline);
        printer.write_comment(&mut output, comment);
    }
    output.push_str(newline);
    Ok(output)
}

fn json_error(msg: impl Into<String>) -> FormatError {
    FormatError::JsonFormatError(msg.into())
}

fn trim_end(text: &str) -> String {
    String::from(text.trim_end())
}

#[cfg(test)]
mod tests;
