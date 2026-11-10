//! TSRX modules: one L1 tree for host TypeScript and markup layout.
//!
//! A `.tsrx` module is parsed once by the L1 TSRX grammar with the script
//! formatter's parse options. Glyph lays out every markup region from that tree,
//! delegates embedded TypeScript to the script formatter, then lets the host
//! formatter lay out the surrounding module. Any parse error or structural
//! change is refused.

mod layout;

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
    if options.end_of_line == crate::EndOfLine::Auto {
        return options.format_with_source_line_ending(source, |options| {
            format_tsrx(source, options, allocator, sort_imports)
        });
    }
    let module = tsrx::parse_with_options(allocator, source, format_parse_options());
    require_clean(&module)?;

    // First give every region canonical bytes. The host formatter then moves
    // those complete regions with their surrounding statements. A final region
    // pass derives continuation indentation from the canonical host position.
    let laid_out = layout::format_regions(&module, source, options, allocator)?;
    let laid_out_module =
        tsrx::parse_with_options(allocator, laid_out.as_str(), format_parse_options());
    require_clean(&laid_out_module)?;
    layout::require_same_structure(&module, source, &laid_out_module, laid_out.as_str())?;

    let regions = laid_out_module
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
        laid_out_module.program(),
        oxc_options,
        Some(ExternalCallbacks::new().with_verbatim_spans(regions)),
    );
    let printed = formatted
        .print()
        .map_err(|error| FormatError::ScriptFormatError(error.to_compact_string()))?;
    let host: String = printed.into_code().into();
    let host_module = tsrx::parse_with_options(allocator, host.as_str(), format_parse_options());
    require_clean(&host_module)?;
    layout::require_same_structure(&module, source, &host_module, host.as_str())?;

    let code = layout::format_regions(&host_module, host.as_str(), options, allocator)?;
    let final_module = tsrx::parse_with_options(allocator, code.as_str(), format_parse_options());
    require_clean(&final_module)?;
    layout::require_same_structure(&module, source, &final_module, code.as_str())?;
    Ok(code)
}

fn require_clean(module: &tsrx::TsrxModule<'_>) -> Result<(), FormatError> {
    if module.is_clean() {
        return Ok(());
    }
    let messages: std::vec::Vec<String> = module
        .diagnostics()
        .iter()
        .map(ToCompactString::to_compact_string)
        .collect();
    Err(FormatError::ScriptParseError(messages.join("; ").into()))
}
