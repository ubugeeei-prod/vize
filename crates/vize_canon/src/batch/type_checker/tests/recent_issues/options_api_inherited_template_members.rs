//! A Vue 3 Options API template member contributed by an imported `mixins:`
//! entry or an `extends:` base is declared from the authored instance's type,
//! so the authored expression is checked against it the way `vue-tsc` checks
//! `__VLS_ctx.<name>`. A name the instance lacks still reports exactly once,
//! with the instance wording.

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::{BatchTypeCheckerOptions, TypeChecker};
use vize_l0::{String, cstr};

const FILES: [(&str, &str); 6] = [
    (
        "src/base.ts",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/base.ts.txt"
        ),
    ),
    (
        "src/shared.ts",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/shared.ts.txt"
        ),
    ),
    (
        "src/Typed.vue",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/Typed.vue.txt"
        ),
    ),
    (
        "src/Invalid.vue",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/Invalid.vue.txt"
        ),
    ),
    (
        "src/Wrapped.vue",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/Wrapped.vue.txt"
        ),
    ),
    (
        "tsconfig.json",
        include_str!(
            "../../../../../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/tsconfig.json.txt"
        ),
    ),
];

/// Native mapped diagnostics for the pack shared with the differential runner:
/// every misuse of an inherited member reports once with the member's declared
/// type, valid uses stay silent, and an undeclared name keeps its single
/// instance report.
#[test]
fn inherited_template_members_are_checked_with_their_declared_types() {
    let Some(native) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "inherited template members need the native checker"
        );
        return;
    };
    let project_root = create_project_case("options-api-inherited-template-members", &FILES);
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        &project_root,
        BatchTypeCheckerOptions::default(),
        Some(&native),
    )
    .expect("batch checker should initialize");
    checker.enable_options_api();
    checker.scan_project().expect("project scan should succeed");
    let result = checker.check_project().expect("project check should run");
    let mut diagnostics: Vec<(String, Option<u32>, String)> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}: {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    with_portable_instance_wording(diagnostic.message.as_str())
                ),
            )
        })
        .collect();
    diagnostics.sort();
    if let Some(capture) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let capture =
            std::path::PathBuf::from(capture).join("options-api-inherited-template-members");
        std::fs::create_dir_all(&capture).unwrap();
        let packet = serde_json::json!({
            "scope": "synthetic Vue 3 Options API fixture with the configured native checker",
            "provider": native,
            "inputs": FILES,
            "mappedDiagnostics": diagnostics,
        });
        std::fs::write(
            capture.join("mapped-diagnostics.json"),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
    }
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        diagnostics,
        vec![
            (
                "src/Invalid.vue".into(),
                Some(2339),
                cstr!("2:107: Property 'missing' does not exist on type '{INSTANCE_WORDING}'.")
            ),
            (
                "src/Invalid.vue".into(),
                Some(2339),
                "2:32: Property 'toFixed' does not exist on type 'boolean'.".into()
            ),
            (
                "src/Invalid.vue".into(),
                Some(2339),
                "2:87: Property 'toUpperCase' does not exist on type 'number'.".into()
            ),
            (
                "src/Invalid.vue".into(),
                Some(2345),
                "2:20: Argument of type 'number' is not assignable to parameter of type 'string'."
                    .into()
            ),
            (
                "src/Invalid.vue".into(),
                Some(2551),
                "2:62: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"
                    .into()
            ),
            (
                "src/Wrapped.vue".into(),
                Some(2339),
                "2:34: Property 'toFixed' does not exist on type 'boolean'.".into()
            ),
        ]
    );
}

/// Placeholder for the authored instance type in the undeclared-name report.
const INSTANCE_WORDING: &str = "<authored instance>";

/// The undeclared-name report names the authored instance type, whose spelling
/// is Vue's (`CreateComponentPublicInstance<…>` on 3.4,
/// `CreateComponentPublicInstanceWithMixins<…>` on 3.6) and lists the `extends`
/// and `mixins` option types inside it. The report's shape, not Vue's spelling
/// of that type, is the contract, so only Vue's instance type text is replaced
/// by a placeholder; any other receiver wording (the test stub's
/// `ComponentPublicInstance<{}>`, `{}`, `never`) stays verbatim and fails the
/// comparison, and every other message is compared verbatim.
fn with_portable_instance_wording(message: &str) -> String {
    const PREFIX: &str = "Property 'missing' does not exist on type '";
    match message.strip_prefix(PREFIX) {
        Some(rest) if rest.starts_with("CreateComponentPublicInstance") && rest.ends_with("'.") => {
            cstr!("{PREFIX}{INSTANCE_WORDING}'.")
        }
        _ => String::from(message),
    }
}
