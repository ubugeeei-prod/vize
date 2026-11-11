//! HTML tokenizer errors the L1 tokenizer recovers from silently, reported
//! the way vue-eslint-parser reports them.

use vize_l0::{ErrorCode, Span};
use vize_l1::markup::{SurfaceChild, SurfaceTree};

/// Elements whose text the tokenizer reads as raw text or RCDATA, where a
/// `<` cannot open a tag.
const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style", "textarea", "title"];

/// `<` before a character that cannot start a tag name (`a < b`, `a <= b`),
/// and `<!-` that does not open a comment.
pub(super) fn tag_open_errors(tree: &SurfaceTree<'_>, push: &mut impl FnMut(ErrorCode, Span)) {
    visit(tree.source, &tree.children, false, push);
}

fn visit(
    source: &str,
    children: &[SurfaceChild<'_>],
    raw_text: bool,
    push: &mut impl FnMut(ErrorCode, Span),
) {
    for child in children {
        match child {
            SurfaceChild::Element(element) => {
                let tag = element.tag();
                let raw = RAW_TEXT_ELEMENTS
                    .iter()
                    .any(|name| tag.eq_ignore_ascii_case(name));
                vize_l0::recursion::ensure_sufficient_stack(|| {
                    visit(source, &element.children, raw, push);
                });
            }
            SurfaceChild::Text(token) if !raw_text => {
                let Some(start) = offset_in(source, token.text) else {
                    continue;
                };
                for (index, _) in token.text.match_indices('<') {
                    let at = start + index;
                    let starts_tag = source
                        .as_bytes()
                        .get(at + 1)
                        .is_none_or(|next| next.is_ascii_alphabetic() || b"!/?".contains(next));
                    if !starts_tag {
                        push(ErrorCode::InvalidFirstCharacterOfTagName, span(at, at + 1));
                    }
                }
            }
            SurfaceChild::Unexpected(token)
                if token.text.starts_with("<!-") && !token.text.starts_with("<!--") =>
            {
                if let Some(start) = offset_in(source, token.text) {
                    push(
                        ErrorCode::IncorrectlyOpenedComment,
                        span(start, start + token.text.len()),
                    );
                }
            }
            _ => {}
        }
    }
}

fn offset_in(source: &str, slice: &str) -> Option<usize> {
    (slice.as_ptr() as usize)
        .checked_sub(source.as_ptr() as usize)
        .filter(|offset| offset + slice.len() <= source.len())
}

fn span(start: usize, end: usize) -> Span {
    Span::new(start as u32, end as u32)
}
