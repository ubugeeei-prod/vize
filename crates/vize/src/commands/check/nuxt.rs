//! Nuxt-specific auto-import and plugin injection helpers.

use std::path::{Path, PathBuf};

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_l0::{FxHashSet, String, ToCompactString, config::VueVersion};

mod fallback;
mod generated;
mod generated_ast;
mod generated_dir;
mod generated_values;
mod legacy_template_globals;
mod parsing;
mod plugins;
mod reference_graph;
mod source_scan;
mod stubs;
mod tsconfig_aliases;
mod type_auto_imports;
mod virtual_modules;

#[cfg(test)]
mod generated_ast_tests;
#[cfg(test)]
mod generated_membership_tests;
#[cfg(test)]
mod generated_tests;
#[cfg(test)]
mod legacy_template_globals_tests;
#[cfg(test)]
mod nuxt4_generated_tests;
#[cfg(test)]
mod strict_globals_tests;
#[cfg(test)]
mod tests;

use fallback::{
    claim_fallback_type_alias_stubs, collect_fallback_stubs, collect_module_fallback_stubs,
};
use generated::{GeneratedImports, collect_generated_stubs, collect_generated_template_globals};
use generated_dir::resolve_nuxt_generated_dir;
use plugins::collect_plugin_injection_stubs;
use reference_graph::installed_reference_stubs;
use source_scan::collect_source_auto_import_stubs;
use stubs::{declared_name, is_template_component_binding};
use tsconfig_aliases::collect_explicit_virtual_module_aliases;
use virtual_modules::{collect_fallback_module_stubs, collect_fallback_path_aliases};

const CHECK_CONFIG_NAMES: [&str; 4] = [
    "nuxt.config.ts",
    "nuxt.config.js",
    "nuxt.config.mts",
    "nuxt.config.mjs",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::commands::check) struct NuxtPathAlias {
    pub(in crate::commands::check) pattern: String,
    pub(in crate::commands::check) targets: Vec<String>,
}

#[derive(Default)]
pub(in crate::commands::check) struct NuxtDetection {
    pub(in crate::commands::check) path_aliases: Vec<NuxtPathAlias>,
    pub(in crate::commands::check) declaration_snapshots: Vec<(PathBuf, String)>,
}

#[cfg(test)]
pub(in crate::commands::check) fn detect_nuxt_auto_imports(
    options: &mut VirtualTsOptions,
    cwd: &Path,
) -> Vec<NuxtPathAlias> {
    detect(options, cwd, cwd, None, None, false, VueVersion::V3)
        .expect("Nuxt declarations")
        .path_aliases
}

#[cfg(test)]
pub(in crate::commands::check) fn detect_legacy_nuxt_auto_imports(
    options: &mut VirtualTsOptions,
    cwd: &Path,
) -> Vec<NuxtPathAlias> {
    detect(options, cwd, cwd, None, None, true, VueVersion::V2_7)
        .expect("Nuxt declarations")
        .path_aliases
}

/// `program_root` is the directory the checked program's virtual project
/// mirrors. It is `cwd` for a standalone app, and the workspace root for an
/// app inside one, so the copied component types name their mirrors from it.
pub(in crate::commands::check) fn detect(
    options: &mut VirtualTsOptions,
    cwd: &Path,
    program_root: &Path,
    tsconfig_path: Option<&Path>,
    configured_files: Option<&[std::path::PathBuf]>,
    legacy_vue2: bool,
    dialect: VueVersion,
) -> Result<NuxtDetection, String> {
    if !is_nuxt_project_root(cwd) {
        return Ok(NuxtDetection::default());
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
    let generated_files = generated_dir.dts_files();
    let mut shadowing_declarations = FxHashSet::default();
    let declaration_graph = configured_files.map(|files| {
        vize_l0::profile!("cli.check.nuxt.declaration_graph", {
            let mut roots = files.to_vec();
            // Pure configured manifests can be intentionally absent from
            // the ambient diagnostic roots (Nuxt 4's nuxt.d.ts). Recover
            // only owned generated candidates through the shared matcher.
            if let Some(config) = tsconfig_path {
                let mut ownership = vize_canon::batch::TsconfigOwnershipCache::default();
                roots.extend(
                    generated_files
                        .iter()
                        .filter(|path| {
                            ownership.project_owns_source(
                                config,
                                path,
                                vize_canon::batch::TsconfigSourceKind::Typed,
                            )
                        })
                        .cloned(),
                );
            }
            super::tsconfig_inputs::collect_installed_declaration_reference_graph_reading(
                &roots,
                Default::default(),
                |path, content| {
                    if super::tsconfig_inputs::declares_shadowing_ambient_module(content) {
                        shadowing_declarations.insert(path.to_path_buf());
                    }
                },
            )
        })
    });
    let mut declaration_snapshots = Vec::new();
    let installed_references = match (&declaration_graph, configured_files) {
        (Some(graph), Some(files)) => {
            declaration_snapshots =
                reference_graph::safe_declaration_snapshots(graph, &shadowing_declarations)?;
            installed_reference_stubs(graph, &shadowing_declarations, files)
        }
        _ => Vec::new(),
    };
    let GeneratedImports {
        found_manifest: has_generated_imports,
        types: mut type_auto_imports,
    } = collect_generated_stubs(
        cwd,
        program_root,
        &generated_dir,
        &generated_files,
        declaration_graph.as_ref().map(|graph| &graph.files),
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
    let fallback_aliases_at = collected.len();
    collect_fallback_stubs(&mut collected, &mut seen_names, has_generated_imports);
    if !has_generated_imports {
        collect_module_fallback_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_auto_import_stubs(
            cwd,
            &mut collected,
            &mut seen_names,
            &mut type_auto_imports,
        );
        // The aliases keep their place ahead of the other fallbacks.
        let aliases = claim_fallback_type_alias_stubs(&mut type_auto_imports);
        collected.splice(fallback_aliases_at..fallback_aliases_at, aliases);
    }
    collected.extend(type_auto_imports.render_stub());
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
        &generated_files,
        declaration_graph.as_ref().map(|graph| &graph.files),
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

    // Triple-slash directives only count ahead of the first statement of the
    // stub file, so the references lead every other stub.
    options.auto_import_stubs.splice(0..0, installed_references);
    options.auto_import_stubs.extend(collected);
    let mut external_template_bindings = external_template_bindings.into_iter().collect::<Vec<_>>();
    external_template_bindings.sort();
    options.external_template_bindings = external_template_bindings;
    Ok(NuxtDetection {
        path_aliases,
        declaration_snapshots,
    })
}

/// Whether Nuxt detection for `cwd` supplies `declaration` itself.
///
/// A detected Nuxt project reads its generated directory into auto-import
/// stubs (or the documented fallback stubs), so the program must not load
/// those generated declarations a second time. Outside a detected project
/// nothing replaces them, and a tsconfig-selected one stays a program root.
pub(in crate::commands::check) fn supplies_generated_declaration(
    cwd: &Path,
    declaration: &Path,
) -> bool {
    if !is_nuxt_project_root(cwd) {
        return false;
    }
    let canonical =
        |path: &Path| std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    canonical(declaration).starts_with(canonical(resolve_nuxt_generated_dir(cwd).path()))
}

pub(in crate::commands::check) fn is_nuxt_project_root(cwd: &Path) -> bool {
    CHECK_CONFIG_NAMES
        .iter()
        .any(|name| cwd.join(name).exists())
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
            return vize_l0::cstr!(
                "vize check: no generated `{}` types found; Nuxt auto-imports fall back to `any` \
                 stubs and some type errors will be missed. For Nuxt 2/Bridge, run the project's \
                 Nuxt type-generation step or build once so the generated type directory exists.",
                generated_dir.display()
            );
        }

        vize_l0::cstr!(
            "vize check: no generated `{}` types found; Nuxt auto-imports fall back to `any` \
             stubs and some type errors will be missed. Run `nuxi prepare` to generate them.",
            generated_dir.display()
        )
    })
}
