//! Diagnostic contracts for historical generator fixes, using the actual
//! production batch checker. Unavailable runtimes and helper errors fail.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

mod support {
    pub(crate) mod fix_history_contract;
    pub(crate) mod fix_history_normalization;
    pub(crate) mod fix_history_observation;
    pub(crate) mod fix_history_paths;
    pub(crate) mod fix_history_reference_paths;
    pub(crate) mod fix_history_router_provider;
    pub(crate) mod fix_history_runner;
}
use support::fix_history_normalization::assert_generated_check_ordinal_normalization_preserves_markers_without_digits;
use support::fix_history_runner::check_pack;

#[test]
fn contextual_prop_anchors_preserve_authored_diagnostics() {
    check_pack(
        "contextual-prop-anchors-10661",
        "contextual_prop_anchors_preserve_authored_diagnostics",
        "da6d833de40c8a084022d0229beaeec25a92eb41",
        Some(10661),
        &[
            "original-from",
            "all-keyword-props",
            "authored-contextual-locals",
            "authored-prop-shadow-imports",
            "reactive-prop-destructure",
            "const-enum-import",
            "unicode-identities",
            "authored-ts2552-retained",
            "non-colliding-keyword-props",
            "authored-undefined-shadow",
        ],
    );
}

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
        &["authored-errors", "syntax-error-keeps-sfc-module"],
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
fn content_mapper_keeps_inferred_type_only_imports_out_of_value_anchors() {
    check_pack(
        "content-mapper-type-only-import-use",
        "content_mapper_keeps_inferred_type_only_imports_out_of_value_anchors",
        "fb89f41ffc518768e3053ccf9fe6b83bf24a1122",
        Some(10193),
        &["inferred-type-only-imports"],
    );
}

#[test]
fn content_mapper_unknown_options_preserve_requested_diagnostics() {
    check_pack(
        "content-mapper-unknown-options-10218",
        "content_mapper_unknown_options_preserve_requested_diagnostics",
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
        Some(10218),
        &["unknown-component-and-directive"],
    );
}

#[test]
fn valueless_directive_imports_preserve_upstream_diagnostic_parity() {
    check_pack(
        "valueless-directive-import-3140",
        "valueless_directive_imports_preserve_upstream_diagnostic_parity",
        "c3311a010ec5d5cea6669a2b84c4d02d272b1f45",
        Some(3140),
        &["imported-valueless-directive"],
    );
}

#[test]
fn plain_javascript_sfcs_follow_check_js_policy() {
    check_pack(
        "content-mapper-javascript-checkjs-10217",
        "plain_javascript_sfcs_follow_check_js_policy",
        "c9b9b5c368fceeed61c13813a617f7a8b5b8351f",
        Some(10217),
        &["plain-javascript-unchecked"],
    );
}

#[test]
fn filename_globals_preserve_clean_batch_diagnostics() {
    check_pack(
        "content-mapper-filename-globals-10190",
        "filename_globals_preserve_clean_batch_diagnostics",
        "55f388568ab900f5b0076d8c34280024ba988044",
        Some(10190),
        &["error-and-map-globals"],
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
        &[
            "click-pointer-handler",
            "auxclick-pointer-handler",
            "vue3-intrinsic-attribute",
            "vue3-target-intrinsic-attributes",
        ],
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
        "f305bcc8c81b8f6009dabb5a3a61bb838c755701",
        Some(10063),
        &[
            "second-package-selection",
            "repaired-private-package-identity",
        ],
    );
}

#[test]
fn workspace_package_import_aliases_preserve_clean_diagnostics() {
    check_pack(
        "workspace-package-import-alias-10175",
        "workspace_package_import_aliases_preserve_clean_diagnostics",
        "c935e00211b3d6ca271e53658f88156921a639a7",
        Some(10175),
        &["package-import-and-workspace-package"],
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
fn strict_component_attrs_infer_default_fallthrough_surfaces() {
    check_pack(
        "strict-fallthrough-default-7856",
        "strict_component_attrs_infer_default_fallthrough_surfaces",
        "d32790c869ac18c1e0e24e82a8876d796190133a",
        Some(7856),
        &["native-single-root-valid"],
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
fn required_prop_reports_once_with_or_without_listener() {
    check_pack(
        "component-listener-prop-duplicate-10397",
        "required_prop_reports_once_with_or_without_listener",
        "950b16bce8d2754e07d44f24bcd58d6a59a031e2",
        Some(10397),
        &["required-boolean-with-listener"],
    );
}

#[test]
fn vue3_native_expectations_preserve_strict_ownership() {
    check_pack(
        "native-expect-error-10413",
        "vue3_native_expectations_preserve_strict_ownership",
        "b005ad1279cc17a2da628e4d7459a0cb9f65dbe7",
        Some(10413),
        &["strict-owned-and-ordinary-permissive"],
    );
}

#[test]
fn generic_constructor_intersections_preserve_slot_payloads() {
    check_pack(
        "generic-constructor-slots-10416",
        "generic_constructor_intersections_preserve_slot_payloads",
        "036b1a34364c3568b8f19d4dad7479aadcff9860",
        Some(10416),
        &["props-infer-plain-and-covariant-slots"],
    );
}

#[test]
fn planned_generic_syntax_keeps_independent_body_errors_through_repairs() {
    check_pack(
        "planned-generic-syntax-route-10598",
        "planned_generic_syntax_keeps_independent_body_errors_through_repairs",
        "957827706dd439d9810a24f65b4a16c45f266b5e",
        Some(10598),
        &[
            "lf-valid-generic-body-error",
            "lf-broken-generic-body-error",
            "lf-repaired-generic-body-error",
            "lf-both-repaired",
            "crlf-valid-generic-body-error",
            "crlf-broken-generic-body-error",
            "crlf-repaired-generic-body-error",
            "crlf-both-repaired",
            "unicode-lf-valid-generic-body-error",
            "unicode-lf-broken-generic-body-error",
            "unicode-lf-repaired-generic-body-error",
            "unicode-lf-both-repaired",
            "unicode-crlf-valid-generic-body-error",
            "unicode-crlf-broken-generic-body-error",
            "unicode-crlf-repaired-generic-body-error",
            "unicode-crlf-both-repaired",
        ],
    );
}

#[test]
fn quoted_generic_parameters_preserve_complete_batch_diagnostics() {
    check_pack(
        "quoted-generic-parameters-10540",
        "quoted_generic_parameters_preserve_complete_batch_diagnostics",
        "711b56b2499b9136d8965a59c65713a2b78540f4",
        Some(10540),
        &[
            "literal-comma-valid",
            "numeric-prop-invalid",
            "repaired-literal-prop-valid",
        ],
    );
}

#[test]
fn content_mapper_global_components_preserve_batch_acceptance() {
    check_pack(
        "content-mapper-global-component-10185",
        "content_mapper_global_components_preserve_batch_acceptance",
        "10a9eaa743e3fb92b6372daa64c509bce2b204f1",
        Some(10185),
        &["unimported-framework-global-with-children"],
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
        &["sibling-guarded-listeners", "reopened-guard-block"],
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
fn content_mapper_musea_art_project_preserves_bound_component_diagnostics() {
    check_pack(
        "content-mapper-musea-art-10186",
        "content_mapper_musea_art_project_preserves_bound_component_diagnostics",
        "d295b324299c549063fab385b7f308c71ac014e0",
        Some(10186),
        &["bound-component-props"],
    );
}

#[test]
fn content_mapper_preserves_emit_with_nuxt_pattern_injections() {
    check_pack(
        "content-mapper-nuxt-emit-10195",
        "content_mapper_preserves_emit_with_nuxt_pattern_injections",
        "369f01b50d8fe6e3b0996cc9a90eecf422dbdd3b",
        Some(10195),
        &["nuxt-pattern-injection-attribute-listener"],
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

#[test]
fn pins_incomplete_setup_import_corpus() {
    check_pack(
        "incomplete-script-setup-recovery-10200",
        "pins_incomplete_setup_import_corpus",
        "4a555b0aae0a394fbe4cedc08092638975f06a2a",
        Some(10200),
        &["relative-imports-before-incomplete-member"],
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
        &["dynamic-slot-and-prop", "dependency-json-module"],
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

#[test]
fn content_mapper_multiline_callbacks_keep_contextual_parameters() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "content-mapper-multiline-callback-10191",
        "content_mapper_multiline_callbacks_keep_contextual_parameters",
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
        Some(10191),
        &["contextual-parameter"],
    );
}

#[test]
fn content_mapper_define_slots_resolves_setup_scoped_types() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "content-mapper-define-slots-typeof-10197",
        "content_mapper_define_slots_resolves_setup_scoped_types",
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
        Some(10197),
        &["setup-function"],
    );
}

#[test]
fn content_mapper_nested_vfor_reuses_branch_guards() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "content-mapper-nested-vfor-guard-10216",
        "content_mapper_nested_vfor_reuses_branch_guards",
        "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
        Some(10216),
        &["else-if"],
    );
}

#[test]
fn generic_slot_outlets_check_against_the_setup_parameters() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "generic-slot-outlet-10883",
        "generic_slot_outlets_check_against_the_setup_parameters",
        "42ca747027a86da7f94b473cff737cb330134d8c",
        Some(10883),
        &[
            "defaulted-parameter",
            "constrained-parameter",
            "wrong-payload",
        ],
    );
}

#[test]
fn slot_host_guards_preserve_authored_impossible_comparisons() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "narrowed-slot-host-guard-10660",
        "slot_host_guards_preserve_authored_impossible_comparisons",
        "da6d833de40c8a084022d0229beaeec25a92eb41",
        Some(10660),
        &["explicit-default", "authored-impossible-comparison"],
    );
}

#[test]
fn empty_generic_markers_preserve_functional_public_contracts() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "empty-generic-public-contract-10540",
        "empty_generic_markers_preserve_functional_public_contracts",
        "4fdc457df6498c4636efb35dd76af61394ad7006",
        Some(10540),
        &["frozen-root-element-contract"],
    );
}

#[test]
fn global_scope_build_info_keeps_complete_diagnostics() {
    check_pack(
        "global-scope-build-info-10819",
        "global_scope_build_info_keeps_complete_diagnostics",
        "61d756842bb44a72c84982f01dc40d3423c96503",
        Some(10819),
        &["imported-global-declaration", "edited-global-declaration"],
    );
}

#[test]
fn missing_exported_package_target_preserves_ts2307() {
    check_pack(
        "package-target-deletion-refresh",
        "missing_exported_package_target_preserves_ts2307",
        "9dfa45b2bcbbb5ca2ff3548ba16e607f0fc283f0",
        None,
        &["missing-target"],
    );
}

#[test]
fn rewritten_import_literals_preserve_complete_batch_diagnostics() {
    check_pack(
        "import-literal-geometry-10762",
        "rewritten_import_literals_preserve_complete_batch_diagnostics",
        "51d3bebcc8988ba6bf42bb5f6b7fdc102de47ee3",
        Some(10762),
        &["original-alias-graph", "after-import-type-error"],
    );
}

#[test]
fn escaped_module_paths_preserve_complete_authored_diagnostics() {
    check_pack(
        "escaped-import-literals-10763",
        "escaped_module_paths_preserve_complete_authored_diagnostics",
        "178f9d7b9137d34941cfdd8cf6b9051bef3b79bd",
        Some(10763),
        &[
            "original-quoted-alias-graph",
            "quoted-alias-with-authored-type-error",
        ],
    );
}

#[test]
fn long_prop_keys_preserve_static_and_async_listener_diagnostics() {
    check_pack(
        "long-prop-kebab-case-10423",
        "long_prop_keys_preserve_static_and_async_listener_diagnostics",
        "d8849fab885cb40b9516889a464549fd29af3f92",
        Some(10423),
        &["static-and-async-with-listeners"],
    );
}

#[test]
fn pug_and_jade_preprocessing_errors_preserve_authored_diagnostics() {
    check_pack(
        "pug-refusal-10430",
        "pug_and_jade_preprocessing_errors_preserve_authored_diagnostics",
        "cb141dfffe07cce5d49bf6221b5d6b7ad8f5cc8e",
        Some(10430),
        &["refused-jade-with-valid-pug-sibling"],
    );
}

#[test]
fn portable_pug_refusals_preserve_native_batch_diagnostics() {
    check_pack(
        "portable-pug-10434",
        "portable_pug_refusals_preserve_native_batch_diagnostics",
        "c2132e45d47aad595aac7e07f5e71a78b8ffd1c8",
        Some(10434),
        &["refused-jade-with-valid-pug-sibling"],
    );
}

#[test]
fn jsx_prop_tokens_preserve_complete_batch_diagnostics() {
    check_pack(
        "jsx-prop-token-ranges-10494",
        "jsx_prop_tokens_preserve_complete_batch_diagnostics",
        "aaeeb1c1264a319737fa11e56971c9d73064d6a3",
        Some(10494),
        &["tsx-invalid-and-valid-props", "jsx-invalid-and-valid-props"],
    );
}

#[test]
fn unicode_prop_bindings_preserve_complete_authored_diagnostics() {
    check_pack(
        "unicode-prop-bindings-10573",
        "unicode_prop_bindings_preserve_complete_authored_diagnostics",
        "751dc1971ba6613ea2a5642e57a4a76b16fc9353",
        Some(10573),
        &[
            "reported-optional-unused",
            "unicode-ascii-collision",
            "unicode-ascii-number",
            "quoted-cooked-unicode",
            "escaped-identifier",
            "supplementary-crlf",
            "runtime-japanese",
            "defaulted-unicode",
            "keyed-unicode",
            "reserved-hyphen-ascii",
            "unicode-continue",
        ],
    );
}

#[test]
fn content_mapper_merged_class_values_keep_native_diagnostics() {
    check_pack(
        "content-mapper-merged-class-10788",
        "content_mapper_merged_class_values_keep_native_diagnostics",
        "a92a2dfaf9d6b62e567c61edb12c6e4f9196c3ea",
        Some(10788),
        &["dynamic-component-class"],
    );
}

#[test]
fn content_mapper_slot_host_callbacks_keep_native_diagnostics() {
    check_pack(
        "content-mapper-slot-host-callbacks-10774",
        "content_mapper_slot_host_callbacks_keep_native_diagnostics",
        "a92a2dfaf9d6b62e567c61edb12c6e4f9196c3ea",
        Some(10774),
        &["function-prop-with-named-slot"],
    );
}

#[test]
fn native_completion_project_keeps_clean_diagnostics() {
    check_pack(
        "native-completion-order-10747",
        "native_completion_project_keeps_clean_diagnostics",
        "7e710ef9e094eb3244e332c90f21727d290b4a35",
        Some(10747),
        &["imported-identifier-project"],
    );
}

#[test]
fn repaired_dependency_buffer_keeps_its_own_diagnostics() {
    check_pack(
        "reopen-repaired-diagnostics-10730",
        "repaired_dependency_buffer_keeps_its_own_diagnostics",
        "9403195279c75693aea30f097bf996cb83dd9729",
        Some(10730),
        &["error", "repaired"],
    );
}

#[test]
fn editor_alias_import_extension_styles_keep_native_diagnostics() {
    check_pack(
        "editor-alias-import-extension-10753",
        "editor_alias_import_extension_styles_keep_native_diagnostics",
        "377e5df97c4effa786a6426a62f61296ce16c750",
        Some(10753),
        &["authored-extension-styles"],
    );
}

#[test]
fn callback_render_position_preserves_complete_authored_diagnostics() {
    check_pack(
        "callback-render-position-10738",
        "callback_render_position_preserves_complete_authored_diagnostics",
        "78719553cf550bcffbf1d0fb891fda642f42de3a",
        Some(10738),
        &["type-query-before-delete"],
    );
}

#[test]
fn experimental_page_import_preserves_published_path_diagnostics() {
    check_pack(
        "experimental-router-import-10520",
        "experimental_page_import_preserves_published_path_diagnostics",
        "a29d9c0b1961ada51037db402c3b74436a56d623",
        Some(10520),
        &["imported-define-page-unknown-path"],
    );
}

#[test]
fn literal_reference_paths_preserve_complete_authored_diagnostics() {
    check_pack(
        "literal-reference-paths-10551",
        "literal_reference_paths_preserve_complete_authored_diagnostics",
        "0a7f029e3be412a89232bd82127d3ac9bb058514",
        Some(10551),
        &["valid-external-reference", "invalid-external-reference"],
    );
}

#[test]
fn content_mapper_experimental_router_context_preserves_published_page_types() {
    check_pack(
        "content-mapper-experimental-router-10526",
        "content_mapper_experimental_router_context_preserves_published_page_types",
        "190c82fb9c8bbdc2c519b335b1661563270f4d2c",
        Some(10526),
        &["published-page-valid", "published-page-invalid-number"],
    );
}

#[test]
fn component_input_copies_preserve_complete_authored_diagnostics() {
    check_pack(
        "component-input-diagnostic-owners-10607",
        "component_input_copies_preserve_complete_authored_diagnostics",
        "bd905e766c37ad5bacabaacbec02c90def36726e",
        Some(10607),
        &[
            "art-generic-form-invalid",
            "art-generic-form-repaired",
            "slot-prop-without-children-invalid",
            "slot-prop-named-child-invalid",
            "slot-prop-two-named-children-invalid",
            "slot-prop-named-child-repaired",
            "two-files-same-offset-invalid",
        ],
    );
}

#[test]
fn undefined_context_keeps_complete_authored_diagnostics() {
    check_pack(
        "template-context-undefined-10667",
        "undefined_context_keeps_complete_authored_diagnostics",
        "2806a05b4b9dac514afd0fb5c0cd3fb9925d9c54",
        Some(10667),
        &["prop-used"],
    );
}

#[test]
fn options_api_reserved_props_resolve_dynamic_component_references() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "options-api-reserved-component-reference",
        "options_api_reserved_props_resolve_dynamic_component_references",
        "482dd05d23ac9d1bec4065608dd8fe562a11b3b0",
        None,
        &[
            "reserved-prop-dynamic-component",
            "reserved-prop-dynamic-component-wrong-member",
            "reserved-prop-authored-local-collision",
        ],
    );
}

#[test]
fn empty_v_slot_expressions_keep_slot_scopes_checked() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "empty-v-slot-expression",
        "empty_v_slot_expressions_keep_slot_scopes_checked",
        "482dd05d23ac9d1bec4065608dd8fe562a11b3b0",
        None,
        &[
            "empty-expression",
            "blank-expression",
            "empty-expression-unknown-slot",
        ],
    );
}
