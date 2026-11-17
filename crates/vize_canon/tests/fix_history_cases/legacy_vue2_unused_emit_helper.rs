use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_no_unused_locals_skips_loose_emit_helper() {
    check_pack(
        "legacy-vue2-unused-emit-helper",
        "fix_history_cases::legacy_vue2_unused_emit_helper::legacy_vue2_no_unused_locals_skips_loose_emit_helper",
        "6fd843fb57857e6f1886ace03c0732cd76f18c98",
        None,
        &[
            "typed-emits-without-unused-helper",
            "user-unused-local-still-reported",
        ],
    );
}
