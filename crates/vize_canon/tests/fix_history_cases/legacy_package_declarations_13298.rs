use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_package_resolution_consumes_generated_declarations() {
    check_pack(
        "legacy-package-declarations-13298",
        "fix_history_cases::legacy_package_declarations_13298::legacy_package_resolution_consumes_generated_declarations",
        "65a280aca9684d4577ad04ef75da8da05ad235ff",
        Some(13298),
        &[
            "generated-declaration-valid",
            "generated-declaration-invalid-consumer",
        ],
    );
}
