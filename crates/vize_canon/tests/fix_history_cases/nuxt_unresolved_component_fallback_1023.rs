use crate::support::fix_history_runner::check_pack;

#[test]
fn unresolved_component_props_use_ambient_global_components() {
    check_pack(
        "nuxt-unresolved-component-fallback-1023",
        "fix_history_cases::nuxt_unresolved_component_fallback_1023::unresolved_component_props_use_ambient_global_components",
        "2160612c4b9dbe2dca53ba99c6bea154bbe95c82",
        Some(1023),
        &[
            "unresolved-component-prop-unchecked",
            "nuxt-global-component-prop-matches",
            "nuxt-global-component-prop-mismatch",
        ],
    );
}
