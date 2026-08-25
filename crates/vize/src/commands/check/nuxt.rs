//! Nuxt-specific auto-import and plugin injection helpers.

#![allow(clippy::disallowed_macros)]

use std::path::Path;

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_s0::{FxHashSet, String, ToCompactString, config::VueVersion};

mod fallback;
mod generated;
mod generated_ast;
mod generated_dir;
mod generated_values;
mod legacy_template_globals;
mod parsing;
mod plugins;
mod source_scan;
mod stubs;
mod tsconfig_aliases;
mod virtual_modules;

#[cfg(test)]
mod generated_ast_tests;
#[cfg(test)]
mod generated_tests;
#[cfg(test)]
mod legacy_template_globals_tests;
#[cfg(test)]
mod strict_globals_tests;
#[cfg(test)]
mod tests;

use fallback::{collect_fallback_stubs, collect_module_fallback_stubs};
use generated::{collect_generated_stubs, collect_generated_template_globals};
use generated_dir::resolve_nuxt_generated_dir;
use plugins::collect_plugin_injection_stubs;
use source_scan::{collect_source_auto_import_stubs, collect_source_type_auto_import_stubs};
use stubs::{declared_name, is_template_component_binding};
use tsconfig_aliases::collect_explicit_virtual_module_aliases;
use virtual_modules::{collect_fallback_module_stubs, collect_fallback_path_aliases};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::commands::check) struct NuxtPathAlias {
    pub(in crate::commands::check) pattern: String,
    pub(in crate::commands::check) targets: Vec<String>,
}

#[cfg(test)]
pub(in crate::commands::check) fn detect_nuxt_auto_imports(
    options: &mut VirtualTsOptions,
    cwd: &Path,
) -> Vec<NuxtPathAlias> {
    detect(options, cwd, None, false, VueVersion::V3)
}

#[cfg(test)]
pub(in crate::commands::check) fn detect_legacy_nuxt_auto_imports(
    options: &mut VirtualTsOptions,
    cwd: &Path,
) -> Vec<NuxtPathAlias> {
    detect(options, cwd, None, true, VueVersion::V2_7)
}

pub(in crate::commands::check) fn detect(
    options: &mut VirtualTsOptions,
    cwd: &Path,
    tsconfig_path: Option<&Path>,
    legacy_vue2: bool,
    dialect: VueVersion,
) -> Vec<NuxtPathAlias> {
    if !is_nuxt_project_root(cwd) {
        return Vec::new();
    }

    let mut seen_names = FxHashSet::default();
    let mut external_template_bindings = options
        .external_template_bindings
        .iter()
        .cloned()
        .collect::<FxHashSet<_>>();
    for stub in &options.auto_import_stubs {
        if let Some(name) = declared_name(stub) {
            seen_names.insert(name.to_compact_string());
            if is_template_component_binding(name) {
                external_template_bindings.insert(name.to_compact_string());
            }
        }
    }

    let mut collected = Vec::new();
    let generated_dir = resolve_nuxt_generated_dir(cwd);
    let has_generated_imports = collect_generated_stubs(
        cwd,
        &generated_dir,
        &mut collected,
        &mut seen_names,
        &mut external_template_bindings,
    );
    // Surface the degraded fallback: without generated Nuxt types,
    // auto-imports resolve to permissive `any` stubs that hide real type
    // errors. detect_nuxt_auto_imports runs once per check, so this warns once.
    let vue2_dialect = legacy_vue2 || matches!(dialect, VueVersion::V2 | VueVersion::V2_7);
    if let Some(message) =
        missing_generated_types_warning(has_generated_imports, &generated_dir, vue2_dialect)
    {
        eprintln!("{message}");
    }
    collect_plugin_injection_stubs(cwd, &mut collected, &mut seen_names);
    collect_fallback_stubs(&mut collected, &mut seen_names, has_generated_imports);
    if !has_generated_imports {
        collect_module_fallback_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_auto_import_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_type_auto_import_stubs(cwd, &mut collected);
    }
    let explicit_aliases = collect_explicit_virtual_module_aliases(tsconfig_path);
    collect_fallback_module_stubs(cwd, &mut collected, &explicit_aliases);
    let path_aliases = collect_fallback_path_aliases(cwd, &generated_dir);
    if legacy_vue2 {
        legacy_template_globals::collect(options);
    }
    // With Nuxt's generated types present, that graph is the authority on which
    // `$`-prefixed template globals exist: a name it never declares resolves on
    // the component instance, so an undeclared one reports the way the Vue
    // toolchain reports it instead of silently widening to `any`.
    //
    // Without generated types the project is in the documented degraded mode the
    // warning above describes, and the inferred globals stay: turning them into
    // errors there would report names the project never got a chance to declare.
    // The Vue 2 dialect keeps the permissive form too, because its template
    // context is a structural fallback that carries almost none of the real
    // instance surface.
    //
    // Names the generated `ComponentCustomProperties` does declare keep their
    // own declaration either way. Those come from files whose `declare module
    // "vue"` does not always merge into the checked program, so reading them off
    // the instance would invent diagnostics for globals the project declared.
    options.strict_instance_globals = has_generated_imports && !vue2_dialect;
    collect_generated_template_globals(
        &generated_dir,
        options,
        &seen_names,
        !options.strict_instance_globals,
    );

    for stub in &collected {
        if let Some(name) = declared_name(stub)
            && is_template_component_binding(name)
        {
            external_template_bindings.insert(name.to_compact_string());
        }
    }

    options.auto_import_stubs.extend(collected);
    let mut external_template_bindings = external_template_bindings.into_iter().collect::<Vec<_>>();
    external_template_bindings.sort();
    options.external_template_bindings = external_template_bindings;
    path_aliases
}

pub(in crate::commands::check) fn is_nuxt_project_root(cwd: &Path) -> bool {
    cwd.join("nuxt.config.ts").exists()
        || cwd.join("nuxt.config.js").exists()
        || cwd.join("nuxt.config.mts").exists()
        || cwd.join("nuxt.config.mjs").exists()
}

/// Warning shown once per `vize check` run for a Nuxt project that has no
/// generated Nuxt type artifacts. Without them, auto-imports fall back to
/// permissive `any` stubs that hide real type errors, so the user is told how
/// to generate them. Returns `None` when generated types are present (the
/// checked types are accurate and no warning is warranted).
fn missing_generated_types_warning(
    has_generated_imports: bool,
    generated_dir: &generated_dir::NuxtGeneratedDir,
    legacy_vue2: bool,
) -> Option<String> {
    (!has_generated_imports).then(|| {
        if legacy_vue2 {
            return format!(
                "vize check: no generated `{}` types found; Nuxt auto-imports fall back to `any` \
                 stubs and some type errors will be missed. For Nuxt 2/Bridge, run the project's \
                 Nuxt type-generation step or build once so the generated type directory exists.",
                generated_dir.display()
            )
            .into();
        }

        format!(
            "vize check: no generated `{}` types found; Nuxt auto-imports fall back to `any` \
             stubs and some type errors will be missed. Run `nuxi prepare` to generate them.",
            generated_dir.display()
        )
        .into()
    })
}
