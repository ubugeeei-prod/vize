//! Shared enumeration of the repository Vue fixtures.

extern crate std;

use self::std::{
    path::{Path, PathBuf},
    vec::Vec,
};

/// Repository trees holding `.vue` files, relative to the workspace root.
const CORPUS: &[&str] = &[
    "tests/_fixtures",
    "tests/fixtures",
    "crates",
    "playground",
    "examples",
    "npm",
];
const SKIPPED_DIRECTORIES: &[&str] = &["node_modules", "target", "dist", ".git"];

fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if !SKIPPED_DIRECTORIES.iter().any(|skip| name == *skip) {
                collect(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "vue") {
            files.push(path);
        }
    }
}

/// The workspace root and every corpus `.vue` file, sorted.
pub fn corpus() -> (PathBuf, Vec<PathBuf>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for tree in CORPUS {
        collect(&root.join(tree), &mut files);
    }
    files.sort();
    (root, files)
}
