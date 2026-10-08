//! Existing single-parse JavaScript expression formatting.

use super::FormattedExpression;
use super::expression_wrapper;
use crate::options::FormatOptions;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_formatter::{QuoteStyle, format_program, parse_for_format};
use oxc_formatter_core::LineWidth;
use oxc_span::SourceType;
use vize_l0::{String, ToCompactString};

#[inline]
fn trim_expression(expression: &str) -> &str {
    let bytes = expression.as_bytes();
    // Graphic ASCII edges make Unicode trimming a no-op. Whitespace,
    // controls and non-ASCII edges retain the original predicate.
    if bytes.first().is_some_and(|byte| byte.is_ascii_graphic())
        && bytes.last().is_some_and(|byte| byte.is_ascii_graphic())
    {
        expression
    } else {
        expression.trim()
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
    format_js_expression_with_quote_style(expr, options, Some(QuoteStyle::Single))
        .map(|formatted| formatted.code)
}

pub(crate) fn format_js_expression_in_attribute_with_layout(
    expr: &str,
    options: &FormatOptions,
    attribute_depth: usize,
) -> Option<FormattedExpression> {
    format_expression_with_layout(
        expr,
        options,
        Some(QuoteStyle::Single),
        Some(attribute_depth),
    )
}

pub(crate) fn format_js_expression_with_quote_style(
    expr: &str,
    options: &FormatOptions,
    quote_style: Option<QuoteStyle>,
) -> Option<FormattedExpression> {
    format_expression_with_layout(expr, options, quote_style, None)
}

fn format_expression_with_layout(
    expr: &str,
    options: &FormatOptions,
    quote_style: Option<QuoteStyle>,
    attribute_depth: Option<usize>,
) -> Option<FormattedExpression> {
    let trimmed = trim_expression(expr);
    if trimmed.is_empty() {
        return Some(FormattedExpression {
            code: String::default(),
            retained_bare_sequence: false,
            owns_attribute_lines: false,
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
        if let Some(depth) = attribute_depth
            && let [Statement::ExpressionStatement(statement)] = parsed.program.body.as_slice()
            && let Expression::UnaryExpression(unary) = &statement.expression
            && matches!(&unary.argument, Expression::CallExpression(_))
        {
            // Only retained direct calls consume the attribute-value budget.
            // Other expression shapes keep the existing program formatter.
            let value_indent = (depth + 2) * options.tab_width as usize;
            let width = options
                .print_width
                .saturating_sub(value_indent as u32)
                .max(1);
            oxc_options.line_width = LineWidth::try_from(width as u16).unwrap_or_default();
            let code = oxc_formatter::format_unary_call_argument(
                oxc_allocator,
                &parsed.program,
                oxc_options,
            )?
            .print()
            .ok()?
            .into_code();
            let code = trim_expression(&code);
            return Some(FormattedExpression {
                code: code.to_compact_string(),
                retained_bare_sequence: false,
                owns_attribute_lines: code.contains('\n'),
            });
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
            code: trim_expression(inner.text).to_compact_string(),
            retained_bare_sequence: inner.retained_bare_sequence,
            owns_attribute_lines: false,
        })
    })
}
