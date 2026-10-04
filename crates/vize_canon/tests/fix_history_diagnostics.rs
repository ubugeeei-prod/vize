//! Diagnostic contracts for historical generator fixes, using the actual
//! production batch checker. Unavailable runtimes and helper errors fail.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

mod support {
    pub(crate) mod fix_history_contract;
    pub(crate) mod fix_history_observation;
    pub(crate) mod fix_history_paths;
}
use support::fix_history_contract::{Diagnostic, Input, Pack};
use support::fix_history_observation as observation;
use support::fix_history_paths::{link_vue, safe_relative};

#[test]
fn inline_event_assignments_preserve_exact_diagnostics() {
    check_pack(
        "event-handler-narrowing",
        "inline_event_assignments_preserve_exact_diagnostics",
        "613ce3a5a31d22ec0dfd42feab25772e63fc54bf",
        Some(4996),
        &[
            "component-sibling-valid",
            "native-sibling-valid",
            "handler-assignment-invalid",
        ],
    );
}

#[test]
fn deferred_template_reads_preserve_script_diagnostics() {
    check_pack(
        "template-definite-assignment",
        "deferred_template_reads_preserve_script_diagnostics",
        "39bf60c0614c888ba82f962b944160ce839cf035",
        Some(4239),
        &["vue3-complete-project", "tsx-deferred-template"],
    );
}

#[test]
fn required_props_keep_exact_unicode_diagnostics() {
    check_pack(
        "required-props-edges",
        "required_props_keep_exact_unicode_diagnostics",
        "f3a26b0e30c98b135a9e897c3584526eb0a2b96d",
        Some(3581),
        &["complete-attribute-boundaries"],
    );
}

#[test]
fn unicode_reserved_props_preserve_value_diagnostics() {
    check_pack(
        "unicode-reserved-props",
        "unicode_reserved_props_preserve_value_diagnostics",
        "13e5ec2a9752c5d8d41a9510f54e7096a583bb78",
        None,
        &["literal-values-valid", "literal-values-invalid"],
    );
}

#[test]
fn reserved_prop_shapes_preserve_literal_and_member_diagnostics() {
    check_pack(
        "reserved-expression-shapes",
        "reserved_prop_shapes_preserve_literal_and_member_diagnostics",
        "27ae56ea668fc6c94653894ec29d75f8339a04c7",
        Some(923),
        &["valid-expression-shapes", "invalid-expression-shapes"],
    );
}

#[test]
fn component_event_tuples_preserve_all_argument_diagnostics() {
    check_pack(
        "component-event-tuples",
        "component_event_tuples_preserve_all_argument_diagnostics",
        "e65154535ad5fb8645330848ea61f3fa8a65575b",
        Some(3085),
        &[
            "unresolved-valid",
            "resolved-valid",
            "inline-tuple-valid",
            "resolved-first-invalid",
            "native-extra-required-invalid",
            "resolved-second-invalid",
            "inline-second-invalid",
        ],
    );
}

#[test]
fn typed_import_meta_preserves_exact_authored_diagnostics() {
    check_pack(
        "typed-import-meta",
        "typed_import_meta_preserves_exact_authored_diagnostics",
        "6e0f763bd0f5e986036244c2c17c9b658b0596bd",
        None,
        &["project-import-meta-types"],
    );
}

#[test]
fn slot_outlet_keys_preserve_complete_project_diagnostics() {
    check_pack(
        "slot-outlet-key",
        "slot_outlet_keys_preserve_complete_project_diagnostics",
        "c6e43ca98cbffe099a6ef323006139d796d62208",
        None,
        &["declared-and-inferred-key-payload"],
    );
}

#[test]
fn options_api_any_instance_preserves_complete_original_diagnostics() {
    check_pack(
        "options-api-any-instance",
        "options_api_any_instance_preserves_complete_original_diagnostics",
        "35bdad84760e6251edd52bb2e3e52701974a9d63",
        Some(6680),
        &["typed-props-control", "loose-and-concrete-mixins"],
    );
}

#[test]
fn authored_unused_symbols_preserve_complete_original_diagnostics() {
    check_pack(
        "authored-unused-symbols",
        "authored_unused_symbols_preserve_complete_original_diagnostics",
        "cd7156d28386e072953476fdbc354a963758dc89",
        Some(1271),
        &[
            "original-locals",
            "inherited-locals",
            "disabled-locals",
            "parameters-only",
            "default-locals",
            "parameters-disabled",
        ],
    );
}

#[test]
fn split_script_setup_preserves_complete_original_diagnostics() {
    check_pack(
        "split-script-original",
        "split_script_setup_preserves_complete_original_diagnostics",
        "b3c2933be5afbdb5f2df07da093f40bc7ce72187",
        Some(3783),
        &["complete-original-split-script"],
    );
}

fn check_pack(
    name: &str,
    test: &str,
    regression: &str,
    historical_issue: Option<u32>,
    case_ids: &[&str],
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = root
        .join("tests/_fixtures/differential/typechecker")
        .join(name);
    let fixture_pack =
        std::fs::read(fixtures.join("cases.json")).expect("the fixture pack must exist");
    let pack: Pack = serde_json::from_slice(&fixture_pack)
        .expect("the fixture pack must have the expected schema");
    assert_eq!(pack.version, 1);
    assert_eq!(pack.issue, 6879);
    assert_eq!(pack.historical_issue, historical_issue);
    assert_eq!(pack.regression_commit, regression);
    assert_eq!(
        pack.source_revision,
        match name {
            "typed-import-meta"
            | "slot-outlet-key"
            | "options-api-any-instance"
            | "authored-unused-symbols"
            | "split-script-original" => regression,
            _ => "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943",
        }
    );
    assert_eq!(pack.diagnostic_contract.position_base, 1);
    assert_eq!(pack.diagnostic_contract.positions, "authored UTF16 start");
    assert_eq!(
        pack.diagnostic_contract.comparison,
        "all production diagnostics in returned order"
    );
    assert_eq!(
        pack.diagnostic_contract.missing_fields,
        ["end", "relatedInformation", "raw backend diagnostics"]
    );
    assert_eq!(pack.diagnostic_contract.native, "unsupported");
    assert_eq!(pack.diagnostic_contract.required_tier, "T1");
    assert_eq!(
        pack.checker_options
            .as_ref()
            .map(|options| options.options_api),
        (name == "options-api-any-instance").then_some(true),
        "the original explicit checker options must be retained"
    );
    assert_eq!(
        pack.cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        case_ids,
        "missing, duplicated or reordered cases must not pass"
    );
    let vue = std::fs::canonicalize(root.join("tests/node_modules/vue"))
        .expect("the pinned real Vue package must be installed");
    assert_eq!(
        pack.project_options,
        (name == "authored-unused-symbols").then(|| serde_json::json!({"vuePackage":"absent"}))
    );
    let mut capture = observation::Capture::from_environment();
    for case in pack.cases {
        let project = tempfile::tempdir().expect("a project must be created");
        let project_root =
            std::fs::canonicalize(project.path()).expect("project identity must resolve");
        if pack.project_options.is_none() {
            std::fs::create_dir(project_root.join("node_modules"))
                .expect("node_modules must be created");
            link_vue(&vue, &project_root.join("node_modules/vue"));
        }
        for input in &case.inputs {
            let file = safe_relative(&input.file);
            let source = safe_relative(&input.source);
            let destination = project_root.join(file);
            std::fs::create_dir_all(destination.parent().expect("an input must have a parent"))
                .expect("input parents must be created");
            std::fs::copy(fixtures.join(source), destination)
                .expect("exact fixture bytes must copy");
        }
        if pack.project_options.is_some() {
            assert!(!project_root.join("node_modules").exists());
        }
        let mut checker = BatchTypeChecker::new(&project_root).expect("the checker must start");
        if pack
            .checker_options
            .as_ref()
            .is_some_and(|options| options.options_api)
        {
            checker.enable_options_api();
        }
        checker.scan_project().expect("the project must scan");
        let result = checker
            .check_project()
            .expect("the production checker must finish");
        let actual: Vec<Diagnostic> = result
            .diagnostics
            .iter()
            .map(|diagnostic| Diagnostic {
                file: diagnostic
                    .file
                    .strip_prefix(&project_root)
                    .expect("every diagnostic must belong to this exact project")
                    .to_string_lossy()
                    .replace('\\', "/"),
                line: diagnostic.line + 1,
                column: diagnostic.column + 1,
                severity: diagnostic.severity,
                code: diagnostic.code,
                message: String::from(diagnostic.message.as_str()),
            })
            .collect();
        assert_eq!(
            actual, case.diagnostics,
            "complete diagnostic list for {}",
            case.id
        );
        if let Some(capture) = &mut capture {
            capture.record(case.id, &case.inputs, &project_root, actual, &result);
        }
    }
    if let Some(capture) = capture {
        capture.finish(name, test, &fixture_pack);
    }
}
