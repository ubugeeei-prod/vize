use crate::support::fix_history_runner::check_pack;

#[test]
fn imported_runtime_and_mixin_props_preserve_requiredness() {
    check_pack(
        "runtime-mixin-required-props-3582",
        "fix_history_cases::runtime_mixin_required_props_3582::imported_runtime_and_mixin_props_preserve_requiredness",
        "b15fb9ebb37a173b1b50d91eef42d3625dd6b014",
        Some(3582),
        &[
            "required-runtime-and-mixin-props-bound",
            "required-runtime-and-mixin-props-missing",
            "inherited-mixin-prop-wrong-type",
        ],
    );
}
