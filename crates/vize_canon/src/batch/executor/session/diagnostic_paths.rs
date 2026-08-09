use super::VirtualProject;
use crate::file_uri::path_to_file_uri;
use std::path::Path;
use vize_carton::String;

pub(super) fn extend_diagnostic_path_uris(project: &VirtualProject, uris: &mut Vec<String>) {
    uris.extend(
        project
            .diagnostic_paths_sorted()
            .into_iter()
            .filter(|path| path.is_file() && is_authored_diagnostic_input(path))
            .map(|path| path_to_file_uri(&path)),
    );
    uris.sort();
    uris.dedup();
}

pub(super) fn is_authored_diagnostic_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs")
    )
}

#[cfg(test)]
mod tests {
    use super::extend_diagnostic_path_uris;
    use crate::{batch::VirtualProject, file_uri::path_to_file_uri};

    #[test]
    fn adds_only_existing_authored_source_uris() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source.js");
        let ignored = root.path().join("notes.txt");
        let missing = root.path().join("missing.ts");
        std::fs::write(&source, "export const value = 1\n").unwrap();
        std::fs::write(&ignored, "notes\n").unwrap();
        let mut project = VirtualProject::new(root.path()).unwrap();
        project.set_diagnostic_paths([source.as_path(), ignored.as_path(), missing.as_path()]);
        let mut uris = Vec::new();

        extend_diagnostic_path_uris(&project, &mut uris);

        assert_eq!(
            uris,
            vec![path_to_file_uri(&source.canonicalize().unwrap())]
        );
    }
}
