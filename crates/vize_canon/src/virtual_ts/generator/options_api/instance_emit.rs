//! A template `$emit` in an Options API component reads its own instance.
//!
//! Vue resolves a template `$emit` against the rendering component's public
//! instance, and types that instance's `$emit` from the component's `emits`
//! option: an array names the events, an object also types each payload
//! through its validator. The template context otherwise declares the generic
//! `ComponentPublicInstance['$emit']`, which accepts any event with any
//! payload, so an Options API template reported neither an undeclared event
//! nor a wrong payload where `vue-tsc` reports both.
//!
//! The authored instance owns the template only for a Vue 3 Options API
//! default export without `<script setup>` (the same owner the template's
//! other instance members resolve against), and only a template that names
//! `$emit` reads it, so every other generated module keeps its bytes. An
//! `any` instance (Vue gave up on the options) keeps `$emit` unchecked.

use vize_carton::config::VueVersion;
use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::VirtualTsGenerationOptions;

/// The type of the template's `$emit`, read off the authored default export.
pub(in crate::virtual_ts::generator) const INSTANCE_EMIT_TYPE: &str = "(typeof __default__ extends abstract new (...args: any) => infer __I ? $IsAny<__I> extends true ? any : __I extends { $emit: infer __E } ? __E : __Ctx['$emit'] : __Ctx['$emit'])";

/// Whether the template's `$emit` is the authored Options API instance's.
pub(in crate::virtual_ts::generator) fn reads_instance_emit(
    summary: &Croquis,
    options: VirtualTsGenerationOptions<'_>,
) -> bool {
    options.options_api
        && !options.legacy_vue2
        && !matches!(options.dialect, VueVersion::V2 | VueVersion::V2_7)
        && summary.options_descriptor.is_some()
        && !summary
            .scopes
            .iter()
            .any(|scope| scope.kind == ScopeKind::ScriptSetup)
        && summary
            .template_expressions
            .iter()
            .any(|expression| expression.content.as_str().contains("$emit"))
}
