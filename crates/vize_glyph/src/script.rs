//! High-performance Script/TypeScript formatting using oxc_formatter.
//!
//! This module provides Prettier-compatible formatting for JavaScript/TypeScript
//! code using OXC's formatter (oxfmt).

mod block_identity;
mod expression_wrapper;
mod format;

use crate::error::FormatError;
use crate::options::FormatOptions;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_formatter::{QuoteStyle, format_program, parse_for_format};
use oxc_span::SourceType;
use vize_l0::{Allocator, String, ToCompactString};

pub(crate) use block_identity::format_sfc_script_content_stable;
pub(crate) use expression_wrapper::FormattedExpression;

const MAX_SCRIPT_STABILIZATION_PASSES: usize = 6;

/// Format JavaScript/TypeScript content using oxc_formatter
///
/// Uses arena allocation for efficient memory management.
#[inline]
#[cfg(test)]
pub fn format_script_content(
    source: &str,
    options: &FormatOptions,
    _allocator: &Allocator,
) -> Result<String, FormatError> {
    format::format_script_content_with_source_type(
        source,
        options,
        _allocator,
        SourceType::ts().with_module(true),
    )
}

pub(crate) fn format_script_content_stable(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    source_type: SourceType,
    sort_imports: Option<&crate::ImportSortOptions>,
) -> Result<String, FormatError> {
    if options.end_of_line == crate::EndOfLine::Auto {
        return options.format_with_source_line_ending(source, |options| {
            format_script_content_stable(source, options, allocator, source_type, sort_imports)
        });
    }
    let mut current = format::format_script_content_with_sort_imports(
        source,
        options,
        allocator,
        source_type,
        sort_imports,
    )?;
    // Check mode returns the first pass; a no-op already is a fixed point.
    if options.skip_script_stabilization {
        return Ok(current);
    }
    let mut current_trimmed_len = current.trim_end().len();
    if current.as_str().get(..current_trimmed_len) == Some(source.trim_end()) {
        return Ok(current);
    }
    for _ in 1..MAX_SCRIPT_STABILIZATION_PASSES {
        let next = match format::format_script_content_with_sort_imports(
            current.as_str(),
            options,
            allocator,
            source_type,
            sort_imports,
        ) {
            Ok(next) => next,
            Err(_) => return Ok(current),
        };
        let next_trimmed = next.trim_end();
        if current.as_str().get(..current_trimmed_len) == Some(next_trimmed) {
            return Ok(next);
        }
        current_trimmed_len = next_trimmed.len();
        current = next;
    }

    Ok(current)
}

pub(crate) fn format_ts_script_content_stable(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
) -> Result<String, FormatError> {
    format_script_content_stable(
        source,
        options,
        allocator,
        SourceType::ts().with_module(true),
        None,
    )
}

pub(crate) fn source_type_for_script_lang(lang: Option<&str>) -> SourceType {
    match lang {
        Some("jsx") => SourceType::jsx().with_module(true),
        Some("tsx") => SourceType::tsx().with_module(true),
        _ => SourceType::ts().with_module(true),
    }
}

thread_local! {
    /// Per-thread scratch reused across template-expression formats. A single
    /// template can call `format_js_expression` thousands of times (once per
    /// interpolation / directive value); reusing the arena (reset between calls)
    /// avoids an arena chunk alloc+teardown per call, and reusing the `void (…)`
    /// wrapper buffer avoids a heap allocation per call. The CLI formats files in
    /// parallel, so per-thread state keeps each worker independent and lock-free.
    static EXPR_SCRATCH: core::cell::RefCell<(OxcAllocator, String)> =
        core::cell::RefCell::new((OxcAllocator::default(), String::from("void (")));
}

/// Format a JS expression (for use in template directive values and interpolations).
/// Returns None if the expression cannot be parsed/formatted.
pub fn format_js_expression(expr: &str, options: &FormatOptions) -> Option<String> {
    format_js_expression_with_quote_style(expr, options, None).map(|formatted| formatted.code)
}

/// HTML attributes use double quotes independently of the script quote option.
/// Single-quoted JavaScript strings keep their delimiters out of the attribute.
pub(crate) fn format_js_expression_in_attribute(
    expr: &str,
    options: &FormatOptions,
) -> Option<String> {
    format_js_expression_in_attribute_with_layout(expr, options).map(|formatted| formatted.code)
}

pub(crate) fn format_js_expression_in_attribute_with_layout(
    expr: &str,
    options: &FormatOptions,
) -> Option<FormattedExpression> {
    format_js_expression_with_quote_style(expr, options, Some(QuoteStyle::Single))
}

pub(crate) fn format_js_expression_with_quote_style(
    expr: &str,
    options: &FormatOptions,
    quote_style: Option<QuoteStyle>,
) -> Option<FormattedExpression> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Some(FormattedExpression {
            code: String::default(),
            retained_bare_sequence: false,
        });
    }

    EXPR_SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        let (oxc_allocator, wrapped) = &mut *scratch;
        // Recycle the arena memory instead of allocating/freeing a fresh one.
        oxc_allocator.reset();

        let source_type = SourceType::ts().with_module(true);

        // Wrap the expression in a `void (…)` statement so it parses as a complete
        // statement the formatter can emit cleanly; we extract the inner part
        // below. Build the wrapper in the reused buffer (no per-call allocation).
        wrapped.truncate("void (".len());
        wrapped.push_str(trimmed);
        wrapped.push(')');
        let parsed = parse_for_format(oxc_allocator, wrapped.as_str(), source_type);

        if !parsed.diagnostics.is_empty() {
            return None;
        }

        let mut oxc_options = options.to_oxc_format_options();
        if let Some(quote_style) = quote_style {
            oxc_options.quote_style = quote_style;
        }
        let formatted = format_program(oxc_allocator, &parsed.program, oxc_options, None)
            .print()
            .ok()?
            .into_code();

        // Extract the expression back from the formatted output.
        // preserve_parens is false, so the formatter may remove the wrapping parens.
        // Expected forms:  "void expression;\n"  or  "void (expression);\n"
        let formatted = formatted.trim();
        let formatted = formatted.strip_suffix(';').unwrap_or(formatted);
        let inner = formatted.strip_prefix("void ").unwrap_or(formatted);

        // The retained AST distinguishes the wrapper from authored call/member
        // parentheses and sequences whose grouping Vue consumers require.
        let inner = expression_wrapper::unwrap_argument(inner, &parsed.program, trimmed)?;

        Some(FormattedExpression {
            code: inner.text.trim().to_compact_string(),
            retained_bare_sequence: inner.retained_bare_sequence,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::format::format_script_content_with_source_type;
    use super::{Allocator, FormatOptions, format_js_expression, format_script_content};
    use oxc_span::SourceType;
    use vize_l0::String;

    #[test]
    fn test_format_simple_script() {
        let source = "const x=1";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_with_imports() {
        let source = "import {ref,computed} from 'vue'";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_tsx_component_script() {
        let source =
            "const Comp=(props:{msg:string})=><section class=\"box\">{props.msg}</section>";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content_with_source_type(
            source,
            &options,
            &allocator,
            SourceType::tsx().with_module(true),
        )
        .unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_jsx_component_script() {
        let source = "const Comp=({msg})=><><span data-id=\"x\">{msg}</span></>";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content_with_source_type(
            source,
            &options,
            &allocator,
            SourceType::jsx().with_module(true),
        )
        .unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_object() {
        let source = "const obj={a:1,b:2}";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_empty_source() {
        let source = "";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_format_whitespace_only() {
        let source = "   \n\t  ";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_format_unicode_whitespace_only() {
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content("\u{a0}\u{2003}\u{2028}", &options, &allocator).unwrap();
        assert_eq!(result.as_str(), "");
    }

    #[test]
    fn test_absent_sorting_override_keeps_pinned_none_default() {
        assert!(
            oxc_formatter::JsFormatOptions::default()
                .sort_imports
                .is_none()
        );
        assert!(
            FormatOptions::default()
                .to_oxc_format_options()
                .sort_imports
                .is_none()
        );
    }

    #[test]
    fn test_expression_scratch_keeps_prefix_after_long_and_rejected_inputs() {
        let options = FormatOptions::default();
        let long = format_js_expression("longIdentifier+otherLongIdentifier", &options).unwrap();
        assert_eq!(long.as_str(), "longIdentifier + otherLongIdentifier");
        assert_eq!(format_js_expression(")", &options), None);
        let short = format_js_expression("a+b", &options).unwrap();
        assert_eq!(short.as_str(), "a + b");
    }

    #[test]
    fn test_format_js_expression_simple() {
        let options = FormatOptions::default();
        let result = format_js_expression("count+1", &options);
        assert!(result.is_some());
        let expr = result.unwrap();
        insta::assert_snapshot!(expr.as_str());
    }

    #[test]
    fn test_format_js_expression_with_optional_chaining() {
        let options = FormatOptions::default();
        let expr = format_js_expression("user?.profile?.name??'Guest'", &options).unwrap();

        insta::assert_snapshot!(expr.as_str());
    }

    #[test]
    fn test_format_js_expression_empty() {
        let options = FormatOptions::default();
        let result = format_js_expression("", &options);
        assert_eq!(result, Some(String::default()));
    }
}
