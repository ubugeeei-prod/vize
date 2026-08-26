//! Shared resolution for relative, package, and tsconfig-path imports.

use std::path::{Component, Path, PathBuf};

use tower_lsp::lsp_types::Url;
use vize_s0::cstr;

use super::module_specifier;

#[cfg(test)]
#[path = "import_resolver_tests.rs"]
mod tests;

pub(crate) fn resolve_import_specifier(uri: &Url, specifier: &str) -> Option<PathBuf> {
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
            if let Some(resolved) = probe(&base)
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
    module_specifier::resolve_specifier(uri, specifier)
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
