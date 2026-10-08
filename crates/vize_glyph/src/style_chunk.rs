use super::{
    FormatError, FormatOptions, ParserOptions, PrinterOptions, String, StyleSheet, ToCompactString,
};

pub(super) fn format_chunk(trimmed: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let colors = super::color::protect(trimmed);
    let collect_layout = super::rule_layout::RuleLayout::may_need_layout(trimmed);
    let mut layout = None;
    let formatted =
        super::stabilization::format_to_fixed_point(colors.source.as_str(), |source| {
            format_chunk_once(source, options, &mut layout, collect_layout)
        })?;
    let formatted = colors.restore(formatted);
    // Complete layout equality also proves authored tokens and groups survived.
    if formatted.as_str().trim() == trimmed
        && !layout
            .as_ref()
            .is_some_and(|layout: &super::rule_layout::RuleLayout| {
                layout.multi_values || layout.preludes.iter().any(|prelude| prelude.selector_list)
            })
    {
        return Ok(formatted);
    }
    // The CSS printer also performs syntax and value normalization. A formatter
    // must never silently change browser support or the scoped selector target.
    // Format only structural whitespace when the print changes authored CSS.
    let formatted = if super::authored::changes_authored_css(trimmed, formatted.as_str()) {
        super::authored::format_layout_only(trimmed, options)
    } else {
        formatted
    };
    Ok(super::blank_lines::preserve_rule_layout(
        trimmed,
        formatted,
        options,
        layout.as_ref(),
        colors.source.as_str(),
    ))
}

fn format_chunk_once(
    trimmed: &str,
    options: &FormatOptions,
    layout: &mut Option<super::rule_layout::RuleLayout>,
    collect_layout: bool,
) -> Result<String, FormatError> {
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
    if options.use_tabs || indent_width != 2 || options.newline_bytes() != b"\n" {
        code = super::authored::reindent_css(&code, options);
    }

    if collect_layout && layout.is_none() {
        // Identical bytes without any comma cannot need selector or gap edits.
        // Retain an empty first-parse marker so later passes never own layout.
        *layout = Some(
            if memchr::memchr(b',', trimmed.as_bytes()).is_none() && code.as_str().trim() == trimmed
            {
                super::rule_layout::RuleLayout::default()
            } else {
                super::rule_layout::RuleLayout::from_parse(trimmed, &stylesheet.rules)
            },
        );
    }

    Ok(code)
}

pub(super) fn contains_comment(source: &str) -> bool {
    memchr::memmem::find(source.as_bytes(), b"/*").is_some()
}
