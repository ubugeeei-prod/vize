//! Shared resolution for relative, package, and tsconfig-path imports.

use std::path::{Component, Path, PathBuf};

use tower_lsp::lsp_types::Url;
use vize_l0::cstr;

use super::module_specifier;
use crate::document::DocumentStore;

#[cfg(test)]
#[path = "import_resolver_tests.rs"]
mod tests;

pub(crate) fn resolve_import_specifier(uri: &Url, specifier: &str) -> Option<PathBuf> {
    resolve_authored_import_specifier(uri, specifier)
        .or_else(|| module_specifier::resolve_specifier(uri, specifier))
}

/// Navigation can retain a project alias whose target is an editor-open file.
/// Filesystem-only consumers keep using `resolve_authored_import_specifier`.
pub(crate) fn resolve_import_specifier_with_documents(
    uri: &Url,
    specifier: &str,
    documents: &DocumentStore,
) -> Option<PathBuf> {
    resolve_authored_import_specifier_with_probe(uri, specifier, |base| {
        probe_with_open_files(base, |candidate| {
            // Only a complete disk-probe miss needs URI construction and lookup.
            // Normalize once, retaining the same path returned for disk files.
            let normalized = normalize_absolute_path(candidate);
            let uri = Url::from_file_path(&normalized).ok()?;
            documents.contains(&uri).then_some(normalized)
        })
    })
    .or_else(|| module_specifier::resolve_specifier(uri, specifier))
}

/// Resolve relative imports and project-owned aliases without adding links for
/// bare packages. Definition may additionally resolve those package routes.
pub(crate) fn resolve_authored_import_specifier(uri: &Url, specifier: &str) -> Option<PathBuf> {
    resolve_authored_import_specifier_with_probe(uri, specifier, probe)
}

fn resolve_authored_import_specifier_with_probe(
    uri: &Url,
    specifier: &str,
    alias_probe: impl Fn(&Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    let file = uri.to_file_path().ok()?;
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return module_specifier::resolve_specifier(uri, specifier).or_else(|| {
            Some(normalize_absolute_path(
                file.parent()?.join(specifier).as_path(),
            ))
        });
    }

    if let Some(paths) = crate::ide::tsconfig_paths::project_paths(&file) {
        let mut best: Option<(usize, PathBuf)> = None;
        for (pattern, target) in &paths.entries {
            let substituted = if let Some(prefix) = pattern.strip_suffix('*') {
                match (specifier.strip_prefix(prefix), target.strip_suffix('*')) {
                    (Some(rest), Some(target_prefix)) => {
                        Some(cstr!("{target_prefix}{rest}").to_string())
                    }
                    _ => None,
                }
            } else if specifier == pattern {
                Some(target.clone())
            } else {
                None
            };
            let Some(substituted) = substituted else {
                continue;
            };
            let base = paths.anchor.join(substituted);
            if let Some(resolved) = alias_probe(&base)
                && best.as_ref().is_none_or(|(len, _)| pattern.len() > *len)
            {
                best = Some((pattern.len(), resolved));
            }
        }
        if let Some((_, path)) = best {
            return Some(path);
        }
    }
    if let Some(path) = resolve_nuxt_source_alias(&file, specifier) {
        return Some(path);
    }
    if let Some(path) = resolve_project_source_alias(&file, specifier) {
        return Some(path);
    }
    None
}

fn resolve_nuxt_source_alias(file: &Path, specifier: &str) -> Option<PathBuf> {
    let rest = specifier
        .strip_prefix("~/")
        .or_else(|| specifier.strip_prefix("@/"))?;
    let root = nearest_nuxt_root(file)?;
    [
        root.join("app").join(rest),
        root.join(rest),
        root.join("src").join(rest),
    ]
    .into_iter()
    .find_map(|base| probe(&base))
}

fn nearest_nuxt_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| {
            [
                "nuxt.config.ts",
                "nuxt.config.mts",
                "nuxt.config.js",
                "nuxt.config.mjs",
            ]
            .iter()
            .any(|config| dir.join(config).is_file())
        })
        .map(Path::to_path_buf)
}

fn resolve_project_source_alias(file: &Path, specifier: &str) -> Option<PathBuf> {
    let rest = specifier.strip_prefix("@/")?;
    let root = nearest_project_src_root(file)?;
    probe(&root.join("src").join(rest))
}

fn nearest_project_src_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| dir.join("src").is_dir() && has_project_source_alias_marker(dir))
        .map(Path::to_path_buf)
}

fn has_project_source_alias_marker(dir: &Path) -> bool {
    dir.join("package.json").is_file()
        || [
            "vite.config.ts",
            "vite.config.mts",
            "vite.config.js",
            "vite.config.mjs",
            "vue.config.js",
        ]
        .iter()
        .any(|config| dir.join(config).is_file())
}

fn probe(base: &Path) -> Option<PathBuf> {
    probe_with(base, &|candidate| {
        candidate
            .is_file()
            .then(|| normalize_absolute_path(candidate))
    })
}

fn probe_with_open_files(
    base: &Path,
    resolve_open: impl Fn(&Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    // Preserve the complete existing disk fast path, including later extensions.
    // The fallback never repeats filesystem probes or visits document contents.
    probe(base).or_else(|| probe_with(base, &resolve_open))
}

fn probe_with(
    base: &Path,
    resolve_candidate: &impl Fn(&Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    if base.extension().is_some()
        && let Some(candidate) = resolve_candidate(base)
    {
        return Some(candidate);
    }
    for extension in ["ts", "tsx", "d.ts", "vue"] {
        let candidate = PathBuf::from(cstr!("{}.{extension}", base.display()).as_str());
        if let Some(candidate) = resolve_candidate(&candidate) {
            return Some(candidate);
        }
    }
    ["index.ts", "index.tsx"]
        .iter()
        .map(|index| base.join(index))
        .find_map(|candidate| resolve_candidate(&candidate))
}

fn normalize_absolute_path(path: &Path) -> PathBuf {
    debug_assert!(path.is_absolute());
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
