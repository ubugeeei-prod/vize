//! Modules whose Composition API exports eslint-plugin-vue's
//! `createCompositionApiTraceMap` follows: `vue`, `@vue/composition-api`,
//! `@nuxtjs/composition-api` and unimport's `#imports`.

/// Whether `source` exports Vue's Composition API.
pub(crate) fn is_composition_api_module(source: &str) -> bool {
    matches!(
        source,
        "vue" | "@vue/composition-api" | "@nuxtjs/composition-api" | "#imports"
    )
}

/// Whether `source` re-exports Vue's own `ref` / `shallowRef` from its published
/// entry: Vue's internal packages, and libraries whose entry is
/// `export * from 'vue'` or `export * from '@vue/runtime-core'`.
///
/// Only `script/prefer-use-template-ref` widens its factory identity with
/// these, as a suggestion rule. Rules that report at error severity keep
/// [`is_composition_api_module`].
pub(crate) fn reexports_vue_ref_factories(source: &str) -> bool {
    matches!(
        source,
        "@vue/runtime-core"
            | "@vue/runtime-dom"
            | "@vue/reactivity"
            | "vue-demi"
            | "vue-termui"
            | "nativescript-vue"
    )
}
