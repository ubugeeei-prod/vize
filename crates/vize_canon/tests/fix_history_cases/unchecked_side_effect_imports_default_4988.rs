use crate::support::fix_history_runner::check_pack;

#[test]
fn side_effect_imports_follow_the_stable_tsc_default() {
    check_pack(
        "unchecked-side-effect-imports-default-4988",
        "fix_history_cases::unchecked_side_effect_imports_default_4988::side_effect_imports_follow_the_stable_tsc_default",
        "27dd5c8df22182f088f827bfcedd121f009cb118",
        Some(4988),
        &[
            "unspecified-option-leaves-side-effect-imports-unchecked",
            "explicit-false-leaves-side-effect-imports-unchecked",
            "explicit-opt-in-checks-side-effect-imports-invalid",
        ],
    );
}
