use std::path::Path;

use corsa::api::{FileChangeSummary, FileChanges};

use super::session::{external_document_path, uri_document_identifier};

pub(super) fn uri_inside_project(uri: &str, project_root: &Path) -> bool {
    external_document_path(uri)
        .is_some_and(|path| path.starts_with(project_root) && target_exists(&path))
}

pub(super) fn target_exists(external_path: &Path) -> bool {
    let Some(name) = external_path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(real_name) = name.strip_suffix(".ts") else {
        return false;
    };
    if !real_name.ends_with(".vue") && !real_name.ends_with(".html") {
        return false;
    }
    external_path.with_file_name(real_name).is_file()
}

pub(super) fn upsert_file_changes(
    external_uri: &str,
    document_uri: &str,
    project_root: &Path,
    was_open: bool,
) -> Option<FileChanges> {
    if document_uri != external_uri || !missing_from_disk(external_uri, project_root) {
        return None;
    }

    let document = uri_document_identifier(document_uri);
    Some(FileChanges::Summary(FileChangeSummary {
        changed: if was_open {
            vec![document.clone()]
        } else {
            Vec::new()
        },
        created: if was_open { Vec::new() } else { vec![document] },
        deleted: Vec::new(),
    }))
}

pub(super) fn delete_file_changes(
    external_uri: &str,
    document_uri: &str,
    project_root: &Path,
) -> Option<FileChanges> {
    if document_uri != external_uri || !missing_from_disk(external_uri, project_root) {
        return None;
    }

    Some(FileChanges::Summary(FileChangeSummary {
        changed: Vec::new(),
        created: Vec::new(),
        deleted: vec![uri_document_identifier(document_uri)],
    }))
}

fn missing_from_disk(uri: &str, project_root: &Path) -> bool {
    external_document_path(uri).is_some_and(|path| {
        path.starts_with(project_root) && !path.exists() && target_exists(&path)
    })
}

#[cfg(test)]
mod tests {
    use corsa::api::FileChanges;

    use super::{delete_file_changes, upsert_file_changes};
    use crate::file_uri::path_to_file_uri;

    #[test]
    fn virtual_vue_overlay_file_changes_do_not_materialize_sibling_files() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let project = std::env::temp_dir().join(format!(
            "vize-canon-virtual-overlay-changes-{}-{nonce}",
            std::process::id()
        ));
        let src = project.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("Panel.vue"), "<template><div /></template>").unwrap();

        let virtual_path = src.join("Panel.vue.ts");
        let uri = path_to_file_uri(&virtual_path);

        let created = upsert_file_changes(&uri, &uri, &project, false).expect("created change");
        let created = match created {
            FileChanges::Summary(summary) => summary,
            FileChanges::InvalidateAll { .. } => panic!("expected summary file changes"),
        };
        assert_eq!(created.created.len(), 1);
        assert!(created.changed.is_empty());
        assert!(!virtual_path.exists());

        let changed = upsert_file_changes(&uri, &uri, &project, true).expect("changed change");
        let changed = match changed {
            FileChanges::Summary(summary) => summary,
            FileChanges::InvalidateAll { .. } => panic!("expected summary file changes"),
        };
        assert_eq!(changed.changed.len(), 1);
        assert!(changed.created.is_empty());
        assert!(!virtual_path.exists());

        let deleted = delete_file_changes(&uri, &uri, &project).expect("deleted change");
        let deleted = match deleted {
            FileChanges::Summary(summary) => summary,
            FileChanges::InvalidateAll { .. } => panic!("expected summary file changes"),
        };
        assert_eq!(deleted.deleted.len(), 1);
        assert!(!virtual_path.exists());

        let _ = std::fs::remove_dir_all(project);
    }
}
