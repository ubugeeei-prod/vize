//! Detection driven by Nuxt's generated type artifacts.

use std::path::{Path, PathBuf};

use vize_canon::virtual_ts::{TemplateGlobal, VirtualTsOptions};
use vize_l0::{FxHashSet, String, ToCompactString, cstr};

use super::super::dts::{
    GlobalTypeReexport, parse_declared_globals, parse_interface_members_with_rewritten_imports,
};
use super::super::tsconfig_inputs::normalize_input_path;
use super::generated_ast::{
    parse_import_manifest_exports, rewrite_component_imports_for_virtual_project,
};
use super::generated_dir::{
    NuxtGeneratedDir, is_imports_declaration, is_nitro_imports_declaration,
};
use super::generated_values::{
    push_generated_declared_const, push_generated_declared_global_const,
};
use super::parsing::normalize_component_binding_name;
use super::stubs::{push_declared_const, tracked_read_to_string};
use super::type_auto_imports::TypeAutoImports;

/// What the generated declarations of a Nuxt project supply.
pub(super) struct GeneratedImports {
    /// Whether the project has a generated import manifest at all, read or not.
    pub(super) found_manifest: bool,
    /// The auto-imported types of the manifests this program reads.
    pub(super) types: TypeAutoImports,
}

pub(super) fn collect_generated_stubs(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
    generated_files: &[PathBuf],
    declaration_graph: Option<&FxHashSet<PathBuf>>,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) -> GeneratedImports {
    let nuxt_types_dir = generated_dir.types_dir();
    let mut found_manifest = false;
    let mut types = TypeAutoImports::default();

    for path in generated_files
        .iter()
        .filter(|path| path.starts_with(&nuxt_types_dir))
    {
        if is_nitro_imports_declaration(path) {
            continue;
        }
        if is_imports_declaration(path) {
            found_manifest = true;
        }
        if !is_configured_declaration(path, declaration_graph) {
            continue;
        }

        let reexports =
            collect_declared_global_stubs(cwd, path, stubs, seen_names, external_template_bindings);
        collect_manifest_type_reexports(path, reexports, &mut types);
        collect_global_component_stubs(cwd, path, stubs, seen_names, external_template_bindings);
    }

    for path in generated_files
        .iter()
        .filter(|path| path.parent() == Some(generated_dir.path()))
    {
        if !is_configured_declaration(path, declaration_graph) {
            continue;
        }
        let reexports =
            collect_declared_global_stubs(cwd, path, stubs, seen_names, external_template_bindings);
        collect_manifest_type_reexports(path, reexports, &mut types);
    }
    collect_root_generated_component_stubs(
        cwd,
        generated_dir,
        generated_files,
        declaration_graph,
        stubs,
        seen_names,
        external_template_bindings,
    );

    if !found_manifest {
        let imports_path = generated_dir.imports_path();
        if imports_path.exists() {
            found_manifest = true;
            if is_configured_declaration(&imports_path, declaration_graph) {
                collect_root_manifest_export_stubs(cwd, &imports_path, stubs, seen_names);
            }
        }
    }

    GeneratedImports {
        found_manifest,
        types,
    }
}

fn collect_root_manifest_export_stubs(
    cwd: &Path,
    imports_path: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let Ok(content) = tracked_read_to_string(imports_path) else {
        return;
    };
    let base_dir = imports_path.parent().unwrap_or_else(|| Path::new("."));
    for manifest_export in parse_import_manifest_exports(content.as_str(), base_dir) {
        let type_annotation = cstr!(
            "typeof import('{}')['{}']",
            manifest_export.module_specifier,
            manifest_export.local_name
        );
        push_generated_declared_const(
            cwd,
            imports_path,
            stubs,
            seen_names,
            manifest_export.exported_name.as_str(),
            type_annotation.as_str(),
        );
    }
}

/// Turn the `declare global` consts of one generated declaration into stubs,
/// and hand back the type re-exports the same walk found.
fn collect_declared_global_stubs(
    cwd: &Path,
    path: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) -> Vec<GlobalTypeReexport> {
    let Ok(globals) = parse_declared_globals(path) else {
        return Vec::new();
    };
    for (name, type_annotation) in globals.values {
        push_generated_declared_global_const(
            cwd,
            path,
            stubs,
            seen_names,
            external_template_bindings,
            &name,
            &type_annotation,
        );
    }
    globals.type_reexports
}

/// Only an import manifest hands its type re-exports to the stub. The program
/// reads every other generated declaration itself, re-exports included, so a
/// stub for those would declare each of their names a second time.
fn collect_manifest_type_reexports(
    path: &Path,
    reexports: Vec<GlobalTypeReexport>,
    types: &mut TypeAutoImports,
) {
    if !is_imports_declaration(path) {
        return;
    }
    for reexport in reexports {
        types.reexport(
            reexport.specifier.as_str(),
            reexport.local.as_str(),
            reexport.exported.as_str(),
        );
    }
}

fn collect_root_generated_component_stubs(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
    generated_files: &[PathBuf],
    declaration_graph: Option<&FxHashSet<PathBuf>>,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) {
    for path in generated_files
        .iter()
        .filter(|path| path.parent() == Some(generated_dir.path()))
    {
        if !is_configured_declaration(path, declaration_graph) {
            continue;
        }
        collect_global_component_stubs(
            cwd,
            path.as_path(),
            stubs,
            seen_names,
            external_template_bindings,
        );
    }
}

fn collect_global_component_stubs(
    cwd: &Path,
    path: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) {
    let Ok(components) =
        parse_interface_members_with_rewritten_imports(path, "interface GlobalComponents")
    else {
        return;
    };

    for (name, type_annotation) in components {
        let Some(name) = normalize_component_binding_name(name.as_str()) else {
            continue;
        };
        let type_annotation =
            rewrite_component_imports_for_virtual_project(type_annotation.as_str(), cwd);
        external_template_bindings.insert(name.clone());
        push_declared_const(stubs, seen_names, name.as_str(), type_annotation.as_str());
    }
}

/// Declare the `$`-prefixed template globals the generated Nuxt types name.
///
/// `infer_i18n_globals` adds the vue-i18n instance properties whenever an i18n
/// composable is auto-imported. That inference is only sound while the project
/// has no generated type graph to consult: once it has one, an auto-imported
/// `useI18n` says nothing about whether `$t` is declared, and inventing it hides
/// the `TS2339` the Vue toolchain reports for every authored use.
pub(super) fn collect_generated_template_globals(
    generated_files: &[PathBuf],
    declaration_graph: Option<&FxHashSet<PathBuf>>,
    options: &mut VirtualTsOptions,
    seen_auto_imports: &FxHashSet<String>,
    infer_i18n_globals: bool,
) {
    let mut seen_globals = options
        .template_globals
        .iter()
        .map(|global| global.name.clone())
        .collect::<FxHashSet<_>>();

    for path in generated_files {
        if !is_configured_declaration(path, declaration_graph) {
            continue;
        }
        let Ok(members) = parse_interface_members_with_rewritten_imports(
            path.as_path(),
            "interface ComponentCustomProperties",
        ) else {
            continue;
        };

        for (name, _type_annotation) in members {
            let Some(name) = normalize_component_binding_name(name.as_str()) else {
                continue;
            };
            if !name.starts_with('$') {
                continue;
            }
            push_template_global(options, &mut seen_globals, name.as_str(), "any");
        }
    }

    if infer_i18n_globals
        && (seen_auto_imports.contains("useI18n") || seen_auto_imports.contains("useLocalePath"))
    {
        collect_i18n_template_globals(options, &mut seen_globals);
    }
    if !infer_i18n_globals {
        collect_router_template_globals(options, &mut seen_globals, seen_auto_imports);
    }
}

fn is_configured_declaration(path: &Path, graph: Option<&FxHashSet<PathBuf>>) -> bool {
    graph.is_none_or(|files| files.contains(&normalize_input_path(path)))
}

fn collect_i18n_template_globals(
    options: &mut VirtualTsOptions,
    seen_globals: &mut FxHashSet<String>,
) {
    for (name, type_annotation) in [
        ("$t", "(...args: any[]) => any"),
        ("$rt", "(...args: any[]) => any"),
        ("$d", "(...args: any[]) => any"),
        ("$n", "(...args: any[]) => any"),
        ("$tm", "(...args: any[]) => any"),
        ("$te", "(...args: any[]) => boolean"),
        ("$i18n", "any"),
    ] {
        push_template_global(options, seen_globals, name, type_annotation);
    }
}

fn collect_router_template_globals(
    options: &mut VirtualTsOptions,
    seen_globals: &mut FxHashSet<String>,
    seen_auto_imports: &FxHashSet<String>,
) {
    for (auto_import, name, type_annotation) in [
        ("useRoute", "$route", "ReturnType<typeof useRoute>"),
        ("useRouter", "$router", "ReturnType<typeof useRouter>"),
    ] {
        if seen_auto_imports.contains(auto_import) {
            push_template_global(options, seen_globals, name, type_annotation);
        }
    }
}

fn push_template_global(
    options: &mut VirtualTsOptions,
    seen_globals: &mut FxHashSet<String>,
    name: &str,
    type_annotation: &str,
) {
    if seen_globals.insert(name.to_compact_string()) {
        options.template_globals.push(TemplateGlobal {
            name: name.to_compact_string(),
            type_annotation: type_annotation.to_compact_string(),
            default_value: "undefined as any".into(),
        });
    }
}
