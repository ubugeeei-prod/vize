//! eslint-plugin-vue rule IDs and the Patina rules they map to under another
//! name. An `eslint-disable` comment naming the upstream rule suppresses the
//! Patina rule, so a migrated codebase keeps its existing suppressions. The
//! list is the rule map's renamed `mapped` entries; a test keeps them equal.

pub(super) const ESLINT_VUE_ALIASES: &[(&str, &str)] = &[
    ("vue/attributes-order", "vue/attribute-order"),
    ("vue/block-order", "vue/sfc-element-order"),
    (
        "vue/component-options-name-casing",
        "script/component-options-name-casing",
    ),
    (
        "vue/custom-event-name-casing",
        "script/custom-event-name-casing",
    ),
    (
        "vue/define-emits-declaration",
        "script/define-emits-declaration",
    ),
    ("vue/define-macros-order", "script/define-macros-order"),
    (
        "vue/define-props-declaration",
        "script/define-props-declaration",
    ),
    (
        "vue/define-props-destructuring",
        "script/define-props-destructuring",
    ),
    (
        "vue/no-arrow-functions-in-watch",
        "script/no-arrow-functions-in-watch",
    ),
    (
        "vue/no-async-in-computed-properties",
        "script/no-async-in-computed",
    ),
    ("vue/no-boolean-default", "script/no-boolean-default"),
    (
        "vue/no-computed-properties-in-data",
        "script/no-computed-properties-in-data",
    ),
    (
        "vue/no-deprecated-data-object-declaration",
        "script/no-deprecated-data-object-declaration",
    ),
    (
        "vue/no-deprecated-delete-set",
        "script/no-deprecated-delete-set",
    ),
    (
        "vue/no-deprecated-destroyed-lifecycle",
        "script/no-deprecated-destroyed-lifecycle",
    ),
    (
        "vue/no-deprecated-dollar-listeners-api",
        "script/no-deprecated-dollar-listeners-api",
    ),
    (
        "vue/no-deprecated-dollar-scopedslots-api",
        "script/no-deprecated-dollar-scopedslots-api",
    ),
    (
        "vue/no-deprecated-events-api",
        "script/no-deprecated-events-api",
    ),
    (
        "vue/no-deprecated-model-definition",
        "script/no-deprecated-model-definition",
    ),
    (
        "vue/no-deprecated-props-default-this",
        "script/no-deprecated-props-default-this",
    ),
    (
        "vue/no-deprecated-vue-config-keycodes",
        "script/no-deprecated-vue-config-keycodes",
    ),
    ("vue/no-dupe-keys", "script/no-dupe-keys"),
    (
        "vue/no-duplicate-attr-inheritance",
        "script/no-duplicate-attr-inheritance",
    ),
    ("vue/no-duplicate-class-names", "html/no-duplicate-class"),
    (
        "vue/no-export-in-script-setup",
        "script/no-export-in-script-setup",
    ),
    ("vue/no-expose-after-await", "script/no-expose-after-await"),
    (
        "vue/no-import-compiler-macros",
        "script/no-import-compiler-macros",
    ),
    (
        "vue/no-lifecycle-after-await",
        "script/no-lifecycle-after-await",
    ),
    ("vue/no-multiple-slot-args", "script/no-multiple-slot-args"),
    ("vue/no-parsing-error", "parser/template"),
    (
        "vue/no-potential-component-option-typo",
        "script/no-potential-component-option-typo",
    ),
    ("vue/no-ref-as-operand", "script/no-ref-as-operand"),
    (
        "vue/no-required-prop-with-default",
        "script/no-required-prop-with-default",
    ),
    ("vue/no-reserved-keys", "script/no-reserved-keys"),
    ("vue/no-reserved-props", "script/no-reserved-props"),
    // Not in the rule map: Vue 2's name for the data-object rule.
    (
        "vue/no-shared-component-data",
        "script/no-deprecated-data-object-declaration",
    ),
    (
        "vue/no-side-effects-in-computed-properties",
        "script/no-side-effects-in-computed-properties",
    ),
    (
        "vue/no-undef-components",
        "vue/require-component-registration",
    ),
    (
        "vue/no-unused-emit-declarations",
        "script/no-unused-emit-declarations",
    ),
    (
        "vue/no-use-computed-property-like-method",
        "script/no-use-computed-property-like-method",
    ),
    ("vue/no-watch-after-await", "script/no-watch-after-await"),
    (
        "vue/one-component-per-file",
        "script/one-component-per-file",
    ),
    ("vue/order-in-components", "script/order-in-components"),
    ("vue/prefer-define-options", "script/prefer-define-options"),
    (
        "vue/prefer-import-from-vue",
        "script/prefer-import-from-vue",
    ),
    (
        "vue/prefer-separate-static-class",
        "vapor/prefer-static-class",
    ),
    (
        "vue/prefer-use-template-ref",
        "script/prefer-use-template-ref",
    ),
    ("vue/require-default-prop", "script/require-default-prop"),
    (
        "vue/require-explicit-emits",
        "script/require-explicit-emits",
    ),
    (
        "vue/require-explicit-slots",
        "script/require-explicit-slots",
    ),
    (
        "vue/require-prop-type-constructor",
        "script/require-prop-type-constructor",
    ),
    ("vue/require-prop-types", "script/require-prop-types"),
    ("vue/require-render-return", "script/require-render-return"),
    (
        "vue/require-slots-as-functions",
        "script/require-slots-as-functions",
    ),
    (
        "vue/require-typed-object-prop",
        "script/require-typed-object-prop",
    ),
    ("vue/require-typed-ref", "script/require-typed-ref"),
    (
        "vue/require-valid-default-prop",
        "script/require-valid-default-prop",
    ),
    (
        "vue/return-in-computed-property",
        "script/return-in-computed-property",
    ),
    (
        "vue/return-in-emits-validator",
        "script/return-in-emits-validator",
    ),
    ("vue/valid-define-emits", "script/valid-define-emits"),
    ("vue/valid-define-options", "script/valid-define-options"),
    ("vue/valid-define-props", "script/valid-define-props"),
    ("vue/valid-next-tick", "script/valid-next-tick"),
];

/// The Patina rule an eslint-plugin-vue rule ID suppresses.
pub(super) fn patina_rule(upstream: &str) -> &str {
    ESLINT_VUE_ALIASES
        .binary_search_by(|(name, _)| name.cmp(&upstream))
        .ok()
        .and_then(|index| ESLINT_VUE_ALIASES.get(index))
        .map_or(upstream, |(_, patina)| patina)
}

#[cfg(test)]
mod tests;
