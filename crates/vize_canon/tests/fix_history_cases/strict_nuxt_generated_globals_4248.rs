use crate::support::fix_history_runner::check_pack;

#[test]
fn generated_nuxt_globals_are_authoritative() {
    check_pack(
        "strict-nuxt-generated-globals-4248",
        "fix_history_cases::strict_nuxt_generated_globals_4248::generated_nuxt_globals_are_authoritative",
        "9687c6e64ec1b731e99dcb77053cbd3d893f5ef2",
        Some(4248),
        &["index", "undeclared-global", "wrong-argument"],
    );
}
