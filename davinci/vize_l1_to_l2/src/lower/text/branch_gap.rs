use super::condense::{beside_comment, condense_internal, is_vue_ws};
use vize_l0::Allocator;
use vize_l1::markup::SurfaceChild;

/// The parse-time text view of a whitespace-only conditional gap, for authored
/// surface consumers. Compiler regions drop these gaps regardless of this view.
pub fn branch_gap_text<'a>(
    allocator: &'a Allocator,
    text: &'a str,
    in_pre: bool,
) -> Option<&'a str> {
    if in_pre {
        Some(text)
    } else if text.chars().all(is_vue_ws) {
        (!text.contains(['\n', '\r'])).then_some(" ")
    } else {
        Some(condense_internal(allocator, text).unwrap_or(text))
    }
}

/// The authored conditional-gap view with its actual sibling context. Retained
/// comments affect condense whitespace; pre and meaningful text are not removed.
/// `children` must be the parser's unmodified maximal text tokens, as retained by
/// authored-surface consumers. Synthetic split/merge trees use the compiler's
/// whole-group whitespace planner instead.
pub fn branch_gap_text_at<'a>(
    allocator: &'a Allocator,
    children: &[SurfaceChild<'a>],
    index: usize,
    in_pre: bool,
) -> Option<&'a str> {
    let SurfaceChild::Text(token) = children.get(index)? else {
        return None;
    };
    let prev = index.checked_sub(1).and_then(|before| children.get(before));
    let next = index.checked_add(1).and_then(|after| children.get(after));
    if !in_pre && token.text.chars().all(is_vue_ws) && beside_comment(prev, next) {
        None
    } else {
        branch_gap_text(allocator, token.text, in_pre)
    }
}

#[cfg(test)]
mod tests {
    use super::branch_gap_text_at;
    use vize_l0::Allocator;
    use vize_l1::markup::SurfaceChild;

    #[test]
    fn authored_gaps_share_comment_condense_without_erasing_real_text_or_pre() {
        for (source, in_pre, expected) in [
            ("<!--a-->  <i></i>", false, None),
            ("<i></i>  <!--a-->", false, None),
            ("<!--a-->  <!--b-->", false, None),
            ("<!--a-->  <i></i>", true, Some("  ")),
            ("<i></i>  <b></b>", false, Some(" ")),
            ("<i></i>\n<b></b>", false, None),
            ("<!--a-->\u{a0}<i></i>", false, Some("\u{a0}")),
            ("<!--a--> tail <i></i>", false, Some(" tail ")),
        ] {
            let arena = Allocator::new();
            let (tree, errors) = vize_l1::markup::parse(&arena, source);
            assert!(errors.is_empty(), "{source}: {errors:?}");
            assert!(
                !tree
                    .children
                    .windows(2)
                    .any(|pair| matches!(pair, [SurfaceChild::Text(_), SurfaceChild::Text(_)])),
                "the parsed-surface contract requires maximal text tokens: {source}",
            );
            let index = tree
                .children
                .iter()
                .position(|child| matches!(child, SurfaceChild::Text(_)))
                .unwrap();
            assert_eq!(
                branch_gap_text_at(&arena, &tree.children, index, in_pre),
                expected,
                "{source}"
            );
        }
    }
}
