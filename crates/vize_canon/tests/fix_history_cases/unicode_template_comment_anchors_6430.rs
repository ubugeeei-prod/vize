use crate::support::fix_history_runner::check_pack;

#[test]
fn template_comments_keep_authored_diagnostic_anchors() {
    check_pack(
        "unicode-template-comment-anchors-6430",
        "fix_history_cases::unicode_template_comment_anchors_6430::template_comments_keep_authored_diagnostic_anchors",
        "8d5a60647d265c898efca2c3597ff59071df9e18",
        Some(6430),
        &[
            "terminated-multibyte-leading-comments",
            "terminated-multibyte-comment-wrong-member",
            "ascii-comments-wrong-member",
            "unterminated-multibyte-comment-attribute",
            "unterminated-multibyte-comment-interpolation",
        ],
    );
}
