//! Deterministic file discovery for native lint requests.

use super::super::lint_fix::is_lintable_extension;
use glob::glob;
use ignore::Walk;
use std::path::PathBuf;

/// Discover lintable paths, then sort and deduplicate overlapping inputs.
pub(super) fn collect_lint_files(patterns: &[String]) -> Vec<PathBuf> {
    let mut files = patterns
        .iter()
        .flat_map(|pattern| {
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                glob(pattern)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(Result::ok)
                    .filter(|path| {
                        path.extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(is_lintable_extension)
                            && !path
                                .components()
                                .any(|component| component.as_os_str() == "node_modules")
                    })
                    .collect::<Vec<_>>()
            } else {
                Walk::new(pattern)
                    .filter_map(Result::ok)
                    .filter(|entry| {
                        entry
                            .path()
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(is_lintable_extension)
                    })
                    .map(|entry| entry.path().to_path_buf())
                    .collect::<Vec<_>>()
            }
        })
        .collect::<Vec<_>>();
    sort_unique_file_paths(&mut files);
    files
}

fn sort_unique_file_paths(files: &mut Vec<PathBuf>) {
    files.sort_unstable();
    files.dedup();
}

#[cfg(test)]
mod tests {
    use super::{collect_lint_files, sort_unique_file_paths};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn overlapping_inputs_are_linted_once_in_stable_order() {
        let mut files = vec![
            PathBuf::from("src/Zeta.vue"),
            PathBuf::from("src/Alpha.vue"),
            PathBuf::from("src/Zeta.vue"),
        ];

        sort_unique_file_paths(&mut files);

        assert_eq!(
            files,
            vec![
                PathBuf::from("src/Alpha.vue"),
                PathBuf::from("src/Zeta.vue")
            ]
        );
    }

    #[test]
    fn directory_collection_includes_script_files() {
        let directory = tempfile::tempdir().unwrap();
        let source_dir = directory.path().join("src");
        fs::create_dir_all(&source_dir).unwrap();
        for filename in [
            "App.vue",
            "index.html",
            "tokens.js",
            "tokens.mjs",
            "tokens.cjs",
            "tokens.ts",
            "tokens.mts",
            "tokens.cts",
            "tokens.jsx",
            "tokens.tsx",
            "README.md",
        ] {
            fs::write(source_dir.join(filename), "").unwrap();
        }

        let files = collect_lint_files(&[source_dir.to_string_lossy().into_owned()]);
        let names = files
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "App.vue",
                "index.html",
                "tokens.cjs",
                "tokens.cts",
                "tokens.js",
                "tokens.jsx",
                "tokens.mjs",
                "tokens.mts",
                "tokens.ts",
                "tokens.tsx",
            ]
        );
    }
}
