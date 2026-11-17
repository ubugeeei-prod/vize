use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_this_includes_typed_mixin_and_extends_members() {
    check_pack(
        "legacy-options-inherited-members-3627",
        "fix_history_cases::legacy_options_inherited_members_3627::legacy_this_includes_typed_mixin_and_extends_members",
        "b2f8c3b2a937f743629e16fe40f92bae2730a42c",
        Some(3627),
        &["typed-mixins-and-extends-resolve", "mixin-missing-member"],
    );
}
