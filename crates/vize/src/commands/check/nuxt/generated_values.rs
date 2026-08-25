//! Generated Nuxt `declare global` value stubs.

use std::path::Path;

use vize_s0::{FxHashSet, String, ToCompactString};

use super::generated_ast::collect_import_type_specifiers;
use super::stubs::push_declared_const;

pub(super) fn push_generated_declared_global_const(
    cwd: &Path,
    type_origin: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
    name: &str,
    type_annotation: &str,
) {
    if name.starts_with('$') {
        external_template_bindings.insert(name.to_compact_string());
    }
    push_generated_declared_const(cwd, type_origin, stubs, seen_names, name, type_annotation);
}

pub(super) fn push_generated_declared_const(
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
