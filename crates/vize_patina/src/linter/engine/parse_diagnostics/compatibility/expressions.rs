//! Rejected template expressions that Vue still compiles.
//!
//! L2 admits an expression only when nothing but whitespace or a closed block
//! comment follows it, because raw emitters append their own tokens on the
//! same line. Vue's compiler parses a value as `(value)`, so it also accepts a
//! `//` comment that a line break ends inside the value, and it renders an
//! empty interpolation as an empty string.

use vize_l0::Span;
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
    interpolation && text.trim().is_empty()
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
