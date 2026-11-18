use crate::support::fix_history_runner::check_pack;

#[test]
fn unicode_script_comments_do_not_break_virtual_ts() {
    check_pack(
        "unicode-script-comment-virtual-ts",
        "fix_history_cases::unicode_script_comment_virtual_ts::unicode_script_comments_do_not_break_virtual_ts",
        "2ac861d0d2247fc4e715117468e4cf874b9ebd04",
        None,
        &[
            "unicode-comments-after-maybe-ref",
            "unicode-comments-historical-script",
            "unicode-comments-binding-mismatch",
        ],
    );
}
