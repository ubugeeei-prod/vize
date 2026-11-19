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
