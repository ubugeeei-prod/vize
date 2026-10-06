use crate::{error::FormatError, options::FormatOptions};
use oxc_allocator::Allocator as OxcAllocator;
use oxc_formatter::{JsFormatOptions, format_program, parse_for_format};
use oxc_span::SourceType;
use vize_l0::{String, ToCompactString};

/// Format JavaScript/TypeScript/JSX/TSX content using an explicit OXC source type.
///
/// SFC formatting calls this with the script block's `lang` attribute so
/// `<script lang="jsx">` and `<script lang="tsx">` preserve JSX syntax instead
/// of falling back through the non-JSX TypeScript parser.
#[inline]
#[cfg(test)]
pub fn format_script_content_with_source_type(
    source: &str,
    options: &FormatOptions,
    _allocator: &vize_l0::Allocator,
    source_type: SourceType,
) -> Result<String, FormatError> {
    format_script_content_with_sort_imports(source, options, source_type, None, &mut None)
}

pub(super) fn format_script_content_with_sort_imports(
    source: &str,
    options: &FormatOptions,
    source_type: SourceType,
    sort_imports: Option<&crate::ImportSortOptions>,
    prepared_options: &mut Option<JsFormatOptions>,
) -> Result<String, FormatError> {
    // Fast path for empty content
    if source.chars().all(char::is_whitespace) {
        return Ok(String::default());
    }

    // Use OXC's allocator for parsing (required by oxc_parser)
    let oxc_allocator = OxcAllocator::default();

    // Parse the source with formatter-compatible options. `parse_for_format` is
    // the parse the formatter requires (`preserve_parens: false`, hashed
    // identifiers, JSX enabled for JavaScript source types); `format_program`
    // may panic on an AST parsed any other way.
    let parsed = parse_for_format(&oxc_allocator, source, source_type);

    if !parsed.diagnostics.is_empty() {
        let error_messages: Vec<String> = parsed
            .diagnostics
            .iter()
            .map(|e| e.to_compact_string())
            .collect();
        return Err(FormatError::ScriptParseError(
            error_messages.join("; ").into(),
        ));
    }

    // Every stabilization pass uses the same base settings. Initialize only
    // after a valid parse, and keep configured import sorting per invocation.
    let mut oxc_options = prepared_options
        .get_or_insert_with(|| options.to_oxc_format_options())
        .clone();
    if let Some(sort_imports) = sort_imports {
        oxc_options.sort_imports = Some(sort_imports.clone());
    }
    let formatted = format_program(&oxc_allocator, &parsed.program, oxc_options, None);
    let printed = formatted
        .print()
        .map_err(|error| FormatError::ScriptFormatError(error.to_compact_string()))?;

    Ok(printed.into_code().into())
}
