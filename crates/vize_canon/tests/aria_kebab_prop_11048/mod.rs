use super::{check_pack, native_fixture_runtime_disabled};

#[test]
fn declared_kebab_aria_props_preserve_batch_diagnostics() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "aria-kebab-prop-11048",
        "aria_kebab_prop_11048::declared_kebab_aria_props_preserve_batch_diagnostics",
        "65b38273c8713a724f42f0bb63a2aef7cc6daae4",
        Some(11048),
        &["declared-kebab-and-optional-valid-with-wrong-control"],
    );
}
