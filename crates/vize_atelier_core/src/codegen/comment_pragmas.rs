//! Template comments used only to control diagnostics.

use crate::{CommentNode, TemplateChildNode};

#[inline]
pub(crate) fn is_directive_comment(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Comment(comment) if is_pragma_comment(comment))
}

#[inline]
pub(crate) fn is_pragma_comment(comment: &CommentNode<'_>) -> bool {
    if comment.directive.is_some() {
        return true;
    }
    matches!(
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
