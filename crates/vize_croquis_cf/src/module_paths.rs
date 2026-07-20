//! Deterministic logical module-path resolution shared by cross-file rules.

use std::path::{Component, Path, PathBuf};

/// Build normalized source candidates for an import specifier.
///
/// Authored imports can retain the extension used by emitted runtime files.
/// The corresponding source extensions are therefore checked after the exact
/// path, without consulting the host filesystem.
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
                ".mts",
                ".cts",
                ".js",
                ".jsx",
                ".mjs",
                ".cjs",
                "/index.vue",
                "/index.ts",
                "/index.tsx",
                "/index.mts",
                "/index.cts",
                "/index.js",
                "/index.jsx",
                "/index.mjs",
                "/index.cjs",
            ] {
                candidates.push(normalize_logical_path(path_with_suffix(&base, suffix)));
            }
        } else {
            push_runtime_source_substitutions(&base, &mut candidates);
        }
    }

    candidates
}

fn push_runtime_source_substitutions(base: &Path, candidates: &mut Vec<PathBuf>) {
    let Some(extension) = base.extension().and_then(|value| value.to_str()) else {
        return;
    };
    let source_extensions: &[&str] = match extension {
        "js" => &["ts", "tsx"],
        "jsx" => &["tsx", "ts"],
        "mjs" => &["mts"],
        "cjs" => &["cts"],
        _ => &[],
    };

    for source_extension in source_extensions {
        candidates.push(normalize_logical_path(
            base.with_extension(source_extension),
        ));
    }
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

    #[test]
    fn runtime_extension_candidates_include_authored_sources() {
        let candidates = import_candidates("../primitive.js", Some(Path::new("src/components")));

        assert_eq!(candidates[0], PathBuf::from("src/primitive.js"));
        assert!(candidates.contains(&PathBuf::from("src/primitive.ts")));
        assert!(candidates.contains(&PathBuf::from("src/primitive.tsx")));
    }

    #[test]
    fn module_runtime_extensions_map_to_matching_sources() {
        let module = import_candidates("./worker.mjs", Some(Path::new("src")));
        let common = import_candidates("./worker.cjs", Some(Path::new("src")));

        assert!(module.contains(&PathBuf::from("src/worker.mts")));
        assert!(common.contains(&PathBuf::from("src/worker.cts")));
    }
}
