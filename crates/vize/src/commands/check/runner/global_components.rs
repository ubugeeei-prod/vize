//! Virtual TS option assembly and global-component stub collection for the
//! `check` runner.
//!
//! These helpers translate `vize.config` globals, template syntax, and project
//! `declare module "vue"` augmentations into the `VirtualTsOptions` the batch
//! type checker consumes.

#![allow(clippy::disallowed_macros)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use vize_carton::{FxHashSet, String, cstr};

use super::resolve::resolve_from_config_dir;
use crate::commands::check::tsconfig_inputs::{
    parse_jsonc_value, read_extends_entries, resolve_extended_tsconfig,
};

pub(super) fn build_virtual_ts_options(
    config: &crate::config::VizeConfig,
    config_dir: &Path,
) -> vize_canon::virtual_ts::VirtualTsOptions {
    let mut template_globals = config
        .global_types
        .iter()
        .map(
            |(name, declaration)| vize_canon::virtual_ts::TemplateGlobal {
                name: name.clone(),
                type_annotation: declaration.type_annotation.clone(),
                default_value: declaration.template_default_value(),
            },
        )
        .collect::<Vec<_>>();

    let globals_path = config
        .type_checker
        .globals_file
        .as_deref()
        .map(|candidate| resolve_from_config_dir(config_dir, candidate));

    if template_globals.is_empty()
        && let Some(ref globals_path) = globals_path
    {
        match parse_dts_globals(globals_path) {
            Ok(globals) => template_globals = globals,
            Err(error) => {
                eprintln!(
                    "\x1b[33mWarning:\x1b[0m Failed to parse globals from {}: {}",
                    globals_path.display(),
                    error
                );
            }
        }
    }

    vize_canon::virtual_ts::VirtualTsOptions {
        template_globals,
        ..Default::default()
    }
}

/// Resolve the configured Vue dialect for canon's virtual-TS generation.
///
/// `vue.version` is optional in `vize.config`; an unset value means the default
/// Vue 3 dialect. Plumbing only today (issue #1392): the resolved dialect is
/// threaded into canon so it can later emit dialect-aware instance types, but it
/// does not change generated output yet.
pub(super) fn dialect_from_features(
    vue_version: Option<crate::config::VueVersion>,
) -> crate::config::VueVersion {
    vue_version.unwrap_or(crate::config::VueVersion::V3)
}

pub(super) fn template_syntax_mode(
    template_syntax: Option<&str>,
) -> vize_atelier_core::TemplateSyntaxMode {
    match template_syntax {
        Some("strict") => vize_atelier_core::TemplateSyntaxMode::Strict,
        Some("quirks") => vize_atelier_core::TemplateSyntaxMode::Quirks,
        Some("standard") | None => vize_atelier_core::TemplateSyntaxMode::Standard,
        Some(_) => vize_atelier_core::TemplateSyntaxMode::Standard,
    }
}

pub(super) fn collect_project_global_component_stubs(
    options: &mut vize_canon::virtual_ts::VirtualTsOptions,
    files: &[PathBuf],
    project_root: &Path,
    tsconfig_path: Option<&Path>,
) {
    let mut seen_names = options
        .auto_import_stubs
        .iter()
        .filter_map(|stub| declared_stub_name(stub))
        .map(String::from)
        .collect::<FxHashSet<_>>();
    let mut external_template_bindings = options
        .external_template_bindings
        .iter()
        .cloned()
        .collect::<FxHashSet<_>>();
    let mut collected = Vec::new();
    let mut package_reference_stubs = Vec::new();
    let mut seen_package_references = FxHashSet::default();

    let mut declaration_sources = files
        .iter()
        .filter(|path| is_declaration_path(path))
        .map(|path| GlobalComponentDeclarationSource {
            path: path.clone(),
            type_package: None,
        })
        .collect::<Vec<_>>();

    for package in collect_global_component_type_packages(files, tsconfig_path) {
        for path in resolve_type_package_declaration_files(project_root, package.as_str()) {
            declaration_sources.push(GlobalComponentDeclarationSource {
                path,
                type_package: Some(package.clone()),
            });
        }
    }

    for source in declaration_sources {
        let Ok(components) =
            super::super::dts::parse_global_component_members_with_rewritten_imports(&source.path)
        else {
            continue;
        };

        for (name, type_annotation) in components {
            let Some(name) = normalize_global_component_binding_name(name.as_str()) else {
                continue;
            };
            external_template_bindings.insert(name.clone());
            if !seen_names.insert(name.clone()) {
                continue;
            }

            if let Some(package) = source.type_package.as_deref() {
                if seen_package_references.insert(String::from(package)) {
                    package_reference_stubs.push(cstr!("/// <reference types=\"{package}\" />"));
                }
                collected.push(cstr!(
                    "declare const {name}: import(\"vue\").GlobalComponents[\"{name}\"];"
                ));
            } else {
                let type_annotation = rewrite_global_component_imports_for_virtual_project(
                    type_annotation.as_str(),
                    project_root,
                );
                collected.push(cstr!("declare const {name}: {type_annotation};"));
            }
        }
    }

    if !package_reference_stubs.is_empty() {
        let mut stubs = Vec::with_capacity(
            package_reference_stubs.len() + options.auto_import_stubs.len() + collected.len(),
        );
        stubs.extend(package_reference_stubs);
        stubs.append(&mut options.auto_import_stubs);
        stubs.extend(collected);
        options.auto_import_stubs = stubs;
    } else if !collected.is_empty() {
        options.auto_import_stubs.extend(collected);
    }
    let mut external_template_bindings = external_template_bindings.into_iter().collect::<Vec<_>>();
    external_template_bindings.sort();
    options.external_template_bindings = external_template_bindings;
}

struct GlobalComponentDeclarationSource {
    path: PathBuf,
    type_package: Option<String>,
}

fn is_declaration_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

fn collect_global_component_type_packages(
    files: &[PathBuf],
    tsconfig_path: Option<&Path>,
) -> Vec<String> {
    let mut packages = Vec::new();
    let mut seen = FxHashSet::default();

    for package in collect_tsconfig_type_packages(tsconfig_path) {
        push_unique_type_package(&mut packages, &mut seen, package);
    }

    for path in files.iter().filter(|path| is_declaration_path(path)) {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for package in reference_type_packages(&content) {
            push_unique_type_package(&mut packages, &mut seen, package);
        }
    }

    packages
}

fn push_unique_type_package(
    packages: &mut Vec<String>,
    seen: &mut FxHashSet<String>,
    package: String,
) {
    if seen.insert(package.clone()) {
        packages.push(package);
    }
}

fn collect_tsconfig_type_packages(tsconfig_path: Option<&Path>) -> Vec<String> {
    let Some(tsconfig_path) = tsconfig_path else {
        return Vec::new();
    };

    let mut seen = FxHashSet::default();
    load_tsconfig_type_packages(tsconfig_path, &mut seen).unwrap_or_default()
}

fn load_tsconfig_type_packages(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> Option<Vec<String>> {
    let resolved = tsconfig_path
        .canonicalize()
        .unwrap_or_else(|_| tsconfig_path.to_path_buf());
    if !seen.insert(resolved.clone()) {
        return None;
    }

    let content = fs::read_to_string(&resolved).ok()?;
    let value = parse_jsonc_value(&content).ok()?;

    let mut inherited = Vec::new();
    for extends in read_extends_entries(&value) {
        let Some(extends_path) = resolve_extended_tsconfig(&resolved, &extends) else {
            continue;
        };
        if let Some(parent_types) = load_tsconfig_type_packages(&extends_path, seen) {
            inherited.extend(parent_types);
        }
    }

    if let Some(types) = value
        .get("compilerOptions")
        .and_then(Value::as_object)
        .and_then(|compiler_options| compiler_options.get("types"))
        .and_then(Value::as_array)
    {
        return Some(
            types
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect(),
        );
    }

    (!inherited.is_empty()).then_some(inherited)
}

fn reference_type_packages(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(reference_types_attribute)
        .map(String::from)
        .collect()
}

fn reference_types_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "types")
}

fn reference_path_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "path")
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let needle = cstr!("{name}=");
    let start = line.find(needle.as_str())? + needle.len();
    let quote = line[start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = line[value_start..].find(quote)? + value_start;
    line.get(value_start..value_end)
}

fn resolve_type_package_declaration_files(project_root: &Path, package: &str) -> Vec<PathBuf> {
    let Some(package_root) = resolve_type_package_root(project_root, package) else {
        return Vec::new();
    };

    for entry in package_declaration_entry_candidates(&package_root) {
        if is_declaration_path(&entry) && entry.is_file() {
            return collect_package_declaration_graph(&entry, &package_root);
        }
    }

    Vec::new()
}

fn resolve_type_package_root(project_root: &Path, package: &str) -> Option<PathBuf> {
    let mut current = Some(project_root);
    while let Some(dir) = current {
        let node_modules = dir.join("node_modules");
        let direct = node_modules.join(package);
        if direct.is_dir() {
            return Some(direct);
        }

        if let Some(types_package) = fallback_types_package_name(package) {
            let fallback = node_modules.join(types_package);
            if fallback.is_dir() {
                return Some(fallback);
            }
        }

        current = dir.parent();
    }

    None
}

fn fallback_types_package_name(package: &str) -> Option<String> {
    if package.starts_with("@types/") {
        return None;
    }
    if let Some(scoped) = package.strip_prefix('@') {
        let mut parts = scoped.split('/');
        let scope = parts.next()?;
        let name = parts.next()?;
        return Some(cstr!("@types/{scope}__{name}"));
    }
    Some(cstr!("@types/{package}"))
}

fn package_declaration_entry_candidates(package_root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let package_json_path = package_root.join("package.json");
    if let Ok(content) = fs::read_to_string(&package_json_path)
        && let Ok(package_json) = parse_jsonc_value(&content)
    {
        for field in ["types", "typings"] {
            if let Some(types) = package_json.get(field).and_then(Value::as_str) {
                push_declaration_entry_candidate(&mut candidates, package_root.join(types));
            }
        }

        if let Some(exports) = package_json.get("exports") {
            let root_export = exports.get(".").unwrap_or(exports);
            collect_export_type_entries(root_export, package_root, &mut candidates);
        }
    }

    push_declaration_entry_candidate(&mut candidates, package_root.join("index.d.ts"));
    candidates
}

fn collect_export_type_entries(value: &Value, package_root: &Path, candidates: &mut Vec<PathBuf>) {
    match value {
        Value::String(path) => {
            push_declaration_entry_candidate(candidates, package_root.join(path))
        }
        Value::Array(values) => {
            for value in values {
                collect_export_type_entries(value, package_root, candidates);
            }
        }
        Value::Object(map) => {
            if let Some(types) = map.get("types").and_then(Value::as_str) {
                push_declaration_entry_candidate(candidates, package_root.join(types));
            }
            for value in map.values() {
                collect_export_type_entries(value, package_root, candidates);
            }
        }
        _ => {}
    }
}

fn push_declaration_entry_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.contains(&path) {
        candidates.push(path.clone());
    }
    if path.extension().is_none() {
        let with_extension = path.with_extension("d.ts");
        if !candidates.contains(&with_extension) {
            candidates.push(with_extension);
        }
    }
    let index = path.join("index.d.ts");
    if !candidates.contains(&index) {
        candidates.push(index);
    }
}

fn collect_package_declaration_graph(entry: &Path, package_root: &Path) -> Vec<PathBuf> {
    let package_root = package_root
        .canonicalize()
        .unwrap_or_else(|_| package_root.to_path_buf());
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();
    let mut queue = vec![entry.to_path_buf()];

    while let Some(path) = queue.pop() {
        let path = path.canonicalize().unwrap_or(path);
        if !seen.insert(path.clone()) {
            continue;
        }
        files.push(path.clone());

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(base_dir) = path.parent() else {
            continue;
        };
        for reference in content.lines().filter_map(reference_path_attribute) {
            let referenced = base_dir.join(reference);
            let referenced = referenced.canonicalize().unwrap_or(referenced);
            if referenced.starts_with(&package_root)
                && is_declaration_path(&referenced)
                && referenced.is_file()
            {
                queue.push(referenced);
            }
        }
    }

    files
}

fn declared_stub_name(stub: &str) -> Option<&str> {
    for prefix in [
        "declare function ",
        "declare const ",
        "declare let ",
        "declare var ",
    ] {
        let Some(rest) = stub.strip_prefix(prefix) else {
            continue;
        };
        let end = rest
            .find(['<', '(', ':', '=', ';', ' '])
            .unwrap_or(rest.len());
        let name = rest[..end].trim();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn normalize_global_component_binding_name(name: &str) -> Option<String> {
    let name = name.trim().trim_matches('"').trim_matches('\'');
    if name.is_empty() {
        return None;
    }
    if name.chars().enumerate().all(|(index, ch)| {
        ch == '_'
            || ch == '$'
            || (ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit()))
    }) {
        return Some(name.into());
    }
    None
}

fn rewrite_global_component_imports_for_virtual_project(
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
        out.push_str(&virtual_project_global_component_specifier(
            specifier,
            project_root,
        ));

        if i < bytes.len() {
            out.push(quote);
            i += 1;
        }
    }

    out
}

fn virtual_project_global_component_specifier(specifier: &str, project_root: &Path) -> String {
    if !specifier.ends_with(".vue") {
        return specifier.into();
    }

    let specifier_path = Path::new(specifier);
    if let Some(relative) = specifier_path
        .is_absolute()
        .then(|| specifier_path.strip_prefix(project_root).ok())
        .flatten()
    {
        let mut rendered = cstr!("./{}", relative.display());
        rendered.push_str(".ts");
        return rendered;
    }

    cstr!("{specifier}.ts")
}

/// Parse a `.d.ts` file containing `ComponentCustomProperties` augmentation.
fn parse_dts_globals(
    path: &Path,
) -> Result<Vec<vize_canon::virtual_ts::TemplateGlobal>, std::io::Error> {
    use super::super::dts::parse_interface_members;
    use vize_canon::virtual_ts::TemplateGlobal;

    Ok(
        parse_interface_members(path, "interface ComponentCustomProperties")?
            .into_iter()
            .map(|(name, type_annotation)| TemplateGlobal {
                name,
                type_annotation,
                default_value: "{} as any".into(),
            })
            .collect(),
    )
}
