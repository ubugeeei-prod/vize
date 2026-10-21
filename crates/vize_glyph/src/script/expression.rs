//! Existing single-parse JavaScript expression formatting.

use super::FormattedExpression;
use super::expression_wrapper;
use crate::options::FormatOptions;
use crate::template::literal_lines::Representation;
use crate::template::literal_spans::ValueLines;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_formatter::{QuoteStyle, format_program, parse_for_format};
use oxc_formatter_core::LineWidth;
use oxc_span::SourceType;
use unicode_width::UnicodeWidthStr;
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

fn strip_generated_group_indent(expression: &str, options: &FormatOptions) -> String {
    let expression = trim_expression(expression);
    let Some((first, rest)) = expression.split_once('\n') else {
        return expression.to_compact_string();
    };
    let indent = options.indent_string();
    let mut unindented = String::with_capacity(expression.len());
    unindented.push_str(first);
    let mut literal = ValueLines::new(expression, Representation::JavaScript);
    literal.advance_line(first);
    for line in rest.split('\n') {
        unindented.push('\n');
        if literal.line_is_raw() {
            unindented.push_str(line);
        } else {
            unindented.push_str(line.strip_prefix(indent.as_str()).unwrap_or(line));
        }
        literal.advance_line(line);
    }
    unindented
}

#[inline]
fn value_starts_on_next_line(value: &str) -> bool {
    for byte in value.bytes() {
        if byte == b'\n' {
            return true;
        }
        if !byte.is_ascii_whitespace() {
            // Non-ASCII may be Unicode whitespace. Keep the original Unicode
            // predicate off the ordinary inline ASCII path.
            return !byte.is_ascii()
                && value.split_once('\n').is_some_and(|(first_line, _)| {
                    first_line.trim_end_matches('\r').trim().is_empty()
                });
        }
    }
    false
}

#[cold]
#[inline(never)]
fn unicode_attribute_value_overflows(name: &str, trimmed: &str, available_width: usize) -> bool {
    name.width() + trimmed.width() > available_width
}

#[inline]
fn value_uses_standalone_layout(
    name: &str,
    value: &str,
    trimmed: &str,
    options: &FormatOptions,
    attribute_depth: usize,
) -> bool {
    if value_starts_on_next_line(value) {
        return true;
    }

    let fixed_width = (attribute_depth + 1) * options.tab_width as usize + 3;
    let available_width = (options.print_width as usize).saturating_sub(fixed_width);
    let byte_width = name.len() + trimmed.len();
    if byte_width <= available_width || value.contains(['\r', '\n']) {
        return false;
    }
    if fixed_width >= options.print_width as usize {
        return true;
    }
    unicode_attribute_value_overflows(name, trimmed, available_width)
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
    attribute_name: &str,
    attribute_value: &str,
    trimmed_attribute_value: &str,
) -> Option<FormattedExpression> {
    format_expression_with_layout(
        expr,
        options,
        Some(QuoteStyle::Single),
        Some(attribute_depth),
        Some((attribute_name, attribute_value, trimmed_attribute_value)),
    )
}

pub(crate) fn format_js_expression_with_quote_style(
    expr: &str,
    options: &FormatOptions,
    quote_style: Option<QuoteStyle>,
) -> Option<FormattedExpression> {
    format_expression_with_layout(expr, options, quote_style, None, None)
}

fn format_expression_with_layout(
    expr: &str,
    options: &FormatOptions,
    quote_style: Option<QuoteStyle>,
    attribute_depth: Option<usize>,
    attribute_layout: Option<(&str, &str, &str)>,
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
            && (matches!(&unary.argument, Expression::CallExpression(_))
                || (matches!(
                    &unary.argument,
                    Expression::BinaryExpression(_)
                        | Expression::LogicalExpression(_)
                        | Expression::StaticMemberExpression(_)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::PrivateFieldExpression(_)
                        | Expression::ChainExpression(_)
                ) && attribute_layout.is_some_and(|(name, value, trimmed)| {
                    value_uses_standalone_layout(name, value, trimmed, options, depth)
                })))
        {
            // Retained direct calls already consume the attribute-value budget.
            // Apply that budget to the other expression shapes when the value
            // is authored on its own line or its inline attribute would
            // overflow onto one. Formatting the wrapper program would otherwise
            // measure that standalone value from column 0 and let it overflow
            // by its full indentation.
            let value_indent = (depth + 2) * options.tab_width as usize;
            let width = options
                .print_width
                .saturating_sub(value_indent as u32)
                .max(1);
            oxc_options.line_width = LineWidth::try_from(width as u16).unwrap_or_default();
            let code =
                oxc_formatter::format_unary_argument(oxc_allocator, &parsed.program, oxc_options)?
                    .print()
                    .ok()?
                    .into_code();
            let code = trim_expression(&code);
            let inner = expression_wrapper::unwrap_argument(code, &parsed.program, trimmed)?;
            let removed_generated_group = inner.text.len() != code.len();
            let code = if removed_generated_group {
                strip_generated_group_indent(inner.text, options)
            } else {
                trim_expression(inner.text).to_compact_string()
            };
            let owns_attribute_lines = code.contains(['\r', '\n']);
            return Some(FormattedExpression {
                code,
                retained_bare_sequence: inner.retained_bare_sequence,
                owns_attribute_lines,
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
