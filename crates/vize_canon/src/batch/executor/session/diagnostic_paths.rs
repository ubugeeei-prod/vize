use super::VirtualProject;
use crate::file_uri::path_to_file_uri;
use std::path::{Path, PathBuf};
use vize_carton::{FxHashSet, String};

pub(super) fn extend_diagnostic_path_uris(project: &VirtualProject, uris: &mut Vec<String>) {
    uris.extend(
        unmirrored_authored_diagnostic_paths(project)
            .into_iter()
            .map(|path| path_to_file_uri(&path)),
    );
    uris.sort();
    uris.dedup();
}

/// Authored paths that are not already copied into the virtual project.
///
/// Reopening a mirrored `.ts` file typechecks its unrewritten `./Widget.vue`
/// import, which binds `declare module "*.vue"` once a parser error forces
/// session completion (#10353).
pub(super) fn unmirrored_authored_diagnostic_paths(project: &VirtualProject) -> Vec<PathBuf> {
    let mirrored = mirrored_authored_paths(project);
    project
        .diagnostic_paths_sorted()
        .into_iter()
        .filter(|path| {
            path.is_file() && is_authored_diagnostic_input(path) && !mirrored.contains(path)
        })
        .collect()
}

fn mirrored_authored_paths(project: &VirtualProject) -> FxHashSet<PathBuf> {
    let mut mirrored = FxHashSet::default();
    for path in project.registered_original_paths_sorted() {
        mirrored.insert(vize_carton::path::canonicalize_non_verbatim(&path));
        mirrored.insert(path);
    }
    mirrored
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

    #[test]
    fn issue_10353_skips_mirrored_authored_diagnostic_uris() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let source = root.join("use.ts");
        let widget = root.join("Widget.vue");
        let notes = root.join("notes.ts");
        std::fs::write(
            &source,
            "import Widget from \"./Widget.vue\";\nexport const value = 1\n",
        )
        .unwrap();
        std::fs::write(
            &widget,
            "<script setup lang=\"ts\">\nexport interface Item { id: number }\n</script>\n",
        )
        .unwrap();
        std::fs::write(&notes, "export const notes = 1\n").unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.register_path(&source).unwrap();
        project.register_path(&widget).unwrap();
        project.set_diagnostic_paths([source.as_path(), widget.as_path(), notes.as_path()]);
        let mut uris = Vec::new();

        extend_diagnostic_path_uris(&project, &mut uris);

        assert_eq!(uris, vec![path_to_file_uri(&notes)]);
        assert!(project.find_by_original(&source).is_some());
    }
}
