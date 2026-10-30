use super::super::{check_pack, native_fixture_runtime_disabled};

#[test]
fn defaulted_required_props_stay_optional_for_the_parent() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "required-default-props",
        "defaulted_required_props_stay_optional_for_the_parent",
        "34b2a8f60f2a73df5736422761c6f5f08cd21fe4",
        None,
        &[
            "defaulted-required-props-accept-omission-and-undefined",
            "required-props-without-default-stay-required",
        ],
    );
}
