use crate::{error::FormatError, options::FormatOptions};
use oxc_allocator::Allocator as OxcAllocator;
use oxc_formatter::format_program;
use oxc_span::SourceType;
use vize_l0::{Allocator, String, ToCompactString};
use vize_l1::embed::syntax::parse_program_for_format;

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
    _allocator: &Allocator,
    source_type: SourceType,
) -> Result<String, FormatError> {
    format_script_content_with_sort_imports(source, options, _allocator, source_type, None)
}

pub(super) fn format_script_content_with_sort_imports(
    source: &str,
    options: &FormatOptions,
    _allocator: &Allocator,
    source_type: SourceType,
    sort_imports: Option<&crate::ImportSortOptions>,
) -> Result<String, FormatError> {
    // ASCII above space cannot start whitespace-only input. Other bytes keep
    // the complete Unicode whitespace check.
    if source.as_bytes().first().is_none_or(|first| {
        (*first <= b' ' || !first.is_ascii()) && source.chars().all(char::is_whitespace)
    }) {
        return Ok(String::default());
    }

    // A fresh printer arena per pass: the printed AST is never retained.
    let oxc_allocator = OxcAllocator::default();

    // The L1 formatter profile is the parse `format_program` requires
    // (`preserve_parens: false`, JSX enabled for JavaScript source types);
    // the printer may panic on an AST parsed any other way. Any diagnostic
    // refuses the parse, so recovered syntax is never printed.
    let parsed = parse_program_for_format(&oxc_allocator, source, source_type);

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

    // Convert options and format
    let mut oxc_options = options.to_oxc_format_options();
    if let Some(sort_imports) = sort_imports {
        oxc_options.sort_imports = Some(sort_imports.clone());
    }
    let formatted = format_program(&oxc_allocator, &parsed.program, oxc_options, None);
    let printed = formatted
        .print()
        .map_err(|error| FormatError::ScriptFormatError(error.to_compact_string()))?;

    Ok(printed.into_code().into())
}
