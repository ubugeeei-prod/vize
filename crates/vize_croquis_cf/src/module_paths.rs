//! Deterministic logical module-path resolution shared by cross-file rules.

use std::path::{Component, Path, PathBuf};

/// Build normalized source candidates for an import specifier.
pub(crate) fn import_candidates(specifier: &str, from_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut bases = Vec::new();

    if let Some(relative) = specifier.strip_prefix("@/") {
        bases.push(PathBuf::from("src").join(relative));
    } else if specifier.starts_with('.') {
        let base = from_dir
            .filter(|dir| !dir.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from(specifier), |dir| dir.join(specifier));
        bases.push(base);
    } else if let Some(stripped) = specifier.strip_prefix('/') {
        bases.push(PathBuf::from(stripped));
        bases.push(PathBuf::from(specifier));
    } else {
        bases.push(PathBuf::from(specifier));
    }

    let mut candidates = Vec::new();
    for base in bases {
        candidates.push(normalize_logical_path(base.clone()));

        if base.extension().is_none() {
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

/// Collapse `.` and `..` segments without accessing the host filesystem.
pub(crate) fn normalize_logical_path(path: PathBuf) -> PathBuf {
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
    use super::import_candidates;
    use std::path::{Path, PathBuf};

    #[test]
    fn relative_candidates_preserve_directory_and_extension_order() {
        let candidates = import_candidates("../primitive", Some(Path::new("src/components")));

        assert_eq!(candidates[0], PathBuf::from("src/primitive"));
        assert_eq!(candidates[1], PathBuf::from("src/primitive.vue"));
        assert!(candidates.contains(&PathBuf::from("src/primitive/index.ts")));
    }

    #[test]
    fn source_alias_candidates_begin_at_the_source_root() {
        let candidates = import_candidates("@/components/Button", None);

        assert_eq!(candidates[0], PathBuf::from("src/components/Button"));
        assert!(candidates.contains(&PathBuf::from("src/components/Button.vue")));
    }
}
