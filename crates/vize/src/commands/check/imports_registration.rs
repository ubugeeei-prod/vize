use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::super::imports_aliases::PathAliasResolver;
use super::super::path_cache::CanonicalPathCache;
use super::{
    extract_import_specifiers, is_declaration_file, is_node_modules_path, is_relative_specifier,
    resolve_import_base, resolve_relative_import,
};

pub(super) fn non_relative_import_needs_virtual_registration(
    path: &Path,
    canonical_paths: &mut CanonicalPathCache,
    include_jsx: bool,
    aliases: Option<&PathAliasResolver>,
    cache: &mut FxHashMap<PathBuf, bool>,
) -> bool {
    if let Some(needs_registration) = cache.get(path) {
        return *needs_registration;
    }

    let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
    let mut queue = vec![path.to_path_buf()];
    let needs_registration = source_needs_virtual_registration(
        &mut visited,
        &mut queue,
        canonical_paths,
        include_jsx,
        aliases,
    );
    cache.insert(path.to_path_buf(), needs_registration);
    needs_registration
}

fn source_needs_virtual_registration(
    visited: &mut FxHashSet<PathBuf>,
    queue: &mut Vec<PathBuf>,
    canonical_paths: &mut CanonicalPathCache,
    include_jsx: bool,
    aliases: Option<&PathAliasResolver>,
) -> bool {
    while let Some(file) = queue.pop() {
        if !visited.insert(file.clone()) {
            continue;
        }
        if file.extension().and_then(|extension| extension.to_str()) == Some("vue") {
            return true;
        }

        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Some(dir) = file.parent() else {
            continue;
        };

        for specifier in extract_import_specifiers(&source) {
            let candidate = Path::new(specifier.as_str());
            let resolved = if is_relative_specifier(&specifier) {
                resolve_relative_import(dir, &specifier, canonical_paths, include_jsx)
            } else if candidate.is_absolute() {
                resolve_import_base(candidate, canonical_paths, include_jsx)
            } else {
                aliases.and_then(|aliases| {
                    aliases.resolve(
                        &specifier,
                        canonical_paths,
                        include_jsx,
                        resolve_import_base,
                    )
                })
            };
            let Some(resolved) = resolved else {
                continue;
            };
            if is_declaration_file(&resolved) || is_node_modules_path(&resolved) {
                continue;
            }
            if resolved
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("vue")
            {
                return true;
            }
            if !visited.contains(&resolved) {
                queue.push(resolved);
            }
        }
    }

    false
}
