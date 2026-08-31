//! Template-scope ref unwrapping for framework auto-import bindings.
//!
//! A provider such as Nuxt fills [`VirtualTsOptions::auto_import_stubs`] with
//! module-scope ambient declarations (`declare const currentUser: typeof
//! import('~/composables/users')['currentUser'];`). In a real build the same
//! name is rewritten by the auto-import transform into a `<script setup>`
//! import, so Vue's template compiler sees an ordinary setup binding and
//! `unref`s it. Canon must therefore route those names through the very same
//! `__U` template shadow the SFC's own bindings use; without it
//! `currentUser.account` in a template is typed against
//! `ComputedRef<UserLogin | undefined>` and reports a `TS2339` `vue-tsc` never
//! produces (#4146).
//!
//! Every name returned here is guaranteed to have a module-scope declaration in
//! the generated file, because the filters below mirror
//! [`super::super::auto_import_stubs::emit_auto_import_stubs`] exactly. That
//! guarantee is load-bearing: a `typeof` capture of an *undeclared* name would
//! report `TS2304` at a generated position and silently turn the authored
//! "Cannot find name" diagnostic for an unknown template identifier into
//! `any`, trading a false positive for a false negative.

use vize_croquis::Croquis;
use vize_s0::{FxHashSet, String};

use crate::virtual_ts::types::VirtualTsOptions;

/// Auto-import binding names the template references and that no SFC-local
/// binding already provides.
///
/// `imported_names` is the module-level import set the generator already
/// computed for `emit_auto_import_stubs`; `options_api_setup_binding_names`
/// are the Options API `setup()` return bindings that own their own shadow.
///
/// `structural_unwrap` is the Vue 2/2.7 dialect flag. That dialect's `__U`
/// helper is the purely structural `T extends { value: infer V } ? V : T`,
/// which cannot tell a ref from a plain `{ text, value }` option constant
/// (#3767). Canon keeps auto-imports out of it rather than re-introducing that
/// false positive on a binding it has no declaration site to classify.
pub(super) fn collect(
    summary: &Croquis,
    options: &VirtualTsOptions,
    imported_names: &FxHashSet<&str>,
    options_api_setup_binding_names: &FxHashSet<&str>,
    template_referenced_names: &FxHashSet<String>,
    structural_unwrap: bool,
) -> Vec<String> {
    if structural_unwrap {
        return Vec::new();
    }

    let reserved = reserved_template_names(options);
    let mut names: Vec<String> = options
        .auto_import_binding_names()
        .into_iter()
        .filter(|name| template_referenced_names.contains(name.as_str()))
        .filter(|name| !summary.bindings.bindings.contains_key(name.as_str()))
        .filter(|name| !summary.used_components.contains(name.as_str()))
        .filter(|name| !reserved.contains(name.as_str()))
        // An Options API `setup()` return spread infers its bindings from the
        // template, so it claims names `summary.bindings` never holds. It
        // already emits a `__R_<name>` capture and a `__U` shadow for each of
        // them; a second pair here would be a `TS2451`.
        .filter(|name| !options_api_setup_binding_names.contains(name.as_str()))
        // A plain `<script>` next to `<script setup>` keeps its imports out of
        // `summary.bindings`, and `emit_auto_import_stubs` skips a stub whose
        // name one of them already declares. Mirror that so a shadow is only
        // ever emitted for a name the generated module really declares
        // ambiently.
        .filter(|name| !imported_names.contains(name.as_str()))
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Names that already own a declaration inside `__template()`: the configured
/// template globals (`$t`, `$route`, CSS modules) and the component bindings
/// that are deliberately left unshadowed so their real props keep checking.
/// Re-declaring any of them as an unwrap shadow would be a `TS2451`.
fn reserved_template_names(options: &VirtualTsOptions) -> FxHashSet<&str> {
    options
        .template_globals
        .iter()
        .map(|global| global.name.as_str())
        .chain(options.css_modules.iter().map(|module| module.as_str()))
        .chain(
            options
                .external_template_bindings
                .iter()
                .map(|binding| binding.as_str()),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use vize_croquis::Croquis;
    use vize_s0::{FxHashSet, String};

    use crate::virtual_ts::types::VirtualTsOptions;

    const REF_STUB: &str =
        "declare const currentUser: typeof import('./composables')['currentUser'];";

    fn collect_with(
        imported_names: &[&'static str],
        options_api_setup_binding_names: &[&'static str],
    ) -> Vec<String> {
        let options = VirtualTsOptions {
            auto_import_stubs: vec![REF_STUB.into()],
            ..Default::default()
        };
        let referenced: FxHashSet<String> = ["currentUser".into()].into_iter().collect();
        super::collect(
            &Croquis::new(),
            &options,
            &imported_names.iter().copied().collect(),
            &options_api_setup_binding_names.iter().copied().collect(),
            &referenced,
            false,
        )
    }

    #[test]
    fn a_referenced_auto_import_is_a_candidate() {
        assert_eq!(collect_with(&[], &[]), vec!["currentUser"]);
    }

    #[test]
    fn a_name_a_module_level_import_already_declares_is_declined() {
        // `emit_auto_import_stubs` skips the stub for such a name, so shadowing
        // it here would capture `typeof` an undeclared name.
        assert!(collect_with(&["currentUser"], &[]).is_empty());
    }

    #[test]
    fn a_name_the_options_api_setup_spread_owns_is_declined() {
        // The spread emits its own `__R_currentUser` capture and `__U` shadow;
        // a second pair would be a `TS2451`.
        assert!(collect_with(&[], &["currentUser"]).is_empty());
    }

    fn names(stub: &str) -> Vec<vize_s0::String> {
        VirtualTsOptions {
            auto_import_stubs: vec![stub.into()],
            ..Default::default()
        }
        .auto_import_binding_names()
    }

    #[test]
    fn typed_const_stubs_are_unwrap_candidates() {
        assert_eq!(
            names("declare const currentUser: typeof import('~/x')['currentUser'];"),
            vec!["currentUser"]
        );
        assert_eq!(
            names("declare let counter: import('vue').Ref<number>;"),
            vec!["counter"]
        );
    }

    #[test]
    fn function_any_dollar_and_type_stubs_are_declined() {
        for stub in [
            "declare function useFoo<T = any>(...args: any[]): any;",
            "declare const useAsyncData: any;",
            "declare const $fetch: typeof import('ofetch')['$fetch'];",
            "type NuxtApp = any;",
        ] {
            assert!(names(stub).is_empty(), "{stub}");
        }
    }
}
