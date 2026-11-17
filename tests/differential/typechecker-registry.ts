// Active #6852 pins are corpus inputs without a batch pack. Keep that manifest
// distinction beside the pack registry instead of teaching every consumer it.
export function packCases<T extends { pack?: unknown }>(cases: readonly T[]): T[] {
  return cases.filter((fixture) => fixture.pack !== undefined);
}

// Exact production fixture-body and original-source registrations, sorted by pack.
export const TYPECHECKER_TESTS = {
  "allow-importing-ts-extensions-9890": "authored_ts_extensions_report_ts5097_by_default",
  "ambient-declarations-3095":
    "fix_history_cases::ambient_declarations_3095::ambient_augmentations_preserve_batch_diagnostics",
  "aria-kebab-prop-11048":
    "aria_kebab_prop_11048::declared_kebab_aria_props_preserve_batch_diagnostics",
  "aria-propertykey-capture-11153":
    "aria_propertykey_capture_11153::authored_propertykey_alias_preserves_batch_diagnostics",
  "art-variant-implicit-component-9854":
    "implicit_art_component_reports_the_same_prop_errors_as_an_explicit_import",
  "authored-unused-symbols": "authored_unused_symbols_preserve_complete_original_diagnostics",
  "branded-ref-template-types-2957":
    "fix_history_cases::branded_ref_template_types_2957::branded_ref_template_unwrap_preserves_brand",
  "cache-maintenance-9860": "bounded_cache_maintenance_preserves_project_diagnostics",
  "callback-render-position-10738":
    "callback_render_position_preserves_complete_authored_diagnostics",
  "component-derived-reserved-props":
    "fix_history_cases::component_derived_reserved_props::component_derived_reserved_prop_shorthand",
  "component-event-tuples": "component_event_tuples_preserve_all_argument_diagnostics",
  "component-fallthrough-touch-listener-2732":
    "fix_history_cases::component_fallthrough_touch_listener_2732::component_fallthrough_touch_listener_stays_loose",
  "component-input-diagnostic-owners-10607":
    "component_input_copies_preserve_complete_authored_diagnostics",
  "component-listener-alias-scope-10112": "component_listener_aliases_stay_inside_guard_scope",
  "component-listener-prop-duplicate-10397": "required_prop_reports_once_with_or_without_listener",
  "component-merged-class-bindings-2538":
    "fix_history_cases::component_merged_class_bindings_2538::component_static_and_dynamic_class_bindings_merge",
  "component-render-h-props-9855": "imported_sfc_h_calls_check_missing_and_wrong_props",
  "component-tag-aliases-11020": "imported_component_tag_aliases_preserve_batch_prop_diagnostics",
  "component-v-bind-duplicate-diagnostic-10069":
    "named_prop_before_spread_reports_once_with_or_without_listener",
  "content-mapper-const-enum-import": "content_mapper_accepts_imported_const_enum_value_anchors",
  "content-mapper-define-slots-typeof-10197":
    "content_mapper_define_slots_resolves_setup_scoped_types",
  "content-mapper-diagnostics-10187": "content_mapper_diagnostics_keep_single_public_owners",
  "content-mapper-duplicate-attributes-10184":
    "content_mapper_isolates_duplicate_class_and_style_attribute_probes",
  "content-mapper-dynamic-template-ref-10198": "dynamic_template_refs_preserve_batch_acceptance",
  "content-mapper-dynamic-template-ref-scope-10251":
    "dynamic_template_ref_union_registry_stays_in_scope",
  "content-mapper-dynamic-template-ref-union-10251":
    "dynamic_template_ref_unions_preserve_batch_acceptance",
  "content-mapper-experimental-router-10526":
    "content_mapper_experimental_router_context_preserves_published_page_types",
  "content-mapper-filename-globals-10190": "filename_globals_preserve_clean_batch_diagnostics",
  "content-mapper-global-component-10185":
    "content_mapper_global_components_preserve_batch_acceptance",
  "content-mapper-javascript-checkjs-10217": "plain_javascript_sfcs_follow_check_js_policy",
  "content-mapper-merged-class-10788": "content_mapper_merged_class_values_keep_native_diagnostics",
  "content-mapper-multiline-callback-10191":
    "content_mapper_multiline_callbacks_keep_contextual_parameters",
  "content-mapper-musea-art-10186":
    "content_mapper_musea_art_project_preserves_bound_component_diagnostics",
  "content-mapper-nested-vfor-guard-10216": "content_mapper_nested_vfor_reuses_branch_guards",
  "content-mapper-nuxt-emit-10195": "content_mapper_preserves_emit_with_nuxt_pattern_injections",
  "content-mapper-props-binding-10196": "authored_props_binding_preserves_batch_acceptance",
  "content-mapper-slot-host-callbacks-10774":
    "content_mapper_slot_host_callbacks_keep_native_diagnostics",
  "content-mapper-template-callback-listener-11023":
    "content_mapper_template_callback_listener_11023::content_mapper_template_callback_listener_preserves_contextual_parameters",
  "content-mapper-template-expression-syntax":
    "template_expression_syntax_preserves_complete_authored_diagnostics",
  "content-mapper-type-only-import-use":
    "content_mapper_keeps_inferred_type_only_imports_out_of_value_anchors",
  "content-mapper-unknown-options-10218":
    "content_mapper_unknown_options_preserve_requested_diagnostics",
  "content-mapper-vue-generic-11051":
    "content_mapper_vue_generic_11051::content_mapper_vue_generic_preserves_explicit_type_arguments",
  "contextual-prop-anchors-10661": "contextual_prop_anchors_preserve_authored_diagnostics",
  "declared-any-slot-provider-11027":
    "fix_history_cases::declared_any_slot_provider_11027::declared_any_slot_providers_preserve_authored_errors_and_intrinsic_controls",
  "declared-prop-names-original": "declared_prop_names_preserve_complete_original_diagnostics",
  "dependency-probe-history-10936": "dependency_probe_histories_preserve_complete_diagnostics",
  "dynamic-component-union-listener-10026":
    "dynamic_component_union_listener_preserves_clean_diagnostics",
  "dynamic-component-v-for-scope":
    "fix_history_cases::dynamic_component_v_for_scope::dynamic_is_on_v_for_elements_preserves_batch_diagnostics",
  "dynamic-directive-expression-mapping-10182":
    "dynamic_directive_expressions_preserve_typed_slot_payloads",
  "dynamic-member-component-emits-2915":
    "fix_history_cases::dynamic_member_component_emits_2915::dynamic_member_component_listener_infers_emit_payload",
  "editor-alias-import-extension-10753":
    "editor_alias_import_extension_styles_keep_native_diagnostics",
  "empty-generic-public-contract-10540":
    "empty_generic_markers_preserve_functional_public_contracts",
  "empty-template-interpolation-3090":
    "fix_history_cases::empty_template_interpolation_3090::empty_interpolations_preserve_batch_diagnostics",
  "empty-v-slot-expression": "empty_v_slot_expressions_keep_slot_scopes_checked",
  "escaped-import-literals-10763": "escaped_module_paths_preserve_complete_authored_diagnostics",
  "event-handler-narrowing": "inline_event_assignments_preserve_exact_diagnostics",
  "event-inference-callback-10739": "event_inference_callbacks_keep_contextual_native_diagnostics",
  "every-unused-sfc-generic-2922":
    "fix_history_cases::every_unused_sfc_generic_2922::every_sfc_generic_used_by_props_or_emits_is_retained",
  "experimental-router-import-10520":
    "experimental_page_import_preserves_published_path_diagnostics",
  "experimental-router-props-10579": "experimental_route_props_follow_the_admitted_project_types",
  "exposed-template-ref-unwrap-2632":
    "fix_history_cases::exposed_template_ref_unwrap_2632::exposed_template_ref_unwraps_on_component_instance",
  "function-prefixed-listener-call-3512":
    "fix_history_cases::function_prefixed_listener_call_3512::function_prefixed_listener_call_is_not_an_inline_function",
  "generic-callback-listener-props-2921":
    "fix_history_cases::generic_callback_listener_props_2921::generic_callback_prop_listener_infers_payload",
  "generic-component-props-inference-2476":
    "fix_history_cases::generic_component_props_inference_2476::generic_component_props_infer_from_template",
  "generic-constructor-slots-10416": "generic_constructor_intersections_preserve_slot_payloads",
  "generic-default-setup-scope-2912":
    "fix_history_cases::generic_default_setup_scope_2912::generic_default_does_not_replace_setup_parameter",
  "generic-emit-overload-depth-3150":
    "fix_history_cases::generic_emit_overload_depth_3150::generic_emit_overload_listener_props_stay_within_depth",
  "generic-fallthrough-spread-9956": "generic_fallthrough_spread_preserves_valid_values",
  "generic-imported-boolean-keys-2920":
    "fix_history_cases::generic_imported_boolean_keys_2920::generic_props_resolve_imported_boolean_keys",
  "generic-props-boolean-keys-2633":
    "fix_history_cases::generic_props_boolean_keys_2633::generic_props_do_not_gain_boolean_intersections",
  "generic-sfc-extracted-props-2701":
    "fix_history_cases::generic_sfc_extracted_props_2701::generic_sfc_props_infer_through_extracted_component_props",
  "generic-slot-outlet-10883": "generic_slot_outlets_check_against_the_setup_parameters",
  "global-scope-build-info-10819": "global_scope_build_info_keeps_complete_diagnostics",
  "guard-diagnostic-position-2960":
    "fix_history_cases::guard_diagnostic_position_2960::authored_guards_own_batch_diagnostics",
  "import-literal-geometry-10762": "rewritten_import_literals_preserve_complete_batch_diagnostics",
  "imported-emits-generic-2914":
    "fix_history_cases::imported_emits_generic_2914::imported_define_emits_type_retains_sfc_generic",
  "imported-ref-unwrap-2958":
    "fix_history_cases::imported_ref_unwrap_2958::imported_refs_preserve_batch_diagnostics",
  "incomplete-script-setup-recovery-10200": "pins_incomplete_setup_import_corpus",
  "inferred-project-tsconfig-9718": "inferred_project_tsconfig_preserves_owned_batch_inputs",
  "jsx-prop-token-ranges-10494": "jsx_prop_tokens_preserve_complete_batch_diagnostics",
  "leading-semicolon-handler-3091":
    "fix_history_cases::leading_semicolon_handler_3091::asi_prefixed_handlers_preserve_batch_diagnostics",
  "library-generic-props-fallback-13021":
    "library_generic_props_fallback::old_library_generic_props_follow_the_unknown_props_policy",
  "listener-attribute-parity-9852": "listener_keeps_class_style_and_spread_attribute_parity",
  "literal-reference-paths-10551": "literal_reference_paths_preserve_complete_authored_diagnostics",
  "long-prop-kebab-case-10423": "long_prop_keys_preserve_static_and_async_listener_diagnostics",
  "misskey-generated-inference-9516":
    "misskey_generated_inference_probes_preserve_authored_diagnostics",
  "narrowed-slot-host-guard-10660": "slot_host_guards_preserve_authored_impossible_comparisons",
  "native-completion-order-10747": "native_completion_project_keeps_clean_diagnostics",
  "native-expect-error-10413": "vue3_native_expectations_preserve_strict_ownership",
  "native-inline-event-handler-9835": "native_inline_event_handler_is_contextually_typed",
  "native-pointer-events-original": "native_click_events_preserve_pointer_event_diagnostics",
  "nested-v-if-narrowing-2959":
    "fix_history_cases::nested_v_if_narrowing_2959::nested_else_v_for_preserves_batch_diagnostics",
  "no-space-default-export-3087":
    "fix_history_cases::no_space_default_export_3087::no_space_default_export_is_wrapped_as_options_api",
  "nuxt-plugin-injection-types": "nuxt_reference_graph_preserves_plugin_injection_import_types",
  "nuxt-unknown-template-options-10013": "nuxt_unknown_template_options_preserve_explicit_checks",
  "optional-boolean-template-props-2731":
    "fix_history_cases::optional_boolean_template_props_2731::optional_boolean_template_prop_is_normalized",
  "optional-event-handlers-3148":
    "fix_history_cases::optional_event_handlers_3148::optional_handlers_preserve_batch_diagnostics",
  "optional-fallthrough-spread-10170":
    "optional_fallthrough_spread_preserves_valid_values_and_rejects_invalid_values",
  "options-api-any-instance": "options_api_any_instance_preserves_complete_original_diagnostics",
  "options-api-reserved-component-reference":
    "options_api_reserved_props_resolve_dynamic_component_references",
  "options-runtime-props-value-scope-3089":
    "fix_history_cases::options_runtime_props_value_scope_3089::options_runtime_props_check_from_value_scope",
  "options-template-literal-bindings-3086":
    "fix_history_cases::options_template_literal_bindings_3086::options_api_template_literals_are_not_bindings",
  "package-target-deletion-refresh": "missing_exported_package_target_preserves_ts2307",
  "plain-script-enum-class-exports-3382":
    "fix_history_cases::plain_script_enum_class_exports_3382::plain_script_value_and_type_exports_preserve_batch_diagnostics",
  "plain-script-named-exports-1966":
    "fix_history_cases::plain_script_named_exports_1966::plain_script_value_exports_preserve_batch_diagnostics",
  "plain-script-type-body-2038":
    "fix_history_cases::plain_script_type_body_2038::plain_script_type_bodies_preserve_batch_diagnostics",
  "planned-generic-syntax-route-10598":
    "planned_generic_syntax_keeps_independent_body_errors_through_repairs",
  "portable-pug-10434": "portable_pug_refusals_preserve_native_batch_diagnostics",
  "public-instance-ts-directives-10984":
    "public_instance_ts_directives_preserve_owned_native_diagnostics",
  "pug-refusal-10430": "pug_and_jade_preprocessing_errors_preserve_authored_diagnostics",
  "quoted-generic-parameters-10540":
    "quoted_generic_parameters_preserve_complete_batch_diagnostics",
  "reactive-props-destructure-10008": "reactive_props_destructure_preserves_unused_diagnostics",
  "relative-sibling-storage-10934":
    "relative_sibling_imports_preserve_complete_package_diagnostics",
  "reopen-repaired-diagnostics-10730": "repaired_dependency_buffer_keeps_its_own_diagnostics",
  "repeated-component-prop-checks-3202":
    "fix_history_cases::repeated_component_prop_checks_3202::repeated_component_prop_names_keep_unique_checks",
  "required-default-props": "defaulted_required_props_stay_optional_for_the_parent",
  "required-props-edges": "required_props_keep_exact_unicode_diagnostics",
  "reserved-enum-member-3092":
    "fix_history_cases::reserved_enum_member_3092::reserved_enum_members_preserve_batch_diagnostics",
  "reserved-expression-shapes": "reserved_prop_shapes_preserve_literal_and_member_diagnostics",
  "runtime-prop-helper-syntax":
    "fix_history_cases::runtime_prop_helper_syntax::runtime_prop_helper_emits_valid_typescript",
  "runtime-props-report-10100-10101": "runtime_props_reports_10100_10101_retain_expected_types",
  "script-block-duplicate-declarations-4241":
    "fix_history_cases::script_block_duplicate_declarations_4241::script_block_collisions_preserve_batch_diagnostics",
  "script-block-scope-isolation-4241":
    "fix_history_cases::script_block_scope_isolation_4241::cross_block_type_sharing_preserves_batch_diagnostics",
  "setup-ambient-adjacent-6223":
    "fix_history_cases::setup_ambient_adjacent_6223::adjacent_ambient_declarations_preserve_lf_batch_diagnostics",
  "setup-ambient-adjacent-crlf-6223":
    "fix_history_cases::setup_ambient_adjacent_crlf_6223::adjacent_ambient_declarations_preserve_crlf_batch_diagnostics",
  "setup-ambient-captures-6232":
    "fix_history_cases::setup_ambient_captures_6232::setup_dependent_ambient_captures_preserve_lf_batch_diagnostics",
  "setup-ambient-captures-crlf-6232":
    "fix_history_cases::setup_ambient_captures_crlf_6232::setup_dependent_ambient_captures_preserve_crlf_batch_diagnostics",
  "setup-ambient-declarations-6170":
    "fix_history_cases::setup_ambient_declarations_6170::independent_ambient_setup_declarations_preserve_batch_diagnostics",
  "setup-ambient-global-types-6211":
    "fix_history_cases::setup_ambient_global_types_6211::ambient_setup_global_types_preserve_batch_diagnostics",
  "setup-custom-properties": "custom_properties_resolve_on_the_template_instance",
  "setup-generic-merged-type-exports-3094":
    "fix_history_cases::setup_generic_merged_type_exports_3094::setup_generic_and_merged_type_exports_stay_legal",
  "setup-props-shadow-anchor-3934":
    "fix_history_cases::setup_props_shadow_anchor_3934::setup_props_read_by_template_is_not_unused",
  "setup-scoped-type-exports-3093":
    "fix_history_cases::setup_scoped_type_exports_3093::setup_scoped_value_dependent_type_exports_are_public",
  "slot-outlet-key": "slot_outlet_keys_preserve_complete_project_diagnostics",
  "split-script-original": "split_script_setup_preserves_complete_original_diagnostics",
  "static-component-prop-values-3197":
    "fix_history_cases::static_component_prop_values_3197::static_component_attribute_values_are_type_checked",
  "strict-fallthrough-default-7856": "strict_component_attrs_infer_default_fallthrough_surfaces",
  "template-context-undefined-10667": "undefined_context_keeps_complete_authored_diagnostics",
  "template-definite-assignment": "deferred_template_reads_preserve_script_diagnostics",
  "template-loop-narrowing-3149":
    "fix_history_cases::template_loop_narrowing_3149::v_for_callbacks_preserve_batch_diagnostics",
  "template-ref-signatures-10948": "template_ref_signatures_preserve_readonly_unknown_and_repairs",
  "template-suppression-original": "template_suppression_preserves_complete_original_diagnostics",
  "tsconfig-literal-brackets-10819": "literal_bracket_pages_keep_their_project_options",
  "typed-import-meta": "typed_import_meta_preserves_exact_authored_diagnostics",
  "undeclared-instance-globals-10617":
    "undeclared_instance_globals_preserve_complete_authored_diagnostics",
  "undefined-unknown-listener-2956":
    "fix_history_cases::undefined_unknown_listener_2956::undefined_unknown_component_listener_is_optional",
  "unicode-prop-bindings-10573": "unicode_prop_bindings_preserve_complete_authored_diagnostics",
  "unicode-reserved-props": "unicode_reserved_props_preserve_value_diagnostics",
  "unknown-component-slot-payload-11027":
    "unknown_component_slots_preserve_authored_type_diagnostics",
  "v-for-component-source-original":
    "component_v_for_source_preserves_complete_original_diagnostics",
  "v-for-source-original": "v_for_source_callbacks_preserve_complete_original_diagnostics",
  "valueless-directive-import-3140":
    "valueless_directive_imports_preserve_upstream_diagnostic_parity",
  "with-defaults-loose-required-2924":
    "fix_history_cases::with_defaults_loose_required_2924::with_defaults_retains_loose_required_props",
  "with-defaults-optional-union-2913":
    "fix_history_cases::with_defaults_optional_union_2913::with_defaults_preserves_optional_union_props",
  "with-defaults-undefined-default":
    "fix_history_cases::with_defaults_undefined_default::with_defaults_undefined_default_stays_optional",
  "workspace-alias-cache-10063": "workspace_package_alias_cache_preserves_sequential_scopes",
  "workspace-package-import-alias-10175":
    "workspace_package_import_aliases_preserve_clean_diagnostics",
  "zero-argument-slot-constructor-11013":
    "zero_argument_slot_constructors_preserve_mapper_and_authored_diagnostics",
} as const;

const originalFixtureRevision = "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943";
export const sourceRevisions = {
  "allow-importing-ts-extensions-9890": "f1b07174a4a86ffc3433f31b8c78406d5bf5b760",
  "ambient-declarations-3095": "9129f14b5fa108e71cb5deed590f68f39054c1c2",
  "aria-kebab-prop-11048": "65b38273c8713a724f42f0bb63a2aef7cc6daae4",
  "aria-propertykey-capture-11153": "aed91c7626f9daf11a07b9eb32a0b7ff27ec9038",
  "art-variant-implicit-component-9854": "f07065df35d25bc76f355e5f106c80dbefa922b5",
  "authored-unused-symbols": "cd7156d28386e072953476fdbc354a963758dc89",
  "branded-ref-template-types-2957": "145baca6ae3b4a45b847b7dcbf2cbcc39cd1e72d",
  "cache-maintenance-9860": "4714c0f264dedffaa81377df9545fbfa9514a64e",
  "callback-render-position-10738": "78719553cf550bcffbf1d0fb891fda642f42de3a",
  "component-derived-reserved-props": "8fe6bd909063488f53b8c46e0a8301e57d1cd104",
  "component-event-tuples": originalFixtureRevision,
  "component-fallthrough-touch-listener-2732": "b41d999468128f920fcbbda1cc5c019e90dc75ee",
  "component-input-diagnostic-owners-10607": "bd905e766c37ad5bacabaacbec02c90def36726e",
  "component-listener-alias-scope-10112": "7cefe3a00e6184de1e121d39b0e153f08a428aa6",
  "component-listener-prop-duplicate-10397": "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943",
  "component-merged-class-bindings-2538": "ed38d543eeed916cf98a6be84d58e3b4b84015ce",
  "component-render-h-props-9855": "f15ca234a4bdd73bde9caea07e652f3a345eb342",
  "component-tag-aliases-11020": "d653e04143051768a96ba51a35fe64c055caf0c5",
  "component-v-bind-duplicate-diagnostic-10069": "7174c46d55825c155325e6314305e661d50ec95a",
  "content-mapper-const-enum-import": "506529117573a96880435dd6466cfd7cf81fdc92",
  "content-mapper-define-slots-typeof-10197": "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
  "content-mapper-diagnostics-10187": "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
  "content-mapper-duplicate-attributes-10184": "0038b601e41dccb6f037aa68c3fc0207ea635e3b",
  "content-mapper-dynamic-template-ref-10198": "b7fb3a9aefd31002b8b2d449695a9e14d657654a",
  "content-mapper-dynamic-template-ref-scope-10251": "8d83800c727eef953852383d46c7324ccdda53b6",
  "content-mapper-dynamic-template-ref-union-10251": "6817c853ca1740fd1070c0ec462c0d457f339c2c",
  "content-mapper-experimental-router-10526": "190c82fb9c8bbdc2c519b335b1661563270f4d2c",
  "content-mapper-filename-globals-10190": "55f388568ab900f5b0076d8c34280024ba988044",
  "content-mapper-global-component-10185": "10a9eaa743e3fb92b6372daa64c509bce2b204f1",
  "content-mapper-javascript-checkjs-10217": "c9b9b5c368fceeed61c13813a617f7a8b5b8351f",
  "content-mapper-merged-class-10788": "a92a2dfaf9d6b62e567c61edb12c6e4f9196c3ea",
  "content-mapper-multiline-callback-10191": "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
  "content-mapper-musea-art-10186": "d295b324299c549063fab385b7f308c71ac014e0",
  "content-mapper-nested-vfor-guard-10216": "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
  "content-mapper-nuxt-emit-10195": "369f01b50d8fe6e3b0996cc9a90eecf422dbdd3b",
  "content-mapper-props-binding-10196": "232f8f514e0ea55a7aeef08bc883b46e0608d12f",
  "content-mapper-slot-host-callbacks-10774": "a92a2dfaf9d6b62e567c61edb12c6e4f9196c3ea",
  "content-mapper-template-callback-listener-11023": "407e7cf15219405110efdd0c977bb30cb1db1644",
  "content-mapper-template-expression-syntax": "a5ae4d313fa333c8bc7168d5da4acf3dc2bb8ee3",
  "content-mapper-type-only-import-use": "fb89f41ffc518768e3053ccf9fe6b83bf24a1122",
  "content-mapper-unknown-options-10218": "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799",
  "content-mapper-vue-generic-11051": "dfc8b0e13c20ea7c8d46e555f70ffd7fe3e89a27",
  "contextual-prop-anchors-10661": "da6d833de40c8a084022d0229beaeec25a92eb41",
  "declared-any-slot-provider-11027": "be8aa8108652c10b7eb02d963e8862fcae6b542c",
  "declared-prop-names-original": "0f15c8b49d0630c1f2492c7ad8447ef4c9711534",
  "dependency-probe-history-10936": "1266cd3a014654bbdcf3378500b93447411b26d9",
  "dynamic-component-union-listener-10026": "0aafae7e41f1ac381d07fa9f0154084cc1e6f211",
  "dynamic-component-v-for-scope": "7f802b69bf57fc555e65174d33d8319fc4753aa4",
  "dynamic-directive-expression-mapping-10182": "f305bcc8c81b8f6009dabb5a3a61bb838c755701",
  "dynamic-member-component-emits-2915": "9c8861cfcbca6ba118dfff9b14250bb30e6aee39",
  "editor-alias-import-extension-10753": "377e5df97c4effa786a6426a62f61296ce16c750",
  "empty-generic-public-contract-10540": "4fdc457df6498c4636efb35dd76af61394ad7006",
  "empty-template-interpolation-3090": "7c18c96fe98af6b74f57753b7801e647f751d50d",
  "empty-v-slot-expression": "482dd05d23ac9d1bec4065608dd8fe562a11b3b0",
  "escaped-import-literals-10763": "62652a4d4029fa11016dfa1ac8c24669b42c8d7a",
  "event-handler-narrowing": originalFixtureRevision,
  "event-inference-callback-10739": "1afbda7e3875d350a56e9a55b9b8753c8fbfadb6",
  "every-unused-sfc-generic-2922": "9aad33772161de7133ef2ebd628e5d21f60b43ac",
  "experimental-router-import-10520": "a29d9c0b1961ada51037db402c3b74436a56d623",
  "experimental-router-props-10579": "5b5ca8272070fffb67152c1ccb20644abe11c62d",
  "exposed-template-ref-unwrap-2632": "745bc7c6301766b462bc8f5a9ba3104656c5c2a6",
  "function-prefixed-listener-call-3512": "7676592d51c264565259de273ccea511601d25b9",
  "generic-callback-listener-props-2921": "e3a12cf323cb3f7213034e5137564ca4daf794b1",
  "generic-component-props-inference-2476": "f4dbdb44c080b869630d8ad83d46a95fa356e124",
  "generic-constructor-slots-10416": "036b1a34364c3568b8f19d4dad7479aadcff9860",
  "generic-default-setup-scope-2912": "5afffd0b3cb0dd06d9eb347b57f40dba3d9c8e4e",
  "generic-emit-overload-depth-3150": "5257a671727fc55e94bb227247269d23a8eda13e",
  "generic-fallthrough-spread-9956": "2b1e41a5b09005a97d24d1ff34f34c7507bbfb67",
  "generic-imported-boolean-keys-2920": "37cb5631062a8926497835844f8d41db680128eb",
  "generic-props-boolean-keys-2633": "3c2edcdebd2f836c3e96a8447f57e000de453315",
  "generic-sfc-extracted-props-2701": "89131c4b4b3a605f5c12c771dccb54fe91d38482",
  "generic-slot-outlet-10883": "42ca747027a86da7f94b473cff737cb330134d8c",
  "global-scope-build-info-10819": "61d756842bb44a72c84982f01dc40d3423c96503",
  "guard-diagnostic-position-2960": "7f01c58cdd13bda478e7b1b70fe5575a173bd063",
  "import-literal-geometry-10762": "62652a4d4029fa11016dfa1ac8c24669b42c8d7a",
  "imported-emits-generic-2914": "db8f7e52c7f337fdacb8f18be99aaf8d53f2cc2f",
  "imported-ref-unwrap-2958": "57dd8e862ab9963567d6762629170bab3b464c05",
  "incomplete-script-setup-recovery-10200": "4a555b0aae0a394fbe4cedc08092638975f06a2a",
  "inferred-project-tsconfig-9718": "f95557dadaab83dc0f7d4a7613bdb7ed4e59d323",
  "jsx-prop-token-ranges-10494": "aaeeb1c1264a319737fa11e56971c9d73064d6a3",
  "leading-semicolon-handler-3091": "f6ffab4e808a96f2d0dac8827d4e00c7d7df260a",
  "library-generic-props-fallback-13021": "f06d427619f943bf1192cfafbae65de537e464d0",
  "listener-attribute-parity-9852": "11cb03950bc8d2a4e9cb8d164b0aaca79e302291",
  "literal-reference-paths-10551": "0a7f029e3be412a89232bd82127d3ac9bb058514",
  "long-prop-kebab-case-10423": "d8849fab885cb40b9516889a464549fd29af3f92",
  "misskey-generated-inference-9516": "cf6c83ab16fb7f20f84f0d51112090bbce89c74a",
  "narrowed-slot-host-guard-10660": "da6d833de40c8a084022d0229beaeec25a92eb41",
  "native-completion-order-10747": "7e710ef9e094eb3244e332c90f21727d290b4a35",
  "native-expect-error-10413": "b005ad1279cc17a2da628e4d7459a0cb9f65dbe7",
  "native-inline-event-handler-9835": "8ac6ce1668e9bede8d73411e63a247dbd6f8eb56",
  "native-pointer-events-original": "e7d8b1c82a8badec89ec9779838dfa50c1b9b790",
  "nested-v-if-narrowing-2959": "ac02c114ce529b6ef42e7ca7757279e1068548a6",
  "no-space-default-export-3087": "894b6f62b772b0a0e4e389bcc38cd9b8469d7557",
  "nuxt-plugin-injection-types": "2c22691f0f8303a28c425213c423e299f9ad5420",
  "nuxt-unknown-template-options-10013": "fb700551a3a3a69f8976efa6fe31bca43ebc19b4",
  "optional-boolean-template-props-2731": "a76940eb95b9fdbba90331b5b3b1bab2331ec902",
  "optional-event-handlers-3148": "805d32a5277a025d062edc47e20cc22ce170f5e7",
  "optional-fallthrough-spread-10170": "8300867552287abb14f4fbb7471c705aea72bbdf",
  "options-api-any-instance": "35bdad84760e6251edd52bb2e3e52701974a9d63",
  "options-api-reserved-component-reference": "482dd05d23ac9d1bec4065608dd8fe562a11b3b0",
  "options-runtime-props-value-scope-3089": "3ed9fbc889078cbbc9b1a86fa3b5f8dfd8612dd6",
  "options-template-literal-bindings-3086": "c40cb562b37a1cde3080bb560df6fbb23a574212",
  "package-target-deletion-refresh": "13fa41356d075047a229f580014b637d3bbeb57f",
  "plain-script-enum-class-exports-3382": "813a7ba187995f46cd4c3c0b8ec03104bcd9d9e3",
  "plain-script-named-exports-1966": "a4da0228d54f89923e2982044eeab2c4809529be",
  "plain-script-type-body-2038": "fd87bb1274a781dad70c3599df9fd3365a1eb870",
  "planned-generic-syntax-route-10598": "957827706dd439d9810a24f65b4a16c45f266b5e",
  "portable-pug-10434": "c2132e45d47aad595aac7e07f5e71a78b8ffd1c8",
  "public-instance-ts-directives-10984": "c001dc27973a316351d1d45739ad84a7db544f11",
  "pug-refusal-10430": "cb141dfffe07cce5d49bf6221b5d6b7ad8f5cc8e",
  "quoted-generic-parameters-10540": "711b56b2499b9136d8965a59c65713a2b78540f4",
  "reactive-props-destructure-10008": "f276e4d59cdaf765fd7d3f14eb3d679f5b685f98",
  "relative-sibling-storage-10934": "cb9c06d2b879da6b47a0a2f783d9a5f324776585",
  "reopen-repaired-diagnostics-10730": "9403195279c75693aea30f097bf996cb83dd9729",
  "repeated-component-prop-checks-3202": "378683f0c1feee643cd1e01347ff8cc6d31141db",
  "required-default-props": "34b2a8f60f2a73df5736422761c6f5f08cd21fe4",
  "required-props-edges": originalFixtureRevision,
  "reserved-enum-member-3092": "347d653aba6f5bc24592275a772e0877f367ce05",
  "reserved-expression-shapes": originalFixtureRevision,
  "runtime-prop-helper-syntax": "e49925f0fbde257854af2aa73e34b45be7f14b3d",
  "runtime-props-report-10100-10101": "5dfe002a9467d152c8545693df33def3aa5f3bf7",
  "script-block-duplicate-declarations-4241": "da1de3654e840605f65b18e6da536340cc6a28b4",
  "script-block-scope-isolation-4241": "da1de3654e840605f65b18e6da536340cc6a28b4",
  "setup-ambient-adjacent-6223": "be16d7c61b992af1e9af7e49a6975ce7a16e2512",
  "setup-ambient-adjacent-crlf-6223": "be16d7c61b992af1e9af7e49a6975ce7a16e2512",
  "setup-ambient-captures-6232": "0c94290f5f02b35a76f8ebc86128bcd1e1193e4a",
  "setup-ambient-captures-crlf-6232": "0c94290f5f02b35a76f8ebc86128bcd1e1193e4a",
  "setup-ambient-declarations-6170": "71451516ec37ed6ae600ba09f57f5d452961645a",
  "setup-ambient-global-types-6211": "a7fcfb4a5a17a4db0cd5fc21326205da7d020e1f",
  "setup-custom-properties": "482dd05d23ac9d1bec4065608dd8fe562a11b3b0",
  "setup-generic-merged-type-exports-3094": "5067f25786f316af91c33beb7ef08d6f4419378e",
  "setup-props-shadow-anchor-3934": "7c83dd0193b06a4cee3a1825a374ca0cd11fbd85",
  "setup-scoped-type-exports-3093": "71e38ec49f6931f5341d433bea8709d767daba87",
  "slot-outlet-key": "c6e43ca98cbffe099a6ef323006139d796d62208",
  "split-script-original": "b3c2933be5afbdb5f2df07da093f40bc7ce72187",
  "static-component-prop-values-3197": "c5af134e4924e31f31c8aba74f9984e356e1f6c2",
  "strict-fallthrough-default-7856": "d32790c869ac18c1e0e24e82a8876d796190133a",
  "template-context-undefined-10667": "2806a05b4b9dac514afd0fb5c0cd3fb9925d9c54",
  "template-definite-assignment": originalFixtureRevision,
  "template-loop-narrowing-3149": "2ecf10de92663303cecb76470bfa5ea42e992764",
  "template-ref-signatures-10948": "9157b9fa83dbdc98100be084be288f21d0d1bc18",
  "template-suppression-original": "e199979dfb32dfd8eb9ad32a2928fb010768f239",
  "tsconfig-literal-brackets-10819": "26b8fa31c7933d800c8bf4c2f37057864caaae9d",
  "typed-import-meta": "6e0f763bd0f5e986036244c2c17c9b658b0596bd",
  "undeclared-instance-globals-10617": "175dfd8a641fb7f06d60176d0a784e80e8c12e84",
  "undefined-unknown-listener-2956": "872d43f244876bf2b3241026abe3f1e2a552d58a",
  "unicode-prop-bindings-10573": "751dc1971ba6613ea2a5642e57a4a76b16fc9353",
  "unicode-reserved-props": originalFixtureRevision,
  "unknown-component-slot-payload-11027": "99ee7d1c4b99f9121f70a5f60d5f9aa81c596d47",
  "v-for-component-source-original": "04aedfb8e8b41dfae89fe65256e2703fc3718cf3",
  "v-for-source-original": "04aedfb8e8b41dfae89fe65256e2703fc3718cf3",
  "valueless-directive-import-3140": "88e8500c1e5f1b29d80f42c8ca065cc9cbd56899",
  "with-defaults-loose-required-2924": "a619bad6a42dc8c6e3cbd4f830a2e691a64c5cef",
  "with-defaults-optional-union-2913": "fc0e623bb9d2e9d50f496d98ca5f78bf1a607218",
  "with-defaults-undefined-default": "38b5adbedf092bca14579a827e1af4f4b28ad03c",
  "workspace-alias-cache-10063": "f305bcc8c81b8f6009dabb5a3a61bb838c755701",
  "workspace-package-import-alias-10175": "c935e00211b3d6ca271e53658f88156921a639a7",
  "zero-argument-slot-constructor-11013": "a95d0aad73c8403767a507f8f0768dd0ffb265c7",
} satisfies Record<keyof typeof TYPECHECKER_TESTS, string>;
