//! Shared resolution for relative, package, and tsconfig-path imports.

use std::path::{Component, Path, PathBuf};

use tower_lsp::lsp_types::Url;
use vize_carton::cstr;

use super::module_specifier;

pub(crate) fn resolve_import_specifier(uri: &Url, specifier: &str) -> Option<PathBuf> {
    if let Some(path) = module_specifier::resolve_specifier(uri, specifier) {
        return Some(path);
    }

    let file = uri.to_file_path().ok()?;
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return Some(normalize_absolute_path(
            file.parent()?.join(specifier).as_path(),
        ));
    }

    let paths = crate::ide::tsconfig_paths::project_paths(&file)?;
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
        if let Some(resolved) = probe(&base)
            && best.as_ref().is_none_or(|(len, _)| pattern.len() > *len)
        {
            best = Some((pattern.len(), resolved));
        }
    }
    best.map(|(_, path)| path)
}

fn probe(base: &Path) -> Option<PathBuf> {
    if base.extension().is_some() && base.is_file() {
        return Some(normalize_absolute_path(base));
    }
    for extension in ["ts", "tsx", "d.ts", "vue"] {
        let candidate = PathBuf::from(cstr!("{}.{extension}", base.display()).as_str());
        if candidate.is_file() {
            return Some(normalize_absolute_path(&candidate));
        }
    }
    let candidate = ["index.ts", "index.tsx"]
        .iter()
        .map(|index| base.join(index))
        .find(|candidate| candidate.is_file())?;
    Some(normalize_absolute_path(&candidate))
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
