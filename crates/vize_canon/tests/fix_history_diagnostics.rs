//! Diagnostic contracts for historical generator fixes, using the actual
//! production batch checker. Unavailable runtimes and helper errors fail.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

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
    assert_generated_check_ordinal_normalization_preserves_markers_without_digits();
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
fn template_expression_syntax_preserves_complete_authored_diagnostics() {
    check_pack(
        "content-mapper-template-expression-syntax",
        "template_expression_syntax_preserves_complete_authored_diagnostics",
        "a5ae4d313fa333c8bc7168d5da4acf3dc2bb8ee3",
        Some(10188),
        &["authored-errors"],
    );
}

#[test]
fn content_mapper_accepts_imported_const_enum_value_anchors() {
    check_pack(
        "content-mapper-const-enum-import",
        "content_mapper_accepts_imported_const_enum_value_anchors",
        "506529117573a96880435dd6466cfd7cf81fdc92",
        Some(10192),
        &["imported-const-enum-value"],
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

#[test]
fn v_for_source_callbacks_preserve_complete_original_diagnostics() {
    check_pack(
        "v-for-source-original",
        "v_for_source_callbacks_preserve_complete_original_diagnostics",
        "04aedfb8e8b41dfae89fe65256e2703fc3718cf3",
        Some(3818),
        &["complete-original-html-callback"],
    );
}

#[test]
fn declared_prop_names_preserve_complete_original_diagnostics() {
    check_pack(
        "declared-prop-names-original",
        "declared_prop_names_preserve_complete_original_diagnostics",
        "0f15c8b49d0630c1f2492c7ad8447ef4c9711534",
        Some(3863),
        &["complete-original-declared-prop-names"],
    );
}

#[test]
fn component_v_for_source_preserves_complete_original_diagnostics() {
    check_pack(
        "v-for-component-source-original",
        "component_v_for_source_preserves_complete_original_diagnostics",
        "04aedfb8e8b41dfae89fe65256e2703fc3718cf3",
        Some(3818),
        &["complete-original-component-callback"],
    );
}

#[test]
fn template_suppression_preserves_complete_original_diagnostics() {
    check_pack(
        "template-suppression-original",
        "template_suppression_preserves_complete_original_diagnostics",
        "e199979dfb32dfd8eb9ad32a2928fb010768f239",
        Some(6009),
        &["complete-original-template-suppression"],
    );
}

#[test]
fn native_click_events_preserve_pointer_event_diagnostics() {
    check_pack(
        "native-pointer-events-original",
        "native_click_events_preserve_pointer_event_diagnostics",
        "e7d8b1c82a8badec89ec9779838dfa50c1b9b790",
        Some(936),
        &["click-pointer-handler", "auxclick-pointer-handler"],
    );
}

#[test]
fn bounded_cache_maintenance_preserves_project_diagnostics() {
    check_pack(
        "cache-maintenance-9860",
        "bounded_cache_maintenance_preserves_project_diagnostics",
        "4714c0f264dedffaa81377df9545fbfa9514a64e",
        Some(9860),
        &["bounded-maintenance", "fresh-helper-handoff"],
    );
}

#[test]
fn workspace_package_alias_cache_preserves_sequential_scopes() {
    check_pack(
        "workspace-alias-cache-10063",
        "workspace_package_alias_cache_preserves_sequential_scopes",
        "7174c46d55825c155325e6314305e661d50ec95a",
        Some(10063),
        &["second-package-selection"],
    );
}

#[test]
fn nuxt_unknown_template_options_preserve_explicit_checks() {
    check_pack(
        "nuxt-unknown-template-options-10013",
        "nuxt_unknown_template_options_preserve_explicit_checks",
        "fb700551a3a3a69f8976efa6fe31bca43ebc19b4",
        Some(10013),
        &["generated-paths-explicit-unknown-checks"],
    );
}

#[test]
fn reactive_props_destructure_preserves_unused_diagnostics() {
    check_pack(
        "reactive-props-destructure-10008",
        "reactive_props_destructure_preserves_unused_diagnostics",
        "f276e4d59cdaf765fd7d3f14eb3d679f5b685f98",
        Some(10008),
        &["unused-binding", "used-rest-binding"],
    );
}

#[test]
fn inferred_project_tsconfig_preserves_owned_batch_inputs() {
    check_pack(
        "inferred-project-tsconfig-9718",
        "inferred_project_tsconfig_preserves_owned_batch_inputs",
        "f95557dadaab83dc0f7d4a7613bdb7ed4e59d323",
        Some(9718),
        &["owned-strict", "inferred-defaults"],
    );
}

#[test]
fn generic_fallthrough_spread_preserves_valid_values() {
    check_pack(
        "generic-fallthrough-spread-9956",
        "generic_fallthrough_spread_preserves_valid_values",
        "2b1e41a5b09005a97d24d1ff34f34c7507bbfb67",
        Some(9956),
        &["broad-index-valid"],
    );
}

#[test]
fn optional_fallthrough_spread_preserves_valid_values_and_rejects_invalid_values() {
    check_pack(
        "optional-fallthrough-spread-10170",
        "optional_fallthrough_spread_preserves_valid_values_and_rejects_invalid_values",
        "8300867552287abb14f4fbb7471c705aea72bbdf",
        Some(10170),
        &["optional-valid-invalid-control"],
    );
}

#[test]
fn misskey_generated_inference_probes_preserve_authored_diagnostics() {
    check_pack(
        "misskey-generated-inference-9516",
        "misskey_generated_inference_probes_preserve_authored_diagnostics",
        "cf6c83ab16fb7f20f84f0d51112090bbce89c74a",
        Some(9516),
        &["inference-probes-valid", "authored-unknown-prop-control"],
    );
}

#[test]
fn named_prop_before_spread_reports_once_with_or_without_listener() {
    check_pack(
        "component-v-bind-duplicate-diagnostic-10069",
        "named_prop_before_spread_reports_once_with_or_without_listener",
        "7174c46d55825c155325e6314305e661d50ec95a",
        Some(10069),
        &["wrong-prop-with-spread-and-listener"],
    );
}

#[test]
fn authored_ts_extensions_report_ts5097_by_default() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "allow-importing-ts-extensions-9890",
        "authored_ts_extensions_report_ts5097_by_default",
        "f1b07174a4a86ffc3433f31b8c78406d5bf5b760",
        Some(9890),
        &[
            "default-policy",
            "explicit-opt-in",
            "declaration-generated-routes",
            "generated-external-script-route",
        ],
    );
}

#[test]
fn nuxt_reference_graph_preserves_plugin_injection_import_types() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "nuxt-plugin-injection-types",
        "nuxt_reference_graph_preserves_plugin_injection_import_types",
        "2c22691f0f8303a28c425213c423e299f9ad5420",
        Some(9919),
        &["authored-dependency-identity"],
    );
}

#[test]
fn listener_keeps_class_style_and_spread_attribute_parity() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "listener-attribute-parity-9852",
        "listener_keeps_class_style_and_spread_attribute_parity",
        "11cb03950bc8d2a4e9cb8d164b0aaca79e302291",
        Some(9852),
        &["class-style-and-spread-listeners"],
    );
}

#[test]
fn dynamic_component_union_listener_preserves_clean_diagnostics() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "dynamic-component-union-listener-10026",
        "dynamic_component_union_listener_preserves_clean_diagnostics",
        "0aafae7e41f1ac381d07fa9f0154084cc1e6f211",
        Some(10026),
        &["native-element-union"],
    );
}

#[test]
fn native_inline_event_handler_is_contextually_typed() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "native-inline-event-handler-9835",
        "native_inline_event_handler_is_contextually_typed",
        "8ac6ce1668e9bede8d73411e63a247dbd6f8eb56",
        Some(9835),
        &["native-mousedown-arrow"],
    );
}

#[test]
fn implicit_art_component_reports_the_same_prop_errors_as_an_explicit_import() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "art-variant-implicit-component-9854",
        "implicit_art_component_reports_the_same_prop_errors_as_an_explicit_import",
        "f07065df35d25bc76f355e5f106c80dbefa922b5",
        Some(9854),
        &[
            "implicit-and-explicit-props",
            "implicit-subject-avoids-unknown-global-diagnostics",
        ],
    );
}

#[test]
fn runtime_props_reports_10100_10101_retain_expected_types() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "runtime-props-report-10100-10101",
        "runtime_props_reports_10100_10101_retain_expected_types",
        "5dfe002a9467d152c8545693df33def3aa5f3bf7",
        None,
        &["imported-helper-spread", "object-factory-assertion"],
    );
}

#[test]
fn imported_sfc_h_calls_check_missing_and_wrong_props() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "component-render-h-props-9855",
        "imported_sfc_h_calls_check_missing_and_wrong_props",
        "f15ca234a4bdd73bde9caea07e652f3a345eb342",
        Some(9855),
        &["missing-and-wrong-props"],
    );
}

#[test]
fn component_listener_aliases_stay_inside_guard_scope() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "component-listener-alias-scope-10112",
        "component_listener_aliases_stay_inside_guard_scope",
        "7cefe3a00e6184de1e121d39b0e153f08a428aa6",
        Some(10112),
        &["sibling-guarded-listeners"],
    );
}

#[test]
fn content_mapper_isolates_duplicate_class_and_style_attribute_probes() {
    check_pack(
        "content-mapper-duplicate-attributes-10184",
        "content_mapper_isolates_duplicate_class_and_style_attribute_probes",
        "0038b601e41dccb6f037aa68c3fc0207ea635e3b",
        Some(10184),
        &["listener-and-template-ref-class-style"],
    );
}

#[test]
fn content_mapper_diagnostics_keep_single_public_owners() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "content-mapper-diagnostics-10187",
        "content_mapper_diagnostics_keep_single_public_owners",
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
        Some(10187),
        &["duplicate-and-missing-props"],
    );
}

fn native_fixture_runtime_disabled() -> bool {
    std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some()
}

#[test]
fn dynamic_directive_expressions_preserve_typed_slot_payloads() {
    check_pack(
        "dynamic-directive-expression-mapping-10182",
        "dynamic_directive_expressions_preserve_typed_slot_payloads",
        "f305bcc8c81b8f6009dabb5a3a61bb838c755701",
        None,
        &["dynamic-slot-and-prop"],
    );
}

#[test]
fn dynamic_template_refs_preserve_batch_acceptance() {
    check_pack(
        "content-mapper-dynamic-template-ref-10198",
        "dynamic_template_refs_preserve_batch_acceptance",
        "b7fb3a9aefd31002b8b2d449695a9e14d657654a",
        Some(10198),
        &["component-typed-and-member-values"],
    );
}

#[test]
fn dynamic_template_ref_unions_preserve_batch_acceptance() {
    check_pack(
        "content-mapper-dynamic-template-ref-union-10251",
        "dynamic_template_ref_unions_preserve_batch_acceptance",
        "6817c853ca1740fd1070c0ec462c0d457f339c2c",
        Some(10251),
        &["conditional-component-and-native-targets"],
    );
}

#[test]
fn dynamic_template_ref_union_registry_stays_in_scope() {
    check_pack(
        "content-mapper-dynamic-template-ref-scope-10251",
        "dynamic_template_ref_union_registry_stays_in_scope",
        "8d83800c727eef953852383d46c7324ccdda53b6",
        Some(10251),
        &["conditional-target-stays-in-registry-scope"],
    );
}

#[test]
fn authored_props_binding_preserves_batch_acceptance() {
    check_pack(
        "content-mapper-props-binding-10196",
        "authored_props_binding_preserves_batch_acceptance",
        "232f8f514e0ea55a7aeef08bc883b46e0608d12f",
        Some(10196),
        &["authored-props-binding-with-unassigned-define-props"],
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
    assert_eq!(
        pack.historical_issues.as_deref(),
        (name == "runtime-props-report-10100-10101").then_some(&[10100, 10101][..]),
        "every historical issue must retain exact fixture custody"
    );
    assert_eq!(pack.regression_commit, regression);
    assert_eq!(
        pack.source_revision,
        match name {
            "typed-import-meta"
            | "slot-outlet-key"
            | "options-api-any-instance"
            | "authored-unused-symbols"
            | "split-script-original"
            | "v-for-source-original"
            | "v-for-component-source-original"
            | "declared-prop-names-original"
            | "template-suppression-original"
            | "native-pointer-events-original"
            | "cache-maintenance-9860"
            | "workspace-alias-cache-10063"
            | "nuxt-unknown-template-options-10013"
            | "reactive-props-destructure-10008"
            | "generic-fallthrough-spread-9956"
            | "optional-fallthrough-spread-10170"
            | "misskey-generated-inference-9516"
            | "component-v-bind-duplicate-diagnostic-10069"
            | "inferred-project-tsconfig-9718"
            | "allow-importing-ts-extensions-9890"
            | "nuxt-plugin-injection-types"
            | "listener-attribute-parity-9852"
            | "dynamic-component-union-listener-10026"
            | "native-inline-event-handler-9835"
            | "art-variant-implicit-component-9854"
            | "runtime-props-report-10100-10101"
            | "component-render-h-props-9855"
            | "component-listener-alias-scope-10112"
            | "content-mapper-duplicate-attributes-10184"
            | "dynamic-directive-expression-mapping-10182"
            | "content-mapper-template-expression-syntax"
            | "content-mapper-const-enum-import"
            | "content-mapper-diagnostics-10187"
            | "content-mapper-dynamic-template-ref-10198"
            | "content-mapper-dynamic-template-ref-union-10251"
            | "content-mapper-dynamic-template-ref-scope-10251"
            | "content-mapper-props-binding-10196" => regression,
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
        if name == "workspace-alias-cache-10063" {
            for (source, destination) in [
                ("packages/b", "packages/a/node_modules/@demo/b"),
                ("packages/c", "packages/b/node_modules/@demo/c"),
            ] {
                let destination = project_root.join(destination);
                std::fs::create_dir_all(
                    destination
                        .parent()
                        .expect("a workspace link must have a parent"),
                )
                .expect("workspace link parents must be created");
                link_vue(&project_root.join(source), &destination);
            }
            let mut first = BatchTypeChecker::with_options(
                &project_root,
                BatchTypeCheckerOptions {
                    tsconfig_path: Some(project_root.join("tsconfig.b.json")),
                    ..Default::default()
                },
            )
            .expect("the first package selection must start");
            first
                .scan_project()
                .expect("the first package selection must scan");
            let first = first
                .check_project()
                .expect("the first package selection must finish");
            // #10063 is a hard materialization error before diagnostics. Both
            // orderings must complete and preserve the empty authored result;
            // the complete diagnostic contract is compared again below.
            assert!(first.diagnostics.is_empty());
            assert!(first.diagnostics.is_empty());

            let mut second = BatchTypeChecker::with_options(
                &project_root,
                BatchTypeCheckerOptions {
                    tsconfig_path: Some(project_root.join("tsconfig.a.json")),
                    ..Default::default()
                },
            )
            .expect("the second package selection must start");
            second
                .scan_project()
                .expect("the second package selection must scan");
            let second = second
                .check_project()
                .expect("the second package selection must finish");
            assert!(second.diagnostics.is_empty());
            assert!(second.diagnostics.is_empty());
        }
        let options = BatchTypeCheckerOptions {
            tsconfig_path: (name == "workspace-alias-cache-10063")
                .then(|| project_root.join("tsconfig.a.json")),
            ..Default::default()
        };
        let mut checker =
            BatchTypeChecker::with_options(&project_root, options).expect("the checker must start");
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
        let (compared_actual, compared_expected) = if name == "required-props-edges" {
            (
                normalize_generated_check_ordinals(&actual),
                normalize_generated_check_ordinals(&case.diagnostics),
            )
        } else {
            (actual.clone(), case.diagnostics.clone())
        };
        assert_eq!(
            compared_actual, compared_expected,
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

fn normalize_generated_check_ordinals(diagnostics: &[Diagnostic]) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .cloned()
        .map(|mut diagnostic| {
            let mut message = diagnostic.message.as_str();
            let mut normalized = std::string::String::with_capacity(message.len());
            while let Some(at) = message.find("_CheckProps_") {
                let after_marker = at + "_CheckProps_".len();
                normalized.push_str(&message[..after_marker]);
                let digit_count = message[after_marker..]
                    .bytes()
                    .take_while(u8::is_ascii_digit)
                    .count();
                if digit_count > 0 {
                    normalized.push('#');
                }
                message = &message[after_marker + digit_count..];
            }
            normalized.push_str(message);
            diagnostic.message = normalized;
            diagnostic
        })
        .collect()
}

fn assert_generated_check_ordinal_normalization_preserves_markers_without_digits() {
    let diagnostics = [Diagnostic {
        file: String::from("src/App.vue"),
        line: 1,
        column: 1,
        severity: 1,
        code: Some(2322),
        message: String::from("_CheckProps_ stays distinct from _CheckProps_12"),
    }];

    let normalized = normalize_generated_check_ordinals(&diagnostics);

    assert_eq!(
        normalized[0].message,
        "_CheckProps_ stays distinct from _CheckProps_#"
    );
}
