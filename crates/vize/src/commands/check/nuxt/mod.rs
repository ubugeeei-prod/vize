//! Nuxt-specific auto-import and plugin injection helpers.

#![allow(clippy::disallowed_macros)]

use std::path::Path;

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::{FxHashSet, String, ToCompactString};

mod fallback;
mod generated;
mod generated_dir;
mod parsing;
mod plugins;
mod source_scan;
mod stubs;
mod virtual_modules;

#[cfg(test)]
mod tests;

use fallback::{collect_fallback_stubs, collect_module_fallback_stubs};
use generated::{collect_generated_stubs, collect_generated_template_globals};
use generated_dir::resolve_nuxt_generated_dir;
use plugins::collect_plugin_injection_stubs;
use source_scan::{collect_source_auto_import_stubs, collect_source_type_auto_import_stubs};
use stubs::{declared_name, is_template_component_binding};
use virtual_modules::{collect_fallback_module_stubs, collect_fallback_path_aliases};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::commands::check) struct NuxtPathAlias {
    pub(in crate::commands::check) pattern: String,
    pub(in crate::commands::check) targets: Vec<String>,
}

pub(in crate::commands::check) fn detect_nuxt_auto_imports(
    options: &mut VirtualTsOptions,
    cwd: &Path,
) -> Vec<NuxtPathAlias> {
    if !is_nuxt_project(cwd) {
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
    if let Some(message) = missing_generated_types_warning(has_generated_imports, &generated_dir) {
        eprintln!("{message}");
    }
    collect_plugin_injection_stubs(cwd, &mut collected, &mut seen_names);
    collect_fallback_stubs(&mut collected, &mut seen_names, has_generated_imports);
    if !has_generated_imports {
        collect_module_fallback_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_auto_import_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_type_auto_import_stubs(cwd, &mut collected);
    }
    collect_fallback_module_stubs(cwd, &mut collected);
    let path_aliases = collect_fallback_path_aliases(cwd, &generated_dir);
    collect_generated_template_globals(&generated_dir, options, &seen_names);

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

fn is_nuxt_project(cwd: &Path) -> bool {
    cwd.join("nuxt.config.ts").exists()
        || cwd.join("nuxt.config.js").exists()
        || cwd.join("nuxt.config.mts").exists()
}

/// Warning shown once per `vize check` run for a Nuxt project that has no
/// generated Nuxt type artifacts. Without them, auto-imports fall back to
/// permissive `any` stubs that hide real type errors, so the user is told how
/// to generate them. Returns `None` when generated types are present (the
/// checked types are accurate and no warning is warranted).
fn missing_generated_types_warning(
    has_generated_imports: bool,
    generated_dir: &generated_dir::NuxtGeneratedDir,
) -> Option<String> {
    (!has_generated_imports).then(|| {
        format!(
            "vize check: no generated `{}` types found; Nuxt auto-imports fall back to `any` \
             stubs and some type errors will be missed. Run `nuxi prepare` to generate them.",
            generated_dir.display()
        )
        .into()
    })
}
