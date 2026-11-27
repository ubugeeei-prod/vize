//! Nesting at and past the limits of the template formatter: open elements
//! it indents, and expressions the shared parser guard admits.

use super::formatter::nesting::{MAX_ELEMENT_NESTING_DEPTH, NESTING_DEPTH_ERROR};
use super::{format_template_content, format_template_content_preserving_text};
use crate::{FormatError, FormatOptions, VueVersion};
use vize_l0::expression_guard::MAX_EXPRESSION_NESTING_DEPTH;

/// `depth` open elements around one self-closing leaf, one tag per line.
///
/// Tags authored on one line stay adjacent, so the line breaks are what lets
/// the formatter indent.
fn nested_around_leaf(depth: usize) -> String {
    "<a>\n".repeat(depth) + "<b />\n" + &"</a>\n".repeat(depth)
}

/// The layout of [`nested_around_leaf`]: one line per tag, indented once per
/// open element.
fn laid_out_around_leaf(depth: usize) -> String {
    let mut lines = Vec::with_capacity(depth * 2 + 1);
    for level in 0..depth {
        lines.push("  ".repeat(level) + "<a>");
    }
    lines.push("  ".repeat(depth) + "<b />");
    for level in (0..depth).rev() {
        lines.push("  ".repeat(level) + "</a>");
    }
    lines.join("\n")
}

/// `depth` open elements around one text node, one tag per line.
fn nested_around_text(depth: usize) -> String {
    "<a>\n".repeat(depth) + "x\n" + &"</a>\n".repeat(depth)
}

/// Where two texts first differ, with the bytes around that offset. Texts
/// with megabytes of indentation are compared through this instead of being
/// printed whole.
fn first_difference(actual: &str, expected: &str) -> Option<(usize, String, String)> {
    let offset = actual
        .bytes()
        .zip(expected.bytes())
        .position(|(left, right)| left != right)
        .or((actual.len() != expected.len()).then_some(actual.len().min(expected.len())))?;
    let window = |text: &str| {
        let bytes = text.as_bytes();
        let end = bytes.len().min(offset + 40);
        let start = offset.saturating_sub(40).min(end);
        String::from_utf8_lossy(bytes.get(start..end).unwrap_or_default()).into_owned()
    };
    Some((offset, window(actual), window(expected)))
}

#[track_caller]
fn assert_nesting_depth_error(result: Result<vize_l0::String, FormatError>) {
    let Err(FormatError::TemplateFormatError(message)) = result else {
        panic!("nesting past the limit must be a template format error");
    };
    assert_eq!(message.as_str(), NESTING_DEPTH_ERROR);
}

fn format(source: &str) -> vize_l0::String {
    format_template_content(source, &FormatOptions::default()).unwrap()
}

#[test]
fn nested_elements_are_indented_once_per_open_element() {
    let formatted = format(&nested_around_leaf(2));
    assert_eq!(formatted.as_str(), "<a>\n  <a>\n    <b />\n  </a>\n</a>");
    assert_eq!(formatted.as_str(), laid_out_around_leaf(2));
}

#[test]
fn nesting_at_the_limit_formats() {
    let formatted = format(&nested_around_leaf(MAX_ELEMENT_NESTING_DEPTH));
    let expected = laid_out_around_leaf(MAX_ELEMENT_NESTING_DEPTH);
    assert_eq!(first_difference(formatted.as_str(), &expected), None);
}

#[test]
fn nesting_past_the_limit_is_a_format_error() {
    let options = FormatOptions::default();
    for source in [
        nested_around_leaf(MAX_ELEMENT_NESTING_DEPTH + 1),
        nested_around_text(MAX_ELEMENT_NESTING_DEPTH + 1),
        // Unclosed and on one line, as a truncated or generated file may be.
        "<a>".repeat(30_000),
    ] {
        assert_nesting_depth_error(format_template_content(&source, &options));
    }
}

#[test]
fn sibling_elements_do_not_accumulate_depth() {
    let siblings = MAX_ELEMENT_NESTING_DEPTH + 1;
    let formatted = format(&"<a>x</a>\n".repeat(siblings));
    let expected = vec!["<a>x</a>"; siblings].join("\n");
    assert_eq!(first_difference(formatted.as_str(), &expected), None);
}

#[test]
fn text_preserving_layout_has_the_same_limit() {
    let options = FormatOptions::default();
    // Plain tags and the text between them are already in preserved form.
    let at_limit = nested_around_text(MAX_ELEMENT_NESTING_DEPTH);
    let formatted =
        format_template_content_preserving_text(&at_limit, &options, VueVersion::V3).unwrap();
    assert_eq!(first_difference(formatted.as_str(), &at_limit), None);
    assert_nesting_depth_error(format_template_content_preserving_text(
        &nested_around_text(MAX_ELEMENT_NESTING_DEPTH + 1),
        &options,
        VueVersion::V3,
    ));
}

/// `inner` inside `depth` pairs of parentheses.
fn parenthesized(depth: usize, inner: &str) -> String {
    "(".repeat(depth) + inner + &")".repeat(depth)
}

fn interpolation(expression: &str) -> String {
    format!("<div>{{{{ {expression} }}}}</div>")
}

fn bound_attribute(expression: &str) -> String {
    format!("<div :a=\"{expression}\"></div>")
}

#[test]
fn expression_nested_to_the_guard_limit_formats() {
    // A formatted expression loses its redundant parentheses.
    let expression = parenthesized(MAX_EXPRESSION_NESTING_DEPTH, "a+b");
    assert_eq!(
        format(&interpolation(&expression)).as_str(),
        "<div>{{ a + b }}</div>"
    );
    assert_eq!(
        format(&bound_attribute(&expression)).as_str(),
        "<div :a=\"a + b\"></div>"
    );
}

#[test]
fn expression_nested_past_the_guard_limit_stays_as_authored() {
    let expression = parenthesized(MAX_EXPRESSION_NESTING_DEPTH + 1, "a+b");
    for source in [interpolation(&expression), bound_attribute(&expression)] {
        assert_eq!(format(&source).as_str(), source);
    }
}

#[test]
fn deeply_nested_interpolation_stays_as_authored() {
    let expression = parenthesized(20_000, "a");
    let source = format!("<div>\n  {{{{\n    {expression}\n  }}}}\n</div>");
    let formatted = format(&source);
    assert_eq!(first_difference(formatted.as_str(), &source), None);
}
