//! Nuxt-specific auto-import and plugin injection helpers.

#![allow(clippy::disallowed_macros)]

use std::path::Path;

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::{FxHashSet, String, ToCompactString};

mod fallback;
mod generated;
mod parsing;
mod plugins;
mod source_scan;
mod stubs;
mod virtual_modules;

#[cfg(test)]
mod tests;

use fallback::{collect_fallback_stubs, collect_module_fallback_stubs};
use generated::{collect_generated_stubs, collect_generated_template_globals};
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
    let has_generated_imports = collect_generated_stubs(
        cwd,
        &mut collected,
        &mut seen_names,
        &mut external_template_bindings,
    );
    collect_plugin_injection_stubs(cwd, &mut collected, &mut seen_names);
    collect_fallback_stubs(&mut collected, &mut seen_names, has_generated_imports);
    if !has_generated_imports {
        collect_module_fallback_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_auto_import_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_type_auto_import_stubs(cwd, &mut collected);
    }
    collect_fallback_module_stubs(cwd, &mut collected);
    let path_aliases = collect_fallback_path_aliases(cwd);
    collect_generated_template_globals(cwd, options, &seen_names);

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
