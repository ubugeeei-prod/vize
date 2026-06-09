//! Detection driven by Nuxt's generated `.nuxt` type artifacts.

use std::{fs, path::Path};

use ignore::WalkBuilder;
use vize_canon::virtual_ts::{TemplateGlobal, VirtualTsOptions};
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::super::dts::{
    parse_declared_global_values, parse_interface_members_with_rewritten_imports,
    rewrite_relative_specifier,
};
use super::parsing::{
    normalize_component_binding_name, parse_export_names, parse_module_specifier,
};
use super::stubs::{push_declared_const, tracked_read_to_string};

pub(super) fn collect_generated_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) -> bool {
    let nuxt_types_dir = cwd.join(".nuxt/types");
    let mut found_import_manifest = false;

    if nuxt_types_dir.exists() {
        let walker = WalkBuilder::new(&nuxt_types_dir)
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
                    push_declared_const(stubs, seen_names, &name, &type_annotation);
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

    collect_root_generated_global_stubs(cwd, stubs, seen_names);
    collect_root_generated_component_stubs(cwd, stubs, seen_names, external_template_bindings);

    if found_import_manifest {
        return true;
    }

    let imports_path = cwd.join(".nuxt/imports.d.ts");
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
                push_declared_const(
                    stubs,
                    seen_names,
                    exported_name,
                    &cstr!("typeof import('{module_specifier}')['{local_name}']"),
                );
            }
        }
    }

    found_import_manifest
}

fn collect_root_generated_global_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    for path in root_generated_dts_files(cwd) {
        if let Ok(values) = parse_declared_global_values(path.as_path()) {
            for (name, type_annotation) in values {
                push_declared_const(stubs, seen_names, &name, &type_annotation);
            }
        }
    }
}

fn collect_root_generated_component_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) {
    for path in root_generated_dts_files(cwd) {
        collect_global_component_stubs(
            cwd,
            path.as_path(),
            stubs,
            seen_names,
            external_template_bindings,
        );
    }
}

fn root_generated_dts_files(cwd: &Path) -> Vec<std::path::PathBuf> {
    let nuxt_dir = cwd.join(".nuxt");
    let Ok(entries) = fs::read_dir(&nuxt_dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".d.ts"))
        })
        .collect()
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
    cwd: &Path,
    options: &mut VirtualTsOptions,
    seen_auto_imports: &FxHashSet<String>,
) {
    let mut seen_globals = options
        .template_globals
        .iter()
        .map(|global| global.name.clone())
        .collect::<FxHashSet<_>>();

    for path in generated_dts_files(cwd) {
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

fn generated_dts_files(cwd: &Path) -> Vec<std::path::PathBuf> {
    let mut files = root_generated_dts_files(cwd);
    let nuxt_types_dir = cwd.join(".nuxt/types");
    if nuxt_types_dir.exists() {
        let walker = WalkBuilder::new(&nuxt_types_dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            let is_dts = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".d.ts"));
            if path.is_file() && is_dts {
                files.push(path.to_path_buf());
            }
        }
    }
    files
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
