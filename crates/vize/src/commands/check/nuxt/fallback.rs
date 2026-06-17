//! Built-in fallback stubs and `nuxt.config` module-driven fallbacks.
//!
//! Module detection parses the `modules` array out of the default-exported
//! config object with OXC instead of substring-matching the raw source, so
//! commented-out entries and module names inside unrelated strings no longer
//! count as installed modules.

mod fallback_values;

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, ToCompactString};

use super::parsing::{
    default_export_config_object, extract_expression, find_object_property, nuxt_config_source,
};
use super::stubs::{
    declared_name, push_generic_function_stub, push_named_overload_stubs, push_stub,
};
use fallback_values::fallback_value_stubs;

pub(super) fn collect_fallback_stubs(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    has_generated_imports: bool,
) {
    let mut fallback_names = FxHashSet::default();
    push_fallback_stub_group(
        fallback_type_alias_stubs(),
        stubs,
        seen_names,
        &mut fallback_names,
    );
    // The hardcoded `any` ladder silently weakens checking, so only inject it
    // when the project has no generated Nuxt import manifest to rely on.
    if !has_generated_imports {
        push_fallback_stub_group(
            fallback_value_stubs(),
            stubs,
            seen_names,
            &mut fallback_names,
        );
    }
    // Built-in component consts stay name-deduped against the generated
    // `GlobalComponents` members: `.nuxt/components.d.ts` may omit built-ins,
    // and dropping these would newly error every `<NuxtLink>` reference.
    push_fallback_stub_group(
        fallback_component_stubs(),
        stubs,
        seen_names,
        &mut fallback_names,
    );
}

fn push_fallback_stub_group(
    group: Vec<String>,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    fallback_names: &mut FxHashSet<String>,
) {
    for stub in group {
        if let Some(name) = declared_name(&stub) {
            let name = name.to_compact_string();
            if fallback_names.contains(name.as_str()) {
                stubs.push(stub);
                continue;
            }
            if seen_names.insert(name.clone()) {
                fallback_names.insert(name);
                stubs.push(stub);
            }
            continue;
        }
        stubs.push(stub);
    }
}

pub(super) fn collect_module_fallback_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let config_source = nuxt_config_source(cwd);
    if config_source.is_empty() {
        return;
    }
    let modules = parse_nuxt_config_modules(&config_source);

    if modules.might_include(&["@nuxtjs/i18n", "@nuxt/i18n"]) {
        push_stub(
            stubs,
            seen_names,
            "declare function useI18n(): ({ locale: Ref<string>; locales: Ref<Array<{ code: string; name?: string; dir?: any }>>; setLocale: (locale: string) => any; t: (...args: any[]) => any } & Record<string, any>);"
                .into(),
        );
        push_generic_function_stub(stubs, seen_names, "useLocalePath");
        push_generic_function_stub(stubs, seen_names, "useLocaleHead");
        push_generic_function_stub(stubs, seen_names, "$t");
        stubs.push("declare module \"@nuxtjs/i18n\" { export type Directions = any; }".into());
    }

    if modules.might_include(&["@nuxt/content"]) {
        push_stub(
            stubs,
            seen_names,
            "declare function queryCollection<T = any>(collection: string): ({ all(): Promise<T[]>; first(): Promise<T | null>; path(path: string): any; where(...args: any[]): any; order(...args: any[]): any; limit(...args: any[]): any } & Record<string, any>);"
                .into(),
        );
    }

    if modules.might_include(&["@vueuse/nuxt"]) {
        for name in ["useClipboard", "usePreferredDark", "useScrollLock"] {
            push_generic_function_stub(stubs, seen_names, name);
        }
        push_named_overload_stubs(
            stubs,
            seen_names,
            "onKeyStroke",
            vec![
                "declare function onKeyStroke(key: string | string[], handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
                "declare function onKeyStroke(predicate: (event: KeyboardEvent) => any, handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
                "declare function onKeyStroke(handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
            ],
        );
        stubs.push(
            "declare module \"@vueuse/integrations/useFocusTrap\" { export function useFocusTrap(...args: any[]): any; }"
                .into(),
        );
    }

    if modules.might_include(&["@nuxtjs/color-mode"]) {
        push_stub(
            stubs,
            seen_names,
            "declare function useColorMode(): ({ preference: 'system' | 'light' | 'dark'; value: any } & Record<string, any>);"
                .into(),
        );
    }

    if modules.might_include(&["nuxt-og-image"]) {
        push_generic_function_stub(stubs, seen_names, "defineOgImageComponent");
    }

    if modules.might_include(&["motion-v/nuxt"]) {
        stubs.push(
            "declare module \"motion-v\" { export const motion: Record<string, any>; export const AnimatePresence: any; }"
                .into(),
        );
    }
}

/// Statically-resolved view of the `modules` array in `nuxt.config`.
#[derive(Debug, Default)]
pub(super) struct NuxtConfigModules {
    /// Module names found as static entries: string literals,
    /// no-substitution template literals, and the first element of
    /// `['module-name', { ...options }]` tuples.
    names: FxHashSet<String>,
    /// Set when the module list cannot be fully resolved statically: spread
    /// or computed entries, a non-literal tuple head, a non-array `modules`
    /// value, or a default export we cannot see through (identifier
    /// reference, unknown wrapper call, CommonJS config).
    has_unresolved_entries: bool,
}

impl NuxtConfigModules {
    fn unresolved() -> Self {
        Self {
            names: FxHashSet::default(),
            has_unresolved_entries: true,
        }
    }

    fn insert(&mut self, name: &str) {
        self.names.insert(name.to_compact_string());
    }

    /// Conservative membership test: an unresolved entry may name any module,
    /// so it counts as a potential match for every candidate. Missing a
    /// module here would surface false "undefined name" diagnostics, while
    /// over-matching only injects a few extra `any` stubs on the (already
    /// `.nuxt`-less) fallback path.
    pub(super) fn might_include(&self, candidates: &[&str]) -> bool {
        self.has_unresolved_entries
            || candidates
                .iter()
                .any(|candidate| self.names.contains(*candidate))
    }
}

/// Parses `nuxt.config` source and extracts the `modules` array from the
/// default-exported config object, handling both
/// `export default defineNuxtConfig({ ... })` and `export default { ... }`.
pub(super) fn parse_nuxt_config_modules(config_source: &str) -> NuxtConfigModules {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);
    let ret = Parser::new(&allocator, config_source, source_type).parse();
    if ret.panicked {
        return NuxtConfigModules::unresolved();
    }

    let Some(config_object) = default_export_config_object(&ret.program.body) else {
        return NuxtConfigModules::unresolved();
    };

    let Some(modules_value) = find_object_property(config_object, "modules") else {
        // A statically-visible config without a `modules` key registers no
        // modules, so nothing needs stubbing.
        return NuxtConfigModules::default();
    };

    let Some(Expression::ArrayExpression(modules)) = extract_expression(modules_value) else {
        return NuxtConfigModules::unresolved();
    };

    let mut resolved = NuxtConfigModules::default();
    for element in &modules.elements {
        match element {
            ArrayExpressionElement::Elision(_) => {}
            ArrayExpressionElement::SpreadElement(_) => resolved.has_unresolved_entries = true,
            _ => match element.as_expression().and_then(extract_expression) {
                Some(Expression::StringLiteral(literal)) => resolved.insert(&literal.value),
                Some(Expression::TemplateLiteral(template)) => match template.single_quasi() {
                    Some(name) => resolved.insert(&name),
                    None => resolved.has_unresolved_entries = true,
                },
                // `['module-name', { ...options }]` tuple form.
                Some(Expression::ArrayExpression(tuple)) => {
                    match tuple
                        .elements
                        .first()
                        .and_then(ArrayExpressionElement::as_expression)
                        .and_then(extract_expression)
                    {
                        Some(Expression::StringLiteral(literal)) => resolved.insert(&literal.value),
                        _ => resolved.has_unresolved_entries = true,
                    }
                }
                _ => resolved.has_unresolved_entries = true,
            },
        }
    }
    resolved
}

#[cfg(test)]
pub(super) fn fallback_stub_strings() -> Vec<String> {
    let mut stubs = fallback_type_alias_stubs();
    stubs.extend(fallback_value_stubs());
    stubs.extend(fallback_component_stubs());
    stubs
}

/// Type aliases for auto-imported Vue types. Generated `.nuxt` artifacts are
/// only mined for `declare global` consts and `GlobalComponents` members, so
/// these aliases have no generated counterpart and are always injected.
fn fallback_type_alias_stubs() -> Vec<String> {
    vec![
        "type Composer = any;".into(),
        "type Ref<T = any> = import('vue').Ref<T>;".into(),
        "type ComputedRef<T = any> = import('vue').ComputedRef<T>;".into(),
        "type WritableComputedRef<T = any> = import('vue').WritableComputedRef<T>;".into(),
        "type ShallowRef<T = any> = import('vue').ShallowRef<T>;".into(),
        "type UnwrapRef<T> = import('vue').UnwrapRef<T>;".into(),
        "type UnwrapNestedRefs<T> = import('vue').UnwrapNestedRefs<T>;".into(),
        "type MaybeRef<T = any> = import('vue').MaybeRef<T>;".into(),
        "type MaybeRefOrGetter<T = any> = import('vue').MaybeRefOrGetter<T>;".into(),
        "type Component = import('vue').Component;".into(),
    ]
}

/// Nuxt built-in component consts. Always injected (name-deduped against the
/// generated `GlobalComponents` members) because `.nuxt/components.d.ts` may
/// legitimately omit built-ins.
fn fallback_component_stubs() -> Vec<String> {
    vec![
        "declare const NuxtLink: any;".into(),
        "declare const NuxtPage: any;".into(),
        "declare const NuxtLayout: any;".into(),
        "declare const NuxtLoadingIndicator: any;".into(),
        "declare const NuxtErrorBoundary: any;".into(),
        "declare const NuxtWelcome: any;".into(),
        "declare const NuxtIsland: any;".into(),
        "declare const NuxtRouteAnnouncer: any;".into(),
        "declare const ClientOnly: any;".into(),
        "declare const DevOnly: any;".into(),
    ]
}
