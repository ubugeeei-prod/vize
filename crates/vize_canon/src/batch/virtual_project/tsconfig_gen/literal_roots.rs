//! Program roots as a literal `files` list instead of `include` entries.
//!
//! The generated config names every program root by its exact path. As
//! `include` entries, tsgo treats each as a pattern: before it loads a file,
//! it walks the mirror and matches every file against every entry
//! (`getFileNamesFromConfigSpecs`), on one core. That is a quarter second for
//! n8n's 2,256 roots and grows with their square. A `files` list is taken as
//! written.
//!
//! The two forms give the same roots in the same order (the include matches
//! are grouped by entry, in entry order) except where the include walk would
//! skip an entry: a missing file, an extension the options do not load, a
//! file behind a symlinked directory the walk reaches once, or one of two
//! entries that differ only by a lower-priority extension (`a.ts` beside
//! `a.d.ts` or `a.js`), which only the include walk drops. Any of those keeps
//! the `include` form.

use std::path::{Component, Path};

use rayon::prelude::*;
use vize_carton::{FxHashSet, String as CompactString};

/// Root extensions in priority-free form, longest first per suffix.
const TYPESCRIPT_EXTENSIONS: &[&str] =
    &[".d.ts", ".d.cts", ".d.mts", ".ts", ".tsx", ".cts", ".mts"];
const JAVASCRIPT_EXTENSIONS: &[&str] = &[".js", ".jsx", ".cjs", ".mjs"];

/// Whether `entries`, relative to `virtual_root`, select the same program
/// as a `files` list as they do as `include` entries.
pub(super) fn files_select_include_roots(
    virtual_root: &Path,
    entries: &[CompactString],
    allow_javascript: bool,
) -> bool {
    if entries.is_empty() {
        return false;
    }
    let mut stems = FxHashSet::default();
    let mut directories = FxHashSet::default();
    for entry in entries {
        let Some(stem) = literal_stem(entry, allow_javascript) else {
            return false;
        };
        // Case-folded, so a case-insensitive file system cannot pair them.
        if !stems.insert(stem.to_lowercase()) {
            return false;
        }
        let mut directory = Path::new(entry.as_str()).parent();
        while let Some(path) = directory.filter(|path| !path.as_os_str().is_empty()) {
            if !directories.insert(path) {
                break;
            }
            directory = path.parent();
        }
    }
    let no_linked_directory = directories.par_iter().all(|directory| {
        std::fs::symlink_metadata(virtual_root.join(directory))
            .is_ok_and(|metadata| metadata.is_dir())
    });
    no_linked_directory
        && entries
            .par_iter()
            .all(|entry| virtual_root.join(entry.as_str()).is_file())
}

/// The entry without its root extension, when it is a plain relative path
/// to a file the options load.
fn literal_stem(entry: &str, allow_javascript: bool) -> Option<&str> {
    if entry.contains(['*', '?'])
        || !Path::new(entry)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let javascript = if allow_javascript {
        JAVASCRIPT_EXTENSIONS
    } else {
        &[]
    };
    TYPESCRIPT_EXTENSIONS
        .iter()
        .chain(javascript)
        .find_map(|extension| entry.strip_suffix(extension))
        .filter(|stem| !stem.is_empty() && !stem.ends_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(paths: &[&str]) -> Vec<CompactString> {
        paths
            .iter()
            .map(|path| CompactString::from(*path))
            .collect()
    }

    fn mirror(files: &[&str]) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        for file in files {
            let path = root.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "export {}\n").unwrap();
        }
        root
    }

    #[test]
    fn existing_typescript_roots_become_files() {
        let files = [
            "__vize_helpers.d.ts",
            "src/App.vue.ts",
            "src/a/b.tsx",
            "src/c.cts",
        ];
        let root = mirror(&files);
        assert!(files_select_include_roots(
            root.path(),
            &entries(&files),
            false
        ));
    }

    #[test]
    fn roots_the_include_walk_would_skip_keep_include() {
        let root = mirror(&["src/a.ts", "src/a.d.ts", "src/b.js", "src/c.json"]);
        let keeps_include = |paths: &[&str], allow_js| {
            !files_select_include_roots(root.path(), &entries(paths), allow_js)
        };
        assert!(keeps_include(&[], false));
        // Missing on disk.
        assert!(keeps_include(&["src/a.ts", "src/missing.ts"], false));
        // `a.d.ts` loses to `a.ts` in the include walk only.
        assert!(keeps_include(&["src/a.d.ts", "src/a.ts"], false));
        // JavaScript without allowJs, and extensions no program loads.
        assert!(keeps_include(&["src/b.js"], false));
        assert!(keeps_include(&["src/c.json"], true));
        // Patterns and paths leaving the root.
        assert!(keeps_include(&["src/*.ts"], false));
        assert!(keeps_include(&["../a.ts"], false));
        assert!(!keeps_include(&["src/a.ts", "src/b.js"], true));
    }

    #[test]
    fn only_a_cli_materialization_lists_its_roots_as_files() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path().canonicalize().unwrap();
        let source = root.join("src/main.ts");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "export const answer = 42;\n").unwrap();
        let mut project = crate::batch::VirtualProject::new(&root).unwrap();
        project.register_path(&source).unwrap();
        let config = || -> serde_json::Value {
            serde_json::from_slice(&std::fs::read(project.generated_tsconfig_path()).unwrap())
                .unwrap()
        };

        project.materialize_for_cli().unwrap();
        let literal = config();
        assert!(literal.get("include").is_none() && literal.get("exclude").is_none());
        let files = literal["files"].as_array().unwrap();
        assert!(files.iter().any(|entry| entry == "src/main.ts"));

        project.materialize().unwrap();
        let patterns = config();
        assert!(patterns.get("files").is_none());
        assert_eq!(&patterns["include"], &literal["files"]);
        assert_eq!(patterns["exclude"], serde_json::json!([]));
    }

    #[cfg(unix)]
    #[test]
    fn a_root_behind_a_linked_directory_keeps_include() {
        let root = mirror(&["real/a.ts"]);
        std::os::unix::fs::symlink(root.path().join("real"), root.path().join("linked")).unwrap();
        assert!(!files_select_include_roots(
            root.path(),
            &entries(&["linked/a.ts"]),
            false
        ));
        assert!(files_select_include_roots(
            root.path(),
            &entries(&["real/a.ts"]),
            false
        ));
    }
}
