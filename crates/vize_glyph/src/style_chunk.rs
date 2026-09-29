use super::{
    FormatError, FormatOptions, ParserOptions, PrinterOptions, String, StyleSheet, ToCompactString,
};

pub(super) fn format_chunk(trimmed: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let colors = super::color::protect(trimmed);
    let formatted =
        super::stabilization::format_to_fixed_point(colors.source.as_str(), |source| {
            format_chunk_once(source, options)
        })?;
    let formatted = colors.restore(formatted);
    // The CSS printer also performs syntax and value normalization. A formatter
    // must never silently change browser support or the scoped selector target.
    // Format only structural whitespace when the print changes authored CSS.
    if super::authored::changes_authored_css(trimmed, formatted.as_str()) {
        Ok(super::authored::format_layout_only(trimmed, options))
    } else {
        Ok(formatted)
    }
}

fn format_chunk_once(trimmed: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let stylesheet = StyleSheet::parse(trimmed, ParserOptions::default())
        .map_err(|e| FormatError::StyleFormatError(e.to_compact_string()))?;

    let indent_width = options.tab_width;
    let printer_options = PrinterOptions {
        minify: false,
        ..Default::default()
    };

    let result = stylesheet
        .to_css(printer_options)
        .map_err(|e| FormatError::StyleFormatError(e.to_compact_string()))?;

    // lightningcss omits leading zeroes even with minify disabled; Oxfmt keeps
    // them in standalone CSS, so align the style block's printed number tokens.
    let mut code = super::number::add_leading_zero_to_fractional_numbers(&result.code);

    // lightningcss uses 2-space indent by default; re-indent if needed
    if options.use_tabs || indent_width != 2 {
        code = super::authored::reindent_css(&code, options);
    }

    Ok(code)
}

pub(super) fn contains_comment(source: &str) -> bool {
    memchr::memmem::find(source.as_bytes(), b"/*").is_some()
}
