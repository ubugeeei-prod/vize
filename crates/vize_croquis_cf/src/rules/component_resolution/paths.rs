use crate::registry::ModuleRegistry;
use std::path::{Component, Path, PathBuf};

/// Try to resolve an import specifier to a file in the registry.
#[allow(clippy::disallowed_macros)]
pub(super) fn resolve_import(
    specifier: &str,
    registry: &ModuleRegistry,
    from_dir: Option<&Path>,
) -> bool {
    // Handle @/ alias (common Vue project alias)
    if let Some(relative) = specifier.strip_prefix("@/") {
        // Check with common extensions
        for ext in &["", ".vue", ".ts", ".tsx", ".js", ".jsx"] {
            let path = format!("src/{}{}", relative, ext);
            if registry.get_by_path(&path).is_some() {
                return true;
            }
        }
        return false;
    }

    // Resolve relative imports against the importing file's directory. Matching
    // only canonical paths keeps `./Button.vue` scoped to its sibling instead of
    // a different directory's same-named file.
    if specifier.starts_with('.') {
        let candidates = import_candidates(specifier, from_dir);
        return registry.iter().any(|entry| {
            let entry_path = normalize_logical_path(entry.path.clone());
            candidates.contains(&entry_path)
        });
    }

    // For absolute or other paths, check directly
    registry.get_by_path(specifier).is_some()
}

/// Build the set of canonical candidate paths a relative import specifier may
/// resolve to, trying the common module extensions and `index` files.
fn import_candidates(specifier: &str, from_dir: Option<&Path>) -> Vec<PathBuf> {
    let base = from_dir
        .filter(|dir| !dir.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from(specifier), |dir| dir.join(specifier));

    let mut candidates = Vec::new();
    let has_extension = base.extension().is_some();
    candidates.push(normalize_logical_path(base.clone()));

    if !has_extension {
        for suffix in [
            ".vue",
            ".ts",
            ".tsx",
            ".js",
            ".jsx",
            "/index.vue",
            "/index.ts",
            "/index.tsx",
            "/index.js",
            "/index.jsx",
        ] {
            candidates.push(normalize_logical_path(path_with_suffix(&base, suffix)));
        }
    }

    candidates
}

fn path_with_suffix(base: &Path, suffix: &str) -> PathBuf {
    if let Some(index_file) = suffix.strip_prefix('/') {
        base.join(index_file)
    } else {
        let mut value = base.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    }
}

/// Normalize a path by collapsing `.`/`..` segments without touching the
/// filesystem, yielding a canonical logical path for comparison.
fn normalize_logical_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::resolve_import;
    use crate::registry::ModuleRegistry;
    use std::path::Path;
    use vize_croquis::Croquis;

    #[test]
    fn relative_import_resolves_to_sibling_not_same_named_file_elsewhere() {
        let mut registry = ModuleRegistry::new();

        // Two components named `Button.vue` in different directories.
        registry.register("pages/Button.vue", "", Croquis::new());
        registry.register("admin/Button.vue", "", Croquis::new());

        // `./Button.vue` imported from `pages/Home.vue` must resolve to the
        // sibling `pages/Button.vue`.
        let from_dir = Path::new("pages/Home.vue").parent();
        assert!(resolve_import("./Button.vue", &registry, from_dir));
    }

    #[test]
    fn relative_import_does_not_cross_directories() {
        let mut registry = ModuleRegistry::new();

        // Only the `admin/` variant exists; the sibling does not.
        registry.register("admin/Button.vue", "", Croquis::new());

        // `./Button.vue` imported from `pages/Home.vue` must NOT resolve to
        // `admin/Button.vue` via a suffix match.
        let from_dir = Path::new("pages/Home.vue").parent();
        assert!(!resolve_import("./Button.vue", &registry, from_dir));
    }

    #[test]
    fn relative_import_resolves_parent_directory() {
        let mut registry = ModuleRegistry::new();

        registry.register("components/Button.vue", "", Croquis::new());

        // `../Button.vue` from `components/forms/Field.vue` resolves to
        // `components/Button.vue`.
        let from_dir = Path::new("components/forms/Field.vue").parent();
        assert!(resolve_import("../Button.vue", &registry, from_dir));
    }
}
