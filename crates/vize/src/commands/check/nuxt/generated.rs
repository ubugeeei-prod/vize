//! Detection driven by Nuxt's generated type artifacts.

use std::path::Path;

use ignore::WalkBuilder;
use vize_canon::virtual_ts::{TemplateGlobal, VirtualTsOptions};
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::super::dts::{
    parse_declared_global_values, parse_interface_members_with_rewritten_imports,
};
use super::generated_ast::{
    collect_import_type_specifiers, parse_import_manifest_exports,
    rewrite_component_imports_for_virtual_project,
};
use super::generated_dir::{
    NuxtGeneratedDir, is_declaration_file, is_imports_declaration, is_nitro_imports_declaration,
};
use super::parsing::normalize_component_binding_name;
use super::stubs::{push_declared_const, tracked_read_to_string};

pub(super) fn collect_generated_stubs(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) -> bool {
    let nuxt_types_dir = generated_dir.types_dir();
    let mut found_import_manifest = false;

    if nuxt_types_dir.exists() {
        let walker = WalkBuilder::new(nuxt_types_dir.as_path())
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() || !is_declaration_file(path) {
                continue;
            }

            if is_nitro_imports_declaration(path) {
                continue;
            }
            if is_imports_declaration(path) {
                found_import_manifest = true;
            }

            if let Ok(values) = parse_declared_global_values(path) {
                for (name, type_annotation) in values {
                    push_generated_declared_const(
                        cwd,
                        path,
                        stubs,
                        seen_names,
                        &name,
                        &type_annotation,
                    );
                }
            }

            collect_global_component_stubs(
                cwd,
                path,
                stubs,
                seen_names,
                external_template_bindings,
            );
        }
    }

    collect_root_generated_global_stubs(cwd, generated_dir, stubs, seen_names);
    collect_root_generated_component_stubs(
        cwd,
        generated_dir,
        stubs,
        seen_names,
        external_template_bindings,
    );

    if found_import_manifest {
        return true;
    }

    let imports_path = generated_dir.imports_path();
    if !imports_path.exists() {
        return false;
    }
    found_import_manifest = true;

    if let Ok(content) = tracked_read_to_string(&imports_path) {
        let base_dir = imports_path.parent().unwrap_or_else(|| Path::new("."));
        for manifest_export in parse_import_manifest_exports(content.as_str(), base_dir) {
            let type_annotation = cstr!(
                "typeof import('{}')['{}']",
                manifest_export.module_specifier,
                manifest_export.local_name
            );
            push_generated_declared_const(
                cwd,
                &imports_path,
                stubs,
                seen_names,
                manifest_export.exported_name.as_str(),
                type_annotation.as_str(),
            );
        }
    }

    found_import_manifest
}

fn collect_root_generated_global_stubs(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    for path in generated_dir.root_dts_files() {
        if let Ok(values) = parse_declared_global_values(path.as_path()) {
            for (name, type_annotation) in values {
                push_generated_declared_const(
                    cwd,
                    path.as_path(),
                    stubs,
                    seen_names,
                    &name,
                    &type_annotation,
                );
            }
        }
    }
}

fn push_generated_declared_const(
    cwd: &Path,
    type_origin: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
    type_annotation: &str,
) {
    let type_annotation = generated_type_annotation_or_any(type_annotation, type_origin, cwd);
    push_declared_const(stubs, seen_names, name, type_annotation.as_str());
}

fn generated_type_annotation_or_any(
    type_annotation: &str,
    type_origin: &Path,
    project_root: &Path,
) -> String {
    if has_missing_project_import_type(type_annotation, type_origin, project_root) {
        return "any".into();
    }

    type_annotation.to_compact_string()
}

fn has_missing_project_import_type(
    type_annotation: &str,
    type_origin: &Path,
    project_root: &Path,
) -> bool {
    for specifier in collect_import_type_specifiers(type_annotation) {
        if generated_import_specifier_is_missing(specifier.as_str(), type_origin, project_root) {
            return true;
        }
    }

    false
}

fn generated_import_specifier_is_missing(
    specifier: &str,
    type_origin: &Path,
    project_root: &Path,
) -> bool {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        let base_dir = type_origin.parent().unwrap_or(project_root);
        return !module_path_exists(&base_dir.join(specifier));
    }

    let specifier_path = Path::new(specifier);
    if !specifier_path.is_absolute() || !specifier_path.starts_with(project_root) {
        return false;
    }
    if specifier_path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| name == "node_modules")
    }) {
        return false;
    }

    !module_path_exists(specifier_path)
}

fn module_path_exists(path: &Path) -> bool {
    if path.is_file() {
        return true;
    }

    for extension in [
        "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "vue", "d.ts", "d.mts", "d.cts",
    ] {
        if path.with_extension(extension).is_file() {
            return true;
        }
    }

    if path.is_dir() {
        for extension in [
            "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "vue", "d.ts", "d.mts", "d.cts",
        ] {
            if path.join("index").with_extension(extension).is_file() {
                return true;
            }
        }
    }

    false
}

fn collect_root_generated_component_stubs(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) {
    for path in generated_dir.root_dts_files() {
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
    generated_dir: &NuxtGeneratedDir,
    options: &mut VirtualTsOptions,
    seen_auto_imports: &FxHashSet<String>,
    infer_i18n_globals: bool,
) {
    let mut seen_globals = options
        .template_globals
        .iter()
        .map(|global| global.name.clone())
        .collect::<FxHashSet<_>>();

    for path in generated_dir.dts_files() {
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
