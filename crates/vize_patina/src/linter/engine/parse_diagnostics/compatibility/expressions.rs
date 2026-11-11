//! Rejected template expressions that Vue still compiles.
//!
//! L2 admits an expression only when nothing but whitespace or a closed block
//! comment follows it, because raw emitters append their own tokens on the
//! same line. Vue's compiler parses a value as `(value)`, so it also accepts a
//! `//` comment that a line break ends inside the value, and it renders an
//! empty interpolation as an empty string.

use oxc_span::{GetSpan, SourceType};
use vize_l0::Span;
use vize_l0::expression_guard::expression_is_safe_to_parse;
use vize_l0::expression_guard::scan::line_comment_terminator;
use vize_l1::markup::{SurfaceChild, SurfaceTree};

pub(super) fn is_accepted_by_vue(tree: &SurfaceTree<'_>, span: Span) -> bool {
    let interpolation = in_interpolation(tree.source, &tree.children, span);
    // The span stops at the expression; an attribute value keeps the line
    // break after it, while Vue trims an interpolation before parsing it.
    let end = if interpolation {
        span.end as usize
    } else {
        let tail = tree.source.get(span.end as usize..).unwrap_or_default();
        span.end as usize + (tail.len() - tail.trim_start().len())
    };
    let Some(text) = tree.source.get(span.start as usize..end) else {
        return false;
    };
    let text = if interpolation {
        text.trim_matches([' ', '\t', '\n', '\r', '\u{0c}'])
    } else {
        text
    };
    if text.trim().is_empty() {
        return interpolation;
    }
    text.contains("//") && parses_with_terminated_line_comments(text)
}

fn parses_with_terminated_line_comments(text: &str) -> bool {
    if !expression_is_safe_to_parse(text) {
        return false;
    }
    let allocator = oxc_allocator::Allocator::default();
    let Ok(expression) =
        oxc_parser::Parser::new(&allocator, text, SourceType::ts()).parse_expression()
    else {
        return false;
    };
    text.get(expression.span().end as usize..)
        .is_some_and(is_wrapped_trailing_trivia)
}

/// Whitespace and comments that `(value)` still closes: a block comment, or a
/// line comment ended by a line break.
fn is_wrapped_trailing_trivia(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    let mut index = 0;
    loop {
        let Some(tail) = rest.get(index..) else {
            return false;
        };
        let trimmed = tail.trim_start();
        index += tail.len() - trimmed.len();
        if trimmed.is_empty() {
            return true;
        }
        if trimmed.starts_with("/*") {
            let Some(close) = trimmed.get(2..).and_then(|body| body.find("*/")) else {
                return false;
            };
            index += close + 4;
        } else if trimmed.starts_with("//") {
            let terminator = line_comment_terminator(bytes, index + 2);
            if terminator.is_empty() {
                return false;
            }
            index = terminator.end;
        } else {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_wrapped_trailing_trivia, parses_with_terminated_line_comments};

    #[test]
    fn line_comments_need_a_line_break_inside_the_value() {
        assert!(parses_with_terminated_line_comments(
            "\n  id // the element id\n"
        ));
        assert!(parses_with_terminated_line_comments(
            "id // a\r\n /* b */ // c\u{2028}"
        ));
        assert!(!parses_with_terminated_line_comments(
            "id // the element id"
        ));
        assert!(!parses_with_terminated_line_comments("a b // c\n"));
        assert!(is_wrapped_trailing_trivia(" /* // */ "));
        assert!(!is_wrapped_trailing_trivia(" /* open"));
    }
}

fn in_interpolation(source: &str, children: &[SurfaceChild<'_>], span: Span) -> bool {
    children.iter().any(|child| match child {
        SurfaceChild::Interpolation(interpolation) => {
            let start = (interpolation.content.text.as_ptr() as usize)
                .checked_sub(source.as_ptr() as usize)
                .unwrap_or(usize::MAX);
            let end = start.saturating_add(interpolation.content.text.len());
            start <= span.start as usize && span.end as usize <= end
        }
        SurfaceChild::Element(element) => vize_l0::recursion::ensure_sufficient_stack(|| {
            in_interpolation(source, &element.children, span)
        }),
        _ => false,
    })
}
