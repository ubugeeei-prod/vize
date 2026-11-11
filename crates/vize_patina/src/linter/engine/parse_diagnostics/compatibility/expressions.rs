//! Empty template interpolations that Vue renders as an empty string.

use vize_l0::Span;
use vize_l1::markup::{SurfaceChild, SurfaceTree};

pub(super) fn is_accepted_by_vue(tree: &SurfaceTree<'_>, span: Span) -> bool {
    tree.source
        .get(span.start as usize..span.end as usize)
        .is_some_and(|text| text.trim().is_empty())
        && in_interpolation(tree.source, &tree.children, span)
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
