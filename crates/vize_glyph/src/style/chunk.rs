use super::{
    FormatError, FormatOptions, ParserOptions, PrinterOptions, String, StyleSheet, ToCompactString,
};

pub(super) fn format_chunk(trimmed: &str, options: &FormatOptions) -> Result<String, FormatError> {
    // Past this depth the engine only reports a parse error, after
    // exponential work; report the same kind of error without it. The check
    // comes before color protection, which already hands authored values to
    // the engine.
    if vize_l1::css::nesting::css_nesting_exceeds_max_depth(trimmed) {
        return Err(nesting_depth_error());
    }
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
        super::authored::format_layout_only(trimmed, options).ok_or_else(nesting_depth_error)?
    } else {
        super::paren_breaks::restore(trimmed, formatted)
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
    let stylesheet =
        super::engine_boundary::catch(|| StyleSheet::parse(trimmed, ParserOptions::default()))?
            .map_err(|e| FormatError::StyleFormatError(e.to_compact_string()))?;
    let indent_width = options.tab_width;
    let printer_options = PrinterOptions {
        minify: false,
        ..Default::default()
    };

    let result = super::engine_boundary::catch(|| stylesheet.to_css(printer_options))?
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

#[cold]
#[inline(never)]
fn nesting_depth_error() -> FormatError {
    FormatError::StyleFormatError(vize_l1::css::nesting::NESTING_DEPTH_ERROR.into())
}

pub(super) fn contains_comment(source: &str) -> bool {
    memchr::memmem::find(source.as_bytes(), b"/*").is_some()
}

#[cfg(test)]
mod tests {
    use super::format_chunk;
    use crate::{FormatError, FormatOptions};
    use vize_l1::css::nesting::{MAX_CSS_NESTING_DEPTH, NESTING_DEPTH_ERROR};

    /// One declaration whose value nests `depth` brackets, the rule's own
    /// block included.
    fn nested_value(depth: usize) -> std::string::String {
        let functions = depth - 1;
        std::format!(
            ".a {{\n  width: {}1px{};\n}}",
            "calc(".repeat(functions),
            ")".repeat(functions)
        )
    }

    #[track_caller]
    fn assert_nesting_depth_error(source: &str) {
        let Err(FormatError::StyleFormatError(message)) =
            format_chunk(source, &FormatOptions::default())
        else {
            panic!("nesting past the limit must be a style format error");
        };
        assert_eq!(message.as_str(), NESTING_DEPTH_ERROR);
    }

    #[test]
    fn nesting_at_the_limit_formats() {
        let source = nested_value(MAX_CSS_NESTING_DEPTH);
        let formatted = format_chunk(&source, &FormatOptions::default()).unwrap();
        assert_eq!(formatted.as_str().trim_end(), source);
    }

    #[test]
    fn nesting_past_the_limit_is_a_format_error() {
        assert_nesting_depth_error(&nested_value(MAX_CSS_NESTING_DEPTH + 1));
        assert_nesting_depth_error(&"(".repeat(MAX_CSS_NESTING_DEPTH + 1));
    }

    #[test]
    fn unclosed_brackets_at_the_limit_are_not_a_nesting_error() {
        let Err(FormatError::StyleFormatError(message)) = format_chunk(
            &"(".repeat(MAX_CSS_NESTING_DEPTH),
            &FormatOptions::default(),
        ) else {
            panic!("unclosed brackets do not parse");
        };
        assert_ne!(message.as_str(), NESTING_DEPTH_ERROR);
    }

    #[test]
    fn nested_color_arguments_far_past_the_limit_are_the_same_format_error() {
        // Color protection reads authored values before the style sheet is
        // parsed, so the limit applies before it too.
        let source = std::format!(".a {{ color: rgb({}", "calc(".repeat(50_000));
        assert_nesting_depth_error(&source);
    }
}
