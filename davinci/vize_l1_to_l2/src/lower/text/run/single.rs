use vize_l0::String;
use vize_l1::SurfaceChild;
use vize_l2::op::Op;

use super::super::{TextAction, normalize_special_text};
use crate::lower::cx::Cx;

pub(super) fn lower_single_text<'a>(
    cx: &mut Cx<'a>,
    child: &SurfaceChild<'a>,
    action: TextAction<'a>,
    out: &mut vize_l0::Vec<'a, Op<'a>>,
) {
    match child {
        SurfaceChild::Text(token) => {
            let mut content = match action {
                TextAction::Content(content) => content,
                _ => token.text,
            };
            if let Some(normalized) = normalize_special_text(cx, content, cx.offset(token.text)) {
                content = cx.allocator.alloc_str(normalized.as_str());
            }
            if content != token.text {
                let span = cx.token_span(token);
                cx.record(
                    "condense.whitespace",
                    None,
                    token.text,
                    String::from(content),
                    span,
                );
            }
            crate::lower::leaf::lower_text(cx, token, content, out);
        }
        child => crate::lower::leaf::lower_leaf(cx, child, out),
    }
}
