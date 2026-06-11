//! Ambient declaration (`.d.ts`) collection for partial `vize check` runs.

use std::{
    fs,
    path::{Path, PathBuf},
};

use vize_carton::FxHashSet;

use super::NODE_MODULES_DIR;
use super::collect_default_check_files_inner;
use super::glob::normalize_input_path;
use super::loader::TsconfigInputCache;
use super::matching::path_has_component;

/// Collect ambient declaration (`.d.ts`) files that belong to the tsconfig
/// "program" so their global types stay in scope when only a subset of files is
/// checked explicitly (e.g. `vize check src/App.vue`).
///
/// Ambient declarations (`declare global`, top-level `declare const`) are not
/// pulled in by imports, so the explicit-path collector drops them and `tsgo`
/// then reports false `TS2304` errors for genuinely global names. This mirrors
/// `tsc`, which always loads the declaration files matched by `files`/`include`
/// regardless of which entry files are requested.
///
/// Project shims such as `declare module "~icons/foo"` and Nuxt's generated
/// `.nuxt/nuxt.d.ts` are part of that program even though they are not imported
/// by the checked file. Only unsafe bare Vue package shims without top-level
/// import/export are excluded, because those replace the real package instead
/// of augmenting it.
pub(crate) fn collect_ambient_declaration_files(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
    cache: &mut TsconfigInputCache,
) -> Vec<PathBuf> {
    let project_root = normalize_input_path(project_root);
    let mut files = collect_default_check_files_inner(&project_root, tsconfig_path, true, cache);
    let mut seen = files.iter().cloned().collect::<FxHashSet<_>>();
    let mut index = 0;
    while index < files.len() {
        let path = files[index].clone();
        index += 1;
        if !is_declaration_file(&path) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for referenced in reference_path_declaration_files(&path, &content, &project_root) {
            if seen.insert(referenced.clone()) {
                files.push(referenced);
            }
        }
    }

    files
        .into_iter()
        .filter(|path| is_declaration_file(path))
        .filter(|path| match fs::read_to_string(path) {
            Ok(content) => {
                !is_reference_manifest_declaration(&content)
                    && !declares_shadowing_ambient_module(&content)
            }
            Err(_) => false,
        })
        .collect()
}

pub(super) fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

/// Returns `true` when a declaration file would replace real Vue package types
/// if loaded as a program root. Project shims such as `declare module "*.css"`
/// and `declare module "~icons/foo"` must still be loaded for explicit checks,
/// while bare ambient `declare module "vue"` files without top-level imports or
/// exports shadow the real package.
fn declares_shadowing_ambient_module(content: &str) -> bool {
    if has_top_level_import_or_export(content) {
        return false;
    }

    ambient_module_specifiers(content)
        .iter()
        .any(|specifier| is_shadowed_vue_package_specifier(specifier))
}

fn ambient_module_specifiers(content: &str) -> Vec<std::string::String> {
    const NEEDLE: &str = "declare module";
    let mut specifiers = Vec::new();
    for (index, _) in content.match_indices(NEEDLE) {
        let preceded_by_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_alphanumeric() && ch != '_' && ch != '$');
        if !preceded_by_boundary {
            continue;
        }
        let mut chars = content[index + NEEDLE.len()..].chars();
        let Some(quote) = chars.find(|ch| !ch.is_whitespace()) else {
            continue;
        };
        if quote != '"' && quote != '\'' {
            continue;
        }
        let mut specifier = std::string::String::new();
        let mut escaped = false;
        for ch in chars {
            if escaped {
                specifier.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                specifiers.push(specifier);
                break;
            } else {
                specifier.push(ch);
            }
        }
    }
    specifiers
}

fn reference_path_declaration_files(
    path: &Path,
    content: &str,
    project_root: &Path,
) -> Vec<PathBuf> {
    let Some(base_dir) = path.parent() else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(reference_path_attribute)
        .filter_map(|reference| {
            let resolved = normalize_input_path(&base_dir.join(reference));
            (resolved.starts_with(project_root)
                && !path_has_component(&resolved, NODE_MODULES_DIR)
                && is_declaration_file(&resolved)
                && resolved.is_file())
            .then_some(resolved)
        })
        .collect()
}

fn reference_path_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "path")
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=");
    let start = line.find(&needle)? + needle.len();
    let quote = line[start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = line[value_start..].find(quote)? + value_start;
    line.get(value_start..value_end)
}

fn is_reference_manifest_declaration(content: &str) -> bool {
    let mut has_reference = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("///") && trimmed.contains("<reference") {
            has_reference = true;
            continue;
        }
        if matches!(trimmed, "export {}" | "export {};") {
            continue;
        }
        return false;
    }
    has_reference
}

fn has_top_level_import_or_export(content: &str) -> bool {
    content.lines().any(|line| {
        line.starts_with("import ")
            || line.starts_with("import{")
            || line.starts_with("export ")
            || line.starts_with("export{")
            || line.starts_with("export {}")
    })
}

fn is_shadowed_vue_package_specifier(specifier: &str) -> bool {
    matches!(
        specifier,
        "vue" | "@vue/runtime-core" | "@vue/runtime-dom" | "vue-router"
    )
}
