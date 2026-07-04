//! TypeScript-like relative import resolution for CLI shard partitioning.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, cstr};

const VIRTUAL_IMPORT_SUFFIXES: &[&str] = &[
    ".ts",
    ".tsx",
    ".mts",
    ".cts",
    ".d.ts",
    ".d.mts",
    ".d.cts",
    "/index.ts",
    "/index.tsx",
    "/index.mts",
    "/index.cts",
    "/index.d.ts",
    "/index.d.mts",
    "/index.d.cts",
];

/// Resolve a normalized relative import target against the registered virtual
/// files, trying the extension candidates TypeScript would.
pub(super) fn resolve_virtual_import(
    target: &Path,
    index_by_virtual: &FxHashMap<&Path, usize>,
) -> Option<usize> {
    if let Some(&index) = index_by_virtual.get(target) {
        return Some(index);
    }
    let target_str = target.to_string_lossy();
    for suffix in VIRTUAL_IMPORT_SUFFIXES {
        let candidate = PathBuf::from(cstr!("{target_str}{suffix}").as_str());
        if let Some(&index) = index_by_virtual.get(candidate.as_path()) {
            return Some(index);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use vize_carton::FxHashMap;

    use super::resolve_virtual_import;

    #[test]
    fn resolves_module_and_declaration_candidates() {
        let paths = [
            "src/module.mts",
            "src/common.cts",
            "src/schema.d.mts",
            "src/legacy/index.d.cts",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
        let index_by_virtual = paths
            .iter()
            .enumerate()
            .map(|(index, path)| (path.as_path(), index))
            .collect::<FxHashMap<&Path, usize>>();

        assert_eq!(
            resolve_virtual_import(Path::new("src/module"), &index_by_virtual),
            Some(0)
        );
        assert_eq!(
            resolve_virtual_import(Path::new("src/common"), &index_by_virtual),
            Some(1)
        );
        assert_eq!(
            resolve_virtual_import(Path::new("src/schema"), &index_by_virtual),
            Some(2)
        );
        assert_eq!(
            resolve_virtual_import(Path::new("src/legacy"), &index_by_virtual),
            Some(3)
        );
    }
}
