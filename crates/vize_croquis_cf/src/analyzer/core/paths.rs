use std::path::{Component, Path, PathBuf};
use vize_carton::String;

pub(super) fn import_candidates(specifier: &str, from_dir: Option<&Path>) -> Vec<PathBuf> {
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

pub(super) fn component_names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

fn to_pascal_case(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::default(),
            }
        })
        .collect()
}
