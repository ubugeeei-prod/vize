//! Stable registry names for the built-in script rules.
//!
//! Each rule has a `RULE_*` identifier and an entry in the ordered name lists.
//! When adding a rule, add its `RULE_*` const here and append the name to
//! [`ALL_BUILTIN_SCRIPT_RULE_NAMES`] (and, for non-always-on rules, to the
//! test-only [`OPT_IN_SCRIPT_RULE_NAMES`]), matching the entry order used in
//! `rules.rs`.

pub(crate) const RULE_NO_OPTIONS_API: &str = "script/no-options-api";
pub(crate) const RULE_NO_GET_CURRENT_INSTANCE: &str = "script/no-get-current-instance";
pub(crate) const RULE_NO_NEXT_TICK: &str = "script/no-next-tick";
pub(crate) const RULE_PINIA_PREFER_STORE_TO_REFS: &str = "ecosystem/pinia-prefer-store-to-refs";
pub(crate) const RULE_VUE_ROUTER_PREFER_NAMED_PUSH: &str = "ecosystem/vue-router-prefer-named-push";
pub(crate) const RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT: &str =
    "ecosystem/vue-test-utils-no-html-snapshot";
pub(crate) const RULE_PREFER_COMPUTED: &str = "script/prefer-computed";
pub(crate) const RULE_NO_ASYNC_IN_COMPUTED: &str = "script/no-async-in-computed";
pub(crate) const RULE_NO_REACTIVE_DESTRUCTURE: &str = "script/no-reactive-destructure";
pub(crate) const RULE_NO_TOP_LEVEL_REF_IN_SCRIPT: &str = "script/no-top-level-ref-in-script";
pub(crate) const RULE_PREFER_REF_OVER_REACTIVE: &str = "script/prefer-ref-over-reactive";
pub(crate) const RULE_PREFER_USE_TEMPLATE_REF: &str = "script/prefer-use-template-ref";
pub(crate) const RULE_PREFER_USE_SLOTS: &str = "script/prefer-use-slots";
pub(crate) const RULE_PREFER_USE_ATTRS: &str = "script/prefer-use-attrs";
pub(crate) const RULE_PREFER_USE_ID: &str = "script/prefer-use-id";
pub(crate) const RULE_PREFER_IMPORT_FROM_VUE: &str = "script/prefer-import-from-vue";
pub(crate) const RULE_NO_WITH_DEFAULTS: &str = "script/no-with-defaults";
pub(crate) const RULE_NO_DEEP_DESTRUCTURE_IN_PROPS: &str = "script/no-deep-destructure-in-props";
pub(crate) const RULE_NO_INTERNAL_IMPORTS: &str = "script/no-internal-imports";
pub(crate) const RULE_NO_IMPORT_COMPILER_MACROS: &str = "script/no-import-compiler-macros";
pub(crate) const RULE_NO_RESERVED_IDENTIFIERS: &str = "script/no-reserved-identifiers";
pub(crate) const RULE_NO_RESERVED_KEYS: &str = "script/no-reserved-keys";
pub(crate) const RULE_REQUIRE_SYMBOL_PROVIDE: &str = "script/require-symbol-provide";
pub(crate) const RULE_REQUIRE_FUNCTION_RETURN_TYPE: &str = "script/require-function-return-type";
pub(crate) const RULE_NO_DUPE_KEYS: &str = "script/no-dupe-keys";
pub(crate) const RULE_NO_SIDE_EFFECTS_IN_COMPUTED: &str =
    "script/no-side-effects-in-computed-properties";
pub(crate) const RULE_NO_ARROW_FUNCTIONS_IN_WATCH: &str = "script/no-arrow-functions-in-watch";
pub(crate) const RULE_NO_EXPORT_IN_SCRIPT_SETUP: &str = "script/no-export-in-script-setup";
pub(crate) const RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API: &str =
    "script/no-deprecated-dollar-listeners-api";
pub(crate) const RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO: &str =
    "script/no-potential-component-option-typo";
pub(crate) const RULE_RETURN_IN_COMPUTED_PROPERTY: &str = "script/return-in-computed-property";
pub(crate) const RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API: &str =
    "script/no-deprecated-dollar-scopedslots-api";
pub(crate) const RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION: &str =
    "script/no-deprecated-data-object-declaration";
pub(crate) const RULE_NO_DEPRECATED_EVENTS_API: &str = "script/no-deprecated-events-api";
pub(crate) const RULE_COMPONENT_OPTIONS_NAME_CASING: &str = "script/component-options-name-casing";
pub(crate) const RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR: &str = "script/require-prop-type-constructor";
pub(crate) const RULE_DEFINE_MACROS_ORDER: &str = "script/define-macros-order";
pub(crate) const RULE_DEFINE_EMITS_DECLARATION: &str = "script/define-emits-declaration";

pub(in crate::linter::script_rules) const ALL_BUILTIN_SCRIPT_RULE_NAMES: &[&str] = &[
    RULE_NO_OPTIONS_API,
    RULE_NO_GET_CURRENT_INSTANCE,
    RULE_NO_NEXT_TICK,
    RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
    RULE_PREFER_COMPUTED,
    RULE_NO_ASYNC_IN_COMPUTED,
    RULE_NO_REACTIVE_DESTRUCTURE,
    RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
    RULE_PREFER_REF_OVER_REACTIVE,
    RULE_PREFER_USE_TEMPLATE_REF,
    RULE_PREFER_USE_SLOTS,
    RULE_PREFER_USE_ATTRS,
    RULE_PREFER_USE_ID,
    RULE_PREFER_IMPORT_FROM_VUE,
    RULE_NO_WITH_DEFAULTS,
    RULE_NO_DEEP_DESTRUCTURE_IN_PROPS,
    RULE_NO_INTERNAL_IMPORTS,
    RULE_NO_IMPORT_COMPILER_MACROS,
    RULE_NO_RESERVED_IDENTIFIERS,
    RULE_NO_RESERVED_KEYS,
    RULE_REQUIRE_SYMBOL_PROVIDE,
    RULE_REQUIRE_FUNCTION_RETURN_TYPE,
    RULE_NO_DUPE_KEYS,
    RULE_NO_SIDE_EFFECTS_IN_COMPUTED,
    RULE_NO_ARROW_FUNCTIONS_IN_WATCH,
    RULE_NO_EXPORT_IN_SCRIPT_SETUP,
    RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API,
    RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO,
    RULE_RETURN_IN_COMPUTED_PROPERTY,
    RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API,
    RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION,
    RULE_NO_DEPRECATED_EVENTS_API,
    RULE_COMPONENT_OPTIONS_NAME_CASING,
    RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR,
    RULE_DEFINE_MACROS_ORDER,
    RULE_DEFINE_EMITS_DECLARATION,
];

#[cfg(test)]
pub(in crate::linter::script_rules) const OPT_IN_SCRIPT_RULE_NAMES: &[&str] = &[
    RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
    RULE_PREFER_COMPUTED,
    RULE_NO_ASYNC_IN_COMPUTED,
    RULE_NO_REACTIVE_DESTRUCTURE,
    RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
    RULE_PREFER_REF_OVER_REACTIVE,
    RULE_PREFER_USE_TEMPLATE_REF,
    RULE_PREFER_USE_SLOTS,
    RULE_PREFER_USE_ATTRS,
    RULE_PREFER_USE_ID,
    RULE_PREFER_IMPORT_FROM_VUE,
    RULE_NO_WITH_DEFAULTS,
    RULE_NO_DEEP_DESTRUCTURE_IN_PROPS,
    RULE_NO_INTERNAL_IMPORTS,
    RULE_NO_IMPORT_COMPILER_MACROS,
    RULE_NO_RESERVED_IDENTIFIERS,
    RULE_NO_RESERVED_KEYS,
    RULE_REQUIRE_SYMBOL_PROVIDE,
    RULE_REQUIRE_FUNCTION_RETURN_TYPE,
    RULE_NO_DUPE_KEYS,
    RULE_NO_SIDE_EFFECTS_IN_COMPUTED,
    RULE_NO_ARROW_FUNCTIONS_IN_WATCH,
    RULE_NO_EXPORT_IN_SCRIPT_SETUP,
    RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API,
    RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO,
    RULE_RETURN_IN_COMPUTED_PROPERTY,
    RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API,
    RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION,
    RULE_NO_DEPRECATED_EVENTS_API,
    RULE_COMPONENT_OPTIONS_NAME_CASING,
    RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR,
    RULE_DEFINE_MACROS_ORDER,
    RULE_DEFINE_EMITS_DECLARATION,
];
