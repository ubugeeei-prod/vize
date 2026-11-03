//! Annotations remain in L2 for analysis and are omitted only from rendered children.

use vize_l2::op::Op;

pub(super) fn is_annotation(op: &Op<'_>) -> bool {
    let Op::Comment(comment) = op else {
        return false;
    };
    vize_l0::directive::parse_vize_directive(comment.content, 1, 0).is_some()
        || matches!(
            comment
                .content
                .trim_ascii_start()
                .split_ascii_whitespace()
                .next(),
            Some(
                "eslint-disable"
                    | "eslint-disable-line"
                    | "eslint-disable-next-line"
                    | "eslint-enable"
                    | "oxlint-disable"
                    | "oxlint-disable-line"
                    | "oxlint-disable-next-line"
                    | "oxlint-enable"
            )
        )
}

/// Omitted children still consume their L2 page id, keeping later facts aligned.
pub(super) fn skip(cx: &mut super::EmitCx<'_>, op: &Op<'_>) -> bool {
    if is_annotation(op) {
        let _id = cx.walk.mint();
        true
    } else {
        false
    }
}
