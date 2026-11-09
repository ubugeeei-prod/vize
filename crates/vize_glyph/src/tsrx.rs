//! TSRX modules: the host TypeScript formatted around verbatim markup.
//!
//! A `.tsrx` module is parsed once by the L1 TSRX grammar with the script
//! formatter's parse options. The script formatter lays out the host program
//! and prints every markup region with its authored bytes, so markup, its text
//! whitespace and its comments are never rewritten. A module with any syntax
//! error is refused, not formatted.

use oxc_formatter::{ExternalCallbacks, format_parse_options, format_program};
use oxc_span::Span;
use vize_l0::{Allocator, String, ToCompactString};
use vize_l1::markup::tsrx;

use crate::{ImportSortOptions, error::FormatError, options::FormatOptions};

/// Format a TSRX module.
pub fn format_tsrx(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    sort_imports: Option<&ImportSortOptions>,
) -> Result<String, FormatError> {
    let module = tsrx::parse_with_options(allocator, source, format_parse_options());
    if !module.is_clean() {
        let messages: std::vec::Vec<String> = module
            .diagnostics()
            .iter()
            .map(ToCompactString::to_compact_string)
            .collect();
        return Err(FormatError::ScriptParseError(messages.join("; ").into()));
    }
    let regions = module
        .regions()
        .iter()
        .map(|region| Span::new(region.span.start, region.span.end))
        .collect();
    let mut oxc_options = options.to_oxc_format_options();
    if let Some(sort_imports) = sort_imports {
        oxc_options.sort_imports = Some(sort_imports.clone());
    }
    let formatted = format_program(
        allocator.as_oxc(),
        module.program(),
        oxc_options,
        Some(ExternalCallbacks::new().with_verbatim_spans(regions)),
    );
    let printed = formatted
        .print()
        .map_err(|error| FormatError::ScriptFormatError(error.to_compact_string()))?;
    Ok(printed.into_code().into())
}
