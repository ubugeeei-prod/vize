//! Detection driven by Nuxt's generated type artifacts.

use std::path::Path;

use ignore::WalkBuilder;
use vize_canon::virtual_ts::{TemplateGlobal, VirtualTsOptions};
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::super::dts::{
    parse_declared_global_values, parse_interface_members_with_rewritten_imports,
    rewrite_relative_specifier,
};
use super::generated_dir::NuxtGeneratedDir;
use super::parsing::{
    normalize_component_binding_name, parse_export_names, parse_module_specifier,
};
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
            let is_dts = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".d.ts"));
            if !path.is_file() || !is_dts {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if file_name == "nitro-imports.d.ts" {
                continue;
            }
            if file_name == "imports.d.ts" {
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
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("export {") {
                continue;
            }

            let Some((exports_part, from_part)) = trimmed
                .strip_prefix("export {")
                .and_then(|rest| rest.split_once("} from "))
            else {
                continue;
            };

            let module_specifier = parse_module_specifier(from_part);
            let Some(module_specifier) = module_specifier else {
                continue;
            };
            let module_specifier = rewrite_relative_specifier(module_specifier, base_dir);

            for export_part in exports_part.split(',') {
                let export_part = export_part.trim();
                if export_part.is_empty() {
                    continue;
                }

                let (local_name, exported_name) = parse_export_names(export_part);
                let type_annotation = cstr!("typeof import('{module_specifier}')['{local_name}']");
                push_generated_declared_const(
                    cwd,
                    &imports_path,
                    stubs,
                    seen_names,
                    exported_name,
                    type_annotation.as_str(),
                );
            }
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
    let bytes = type_annotation.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let quote = if type_annotation[i..].starts_with("import('") {
            Some('\'')
        } else if type_annotation[i..].starts_with("import(\"") {
            Some('"')
        } else {
            None
        };

        let Some(quote) = quote else {
            i += 1;
            continue;
        };

        i += 8;
        let start = i;
        while i < bytes.len() && bytes[i] != quote as u8 {
            i += 1;
        }

        if generated_import_specifier_is_missing(
            &type_annotation[start..i],
            type_origin,
            project_root,
        ) {
            return true;
        }

        if i < bytes.len() {
            i += 1;
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
        "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "vue", "d.ts",
    ] {
        if path.with_extension(extension).is_file() {
            return true;
        }
    }

    if path.is_dir() {
        for extension in [
            "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "vue", "d.ts",
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

pub(super) fn collect_generated_template_globals(
    generated_dir: &NuxtGeneratedDir,
    options: &mut VirtualTsOptions,
    seen_auto_imports: &FxHashSet<String>,
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

    if seen_auto_imports.contains("useI18n") || seen_auto_imports.contains("useLocalePath") {
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

fn rewrite_component_imports_for_virtual_project(
    type_annotation: &str,
    project_root: &Path,
) -> String {
    let bytes = type_annotation.as_bytes();
    let mut out = String::with_capacity(type_annotation.len());
    let mut i = 0usize;

    while i < bytes.len() {
        let quote = if type_annotation[i..].starts_with("import('") {
            Some('\'')
        } else if type_annotation[i..].starts_with("import(\"") {
            Some('"')
        } else {
            None
        };

        let Some(quote) = quote else {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        };

        out.push_str("import(");
        out.push(quote);
        i += 8;

        let start = i;
        while i < bytes.len() && bytes[i] != quote as u8 {
            i += 1;
        }

        let specifier = &type_annotation[start..i];
        out.push_str(&virtual_project_specifier(specifier, project_root));

        if i < bytes.len() {
            out.push(quote);
            i += 1;
        }
    }

    out
}

fn virtual_project_specifier(specifier: &str, project_root: &Path) -> String {
    if !specifier.ends_with(".vue") {
        return specifier.to_compact_string();
    }

    let specifier_path = Path::new(specifier);
    let relative = if specifier_path.is_absolute() {
        specifier_path.strip_prefix(project_root).ok()
    } else {
        None
    };

    if let Some(relative) = relative {
        let mut rendered = cstr!("./{}", relative.display());
        rendered.push_str(".ts");
        return rendered;
    }

    cstr!("{specifier}.ts")
}
