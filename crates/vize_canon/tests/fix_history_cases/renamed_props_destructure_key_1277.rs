use crate::support::fix_history_runner::check_pack;

#[test]
fn renamed_props_destructure_keys_resolve_in_template() {
    check_pack(
        "renamed-props-destructure-key-1277",
        "fix_history_cases::renamed_props_destructure_key_1277::renamed_props_destructure_keys_resolve_in_template",
        "5c1ddef5af5c99711160a419899b1dff9f440012",
        Some(1277),
        &[
            "historical-original-and-renamed-keys",
            "renamed-local-only",
            "renamed-local-wrong-type",
            "original-key-wrong-member",
        ],
    );
}
