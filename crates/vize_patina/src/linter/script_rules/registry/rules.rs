//! The ordered table of built-in script rules. Order is load-bearing: the
//! first three are always-on and fix default diagnostic ordering. Append new
//! rules at the end of their category and mirror the addition in `names.rs`.

use super::names::{
    RULE_COMPONENT_OPTIONS_NAME_CASING, RULE_DEFINE_EMITS_DECLARATION, RULE_DEFINE_MACROS_ORDER,
    RULE_DEFINE_PROPS_DECLARATION, RULE_NO_ARROW_FUNCTIONS_IN_WATCH, RULE_NO_ASYNC_IN_COMPUTED,
    RULE_NO_DEEP_DESTRUCTURE_IN_PROPS, RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION,
    RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API, RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API,
    RULE_NO_DEPRECATED_EVENTS_API, RULE_NO_DUPE_KEYS, RULE_NO_EXPORT_IN_SCRIPT_SETUP,
    RULE_NO_GET_CURRENT_INSTANCE, RULE_NO_IMPORT_COMPILER_MACROS, RULE_NO_INTERNAL_IMPORTS,
    RULE_NO_NEXT_TICK, RULE_NO_OPTIONS_API, RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO,
    RULE_NO_REACTIVE_DESTRUCTURE, RULE_NO_RESERVED_IDENTIFIERS, RULE_NO_RESERVED_KEYS,
    RULE_NO_SIDE_EFFECTS_IN_COMPUTED, RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
    RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD, RULE_NO_WITH_DEFAULTS,
    RULE_PINIA_PREFER_STORE_TO_REFS, RULE_PREFER_COMPUTED, RULE_PREFER_IMPORT_FROM_VUE,
    RULE_PREFER_REF_OVER_REACTIVE, RULE_PREFER_USE_ATTRS, RULE_PREFER_USE_ID,
    RULE_PREFER_USE_SLOTS, RULE_PREFER_USE_TEMPLATE_REF, RULE_REQUIRE_FUNCTION_RETURN_TYPE,
    RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR, RULE_REQUIRE_SYMBOL_PROVIDE,
    RULE_RETURN_IN_COMPUTED_PROPERTY, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
};
use super::{
    BuiltinScriptRuleEntry, ECOSYSTEM_SCRIPT_PRESETS, OPINIONATED_SCRIPT_PRESETS,
    OPT_IN_SCRIPT_PRESETS,
};
use crate::rules::script::{
    ComponentOptionsNameCasing, DefineEmitsDeclaration, DefineMacrosOrder, DefinePropsDeclaration,
    NoArrowFunctionsInWatch, NoAsyncInComputed, NoDeepDestructureInProps,
    NoDeprecatedDataObjectDeclaration, NoDeprecatedDollarListenersApi,
    NoDeprecatedDollarScopedSlotsApi, NoDeprecatedEventsApi, NoDupeKeys, NoExportInScriptSetup,
    NoGetCurrentInstance, NoImportCompilerMacros, NoInternalImports, NoNextTick, NoOptionsApi,
    NoPotentialComponentOptionTypo, NoReactiveDestructure, NoReservedIdentifiers, NoReservedKeys,
    NoSideEffectsInComputed, NoTopLevelRefInScript, NoUseComputedPropertyLikeMethod,
    NoWithDefaults, PiniaPreferStoreToRefs, PreferComputed, PreferImportFromVue,
    PreferRefOverReactive, PreferUseAttrs, PreferUseId, PreferUseSlots, PreferUseTemplateRef,
    RequireFunctionReturnType, RequirePropTypeConstructor, RequireSymbolProvide,
    ReturnInComputedProperty, VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot,
};

static NO_DEEP_DESTRUCTURE_IN_PROPS_RULE: NoDeepDestructureInProps =
    NoDeepDestructureInProps { max_depth: 1 };

/// The full ordered set of built-in script rules. The first 6 engine-reachable
/// rules stay first to preserve default diagnostic ordering; the rest are opt-in.
#[rustfmt::skip]
pub(in crate::linter::script_rules) static BUILTIN_SCRIPT_RULES: &[BuiltinScriptRuleEntry] = &[
    BuiltinScriptRuleEntry { rule_name: RULE_NO_OPTIONS_API, profile_name: "patina.script_rule.no_options_api", category: "Vapor", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoOptionsApi },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_GET_CURRENT_INSTANCE, profile_name: "patina.script_rule.no_get_current_instance", category: "Vapor", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoGetCurrentInstance },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_NEXT_TICK, profile_name: "patina.script_rule.no_next_tick", category: "Vapor", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoNextTick },
    BuiltinScriptRuleEntry { rule_name: RULE_PINIA_PREFER_STORE_TO_REFS, profile_name: "patina.script_rule.pinia_prefer_store_to_refs", category: "Ecosystem", fixable: false, presets: ECOSYSTEM_SCRIPT_PRESETS, rule: &PiniaPreferStoreToRefs },
    BuiltinScriptRuleEntry { rule_name: RULE_VUE_ROUTER_PREFER_NAMED_PUSH, profile_name: "patina.script_rule.vue_router_prefer_named_push", category: "Ecosystem", fixable: false, presets: ECOSYSTEM_SCRIPT_PRESETS, rule: &VueRouterPreferNamedPush },
    BuiltinScriptRuleEntry { rule_name: RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT, profile_name: "patina.script_rule.vue_test_utils_no_html_snapshot", category: "Ecosystem", fixable: false, presets: ECOSYSTEM_SCRIPT_PRESETS, rule: &VueTestUtilsNoHtmlSnapshot },
    BuiltinScriptRuleEntry { rule_name: RULE_PREFER_COMPUTED, profile_name: "patina.script_rule.prefer_computed", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &PreferComputed },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_ASYNC_IN_COMPUTED, profile_name: "patina.script_rule.no_async_in_computed", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoAsyncInComputed },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_REACTIVE_DESTRUCTURE, profile_name: "patina.script_rule.no_reactive_destructure", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoReactiveDestructure },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_TOP_LEVEL_REF_IN_SCRIPT, profile_name: "patina.script_rule.no_top_level_ref_in_script", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoTopLevelRefInScript },
    BuiltinScriptRuleEntry { rule_name: RULE_PREFER_REF_OVER_REACTIVE, profile_name: "patina.script_rule.prefer_ref_over_reactive", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &PreferRefOverReactive },
    BuiltinScriptRuleEntry { rule_name: RULE_PREFER_USE_TEMPLATE_REF, profile_name: "patina.script_rule.prefer_use_template_ref", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &PreferUseTemplateRef },
    BuiltinScriptRuleEntry { rule_name: RULE_PREFER_USE_SLOTS, profile_name: "patina.script_rule.prefer_use_slots", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &PreferUseSlots },
    BuiltinScriptRuleEntry { rule_name: RULE_PREFER_USE_ATTRS, profile_name: "patina.script_rule.prefer_use_attrs", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &PreferUseAttrs },
    BuiltinScriptRuleEntry { rule_name: RULE_PREFER_USE_ID, profile_name: "patina.script_rule.prefer_use_id", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &PreferUseId },
    BuiltinScriptRuleEntry { rule_name: RULE_PREFER_IMPORT_FROM_VUE, profile_name: "patina.script_rule.prefer_import_from_vue", category: "Script", fixable: true, presets: OPT_IN_SCRIPT_PRESETS, rule: &PreferImportFromVue },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_WITH_DEFAULTS, profile_name: "patina.script_rule.no_with_defaults", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoWithDefaults },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_DEEP_DESTRUCTURE_IN_PROPS, profile_name: "patina.script_rule.no_deep_destructure_in_props", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NO_DEEP_DESTRUCTURE_IN_PROPS_RULE },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_INTERNAL_IMPORTS, profile_name: "patina.script_rule.no_internal_imports", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoInternalImports },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_IMPORT_COMPILER_MACROS, profile_name: "patina.script_rule.no_import_compiler_macros", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoImportCompilerMacros },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_RESERVED_IDENTIFIERS, profile_name: "patina.script_rule.no_reserved_identifiers", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoReservedIdentifiers },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_RESERVED_KEYS, profile_name: "patina.script_rule.no_reserved_keys", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoReservedKeys },
    BuiltinScriptRuleEntry { rule_name: RULE_REQUIRE_SYMBOL_PROVIDE, profile_name: "patina.script_rule.require_symbol_provide", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &RequireSymbolProvide },
    BuiltinScriptRuleEntry { rule_name: RULE_REQUIRE_FUNCTION_RETURN_TYPE, profile_name: "patina.script_rule.require_function_return_type", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &RequireFunctionReturnType },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_DUPE_KEYS, profile_name: "patina.script_rule.no_dupe_keys", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoDupeKeys },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_SIDE_EFFECTS_IN_COMPUTED, profile_name: "patina.script_rule.no_side_effects_in_computed", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoSideEffectsInComputed },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_ARROW_FUNCTIONS_IN_WATCH, profile_name: "patina.script_rule.no_arrow_functions_in_watch", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoArrowFunctionsInWatch },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_EXPORT_IN_SCRIPT_SETUP, profile_name: "patina.script_rule.no_export_in_script_setup", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoExportInScriptSetup },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API, profile_name: "patina.script_rule.no_deprecated_dollar_listeners_api", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoDeprecatedDollarListenersApi },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO, profile_name: "patina.script_rule.no_potential_component_option_typo", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoPotentialComponentOptionTypo },
    BuiltinScriptRuleEntry { rule_name: RULE_RETURN_IN_COMPUTED_PROPERTY, profile_name: "patina.script_rule.return_in_computed_property", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &ReturnInComputedProperty },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API, profile_name: "patina.script_rule.no_deprecated_dollar_scopedslots_api", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoDeprecatedDollarScopedSlotsApi },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION, profile_name: "patina.script_rule.no_deprecated_data_object_declaration", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoDeprecatedDataObjectDeclaration },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_DEPRECATED_EVENTS_API, profile_name: "patina.script_rule.no_deprecated_events_api", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoDeprecatedEventsApi },
    BuiltinScriptRuleEntry { rule_name: RULE_COMPONENT_OPTIONS_NAME_CASING, profile_name: "patina.script_rule.component_options_name_casing", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &ComponentOptionsNameCasing },
    BuiltinScriptRuleEntry { rule_name: RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR, profile_name: "patina.script_rule.require_prop_type_constructor", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &RequirePropTypeConstructor },
    BuiltinScriptRuleEntry { rule_name: RULE_DEFINE_MACROS_ORDER, profile_name: "patina.script_rule.define_macros_order", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &DefineMacrosOrder },
    BuiltinScriptRuleEntry { rule_name: RULE_DEFINE_EMITS_DECLARATION, profile_name: "patina.script_rule.define_emits_declaration", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &DefineEmitsDeclaration },
    BuiltinScriptRuleEntry { rule_name: RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD, profile_name: "patina.script_rule.no_use_computed_property_like_method", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &NoUseComputedPropertyLikeMethod },
    BuiltinScriptRuleEntry { rule_name: RULE_DEFINE_PROPS_DECLARATION, profile_name: "patina.script_rule.define_props_declaration", category: "Script", fixable: false, presets: OPINIONATED_SCRIPT_PRESETS, rule: &DefinePropsDeclaration },
];
