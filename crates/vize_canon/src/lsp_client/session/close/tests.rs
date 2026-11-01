use super::close_materialized_document;
use crate::{
    file_uri::path_to_file_uri,
    lsp_client::{CorsaProjectClient, session_storage::SessionStorage},
};
use corsa::api::{FileChangeSummary, FileChanges};
use std::{fs, path::PathBuf, sync::Arc};

const DISK: &[u8] = b"export const value = 1;\r\n";
const OVERLAY: &[u8] = b"export const value = 'unsaved';\r\n";

struct Fixture {
    _directory: tempfile::TempDir,
    root: PathBuf,
    storage: Arc<SessionStorage>,
    original: PathBuf,
    mirror: PathBuf,
}

impl Fixture {
    fn new(relative: &str, disk: Option<&[u8]>) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let original = root.join(relative);
        fs::create_dir_all(original.parent().unwrap()).unwrap();
        if let Some(bytes) = disk {
            fs::write(&original, bytes).unwrap();
        }
        let storage = Arc::new(SessionStorage::new(&root).unwrap());
        let mirror = storage.root().join(relative);
        fs::create_dir_all(mirror.parent().unwrap()).unwrap();
        fs::write(&mirror, OVERLAY).unwrap();
        Self {
            _directory: directory,
            root,
            storage,
            original,
            mirror,
        }
    }

    fn close(&self) -> Result<Option<FileChanges>, vize_l0::String> {
        close_materialized_document(
            &path_to_file_uri(&self.original),
            &path_to_file_uri(&self.mirror),
            &self.root,
            self.storage.root(),
        )
    }

    fn client(&self) -> CorsaProjectClient {
        let mut client = CorsaProjectClient::empty_for_test(self.root.clone());
        client.overlay_storage = Arc::clone(&self.storage);
        client.materialized_project_session = true;
        let uri = path_to_file_uri(&self.original);
        let mapped = path_to_file_uri(&self.mirror);
        client
            .document_texts
            .insert(uri.clone(), "unsaved overlay".into());
        client.overlay_versions.insert(uri.clone(), 7);
        client.diagnostics.insert(uri.clone(), Vec::new());
        client.remember_session_document_uri(&uri, mapped);
        client.editor_lsp_documents_dirty = false;
        client
    }
}

fn summary(changes: Option<FileChanges>) -> FileChangeSummary {
    let Some(FileChanges::Summary(summary)) = changes else {
        panic!("complete file-change summary required");
    };
    summary
}

#[test]
fn disk_close_restores_current_complete_bytes_and_preserves_original_metadata() {
    let fixture = Fixture::new("src/support.ts", Some(DISK));
    let current = b"// current disk bytes\r\nexport const value = 2;\r\n\xff";
    fs::write(&fixture.original, current).unwrap();
    let modified = fs::metadata(&fixture.original).unwrap().modified().unwrap();
    let delta = summary(fixture.close().unwrap());
    assert_eq!(
        delta.changed,
        vec![super::uri_document_identifier(&path_to_file_uri(
            &fixture.mirror
        ))]
    );
    assert!(delta.created.is_empty());
    assert!(delta.deleted.is_empty());
    assert_eq!(fs::read(&fixture.mirror).unwrap(), current);
    assert_eq!(fs::read(&fixture.original).unwrap(), current);
    assert_eq!(
        fs::metadata(&fixture.original).unwrap().modified().unwrap(),
        modified
    );
}

#[test]
fn disk_close_recreates_a_missing_owned_member_as_created_topology() {
    let fixture = Fixture::new("src/support.ts", Some(DISK));
    fs::remove_file(&fixture.mirror).unwrap();
    let delta = summary(fixture.close().unwrap());
    assert!(delta.changed.is_empty());
    assert!(delta.deleted.is_empty());
    assert_eq!(
        delta.created,
        vec![super::uri_document_identifier(&path_to_file_uri(
            &fixture.mirror
        ))]
    );
    assert_eq!(fs::read(&fixture.mirror).unwrap(), DISK);
    assert_eq!(fs::read(&fixture.original).unwrap(), DISK);
}

#[test]
fn closed_disk_dependency_keeps_navigation_identity_but_retires_active_overlay_state() {
    let fixture = Fixture::new("src/support.ts", Some(DISK));
    let uri = path_to_file_uri(&fixture.original);
    let mapped = path_to_file_uri(&fixture.mirror);
    let mut client = fixture.client();
    for _ in 0..2 {
        client.did_close(&uri).unwrap();
        assert!(!client.document_texts.contains_key(uri.as_str()));
        assert!(!client.overlay_versions.contains_key(uri.as_str()));
        assert!(!client.diagnostics.contains_key(uri.as_str()));
        assert_eq!(
            client.session_document_uris.get(uri.as_str()),
            Some(&mapped)
        );
        assert_eq!(
            client.external_document_uris.get(mapped.as_str()),
            Some(&uri)
        );
        let mut definition = serde_json::json!({"uri":mapped,"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}}});
        client.remap_result_uris(&mut definition);
        assert_eq!(definition["uri"], uri.as_str());
        assert_eq!(fs::read(&fixture.mirror).unwrap(), DISK);
        assert_eq!(fs::read(&fixture.original).unwrap(), DISK);
    }
    assert!(client.editor_lsp_documents_dirty);
    client
        .did_open(&uri, "export const next = true;\n")
        .unwrap();
    assert_eq!(
        fs::read(&fixture.mirror).unwrap(),
        b"export const next = true;\n"
    );
    client.did_close(&uri).unwrap();
    assert_eq!(fs::read(&fixture.mirror).unwrap(), DISK);
    assert_eq!(fs::read(&fixture.original).unwrap(), DISK);
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
}

#[test]
fn absent_virtual_and_deleted_disk_sources_remove_owned_mirror_and_identity_maps() {
    for relative in ["src/Panel.vue.ts", "src/Panel.vue.tsx", "src/removed.ts"] {
        let fixture = Fixture::new(relative, Some(DISK));
        fs::remove_file(&fixture.original).unwrap();
        let vue = fixture.root.join("src/Panel.vue");
        fs::write(&vue, b"<template>authored Vue remains</template>\n").unwrap();
        let mut client = fixture.client();
        let uri = path_to_file_uri(&fixture.original);
        let mapped = path_to_file_uri(&fixture.mirror);
        client.did_close(&uri).unwrap();
        assert!(!fixture.original.exists());
        assert!(!fixture.mirror.exists());
        assert!(!client.document_texts.contains_key(uri.as_str()));
        assert!(!client.overlay_versions.contains_key(uri.as_str()));
        assert!(!client.diagnostics.contains_key(uri.as_str()));
        assert!(!client.session_document_uris.contains_key(uri.as_str()));
        assert!(!client.external_document_uris.contains_key(mapped.as_str()));
        assert_eq!(
            fs::read(vue).unwrap(),
            b"<template>authored Vue remains</template>\n"
        );
        assert!(fixture.close().unwrap().is_none());
    }
}

#[test]
fn synthetic_close_reports_only_owned_deletion() {
    let fixture = Fixture::new("src/Virtual.vue.ts", None);
    let delta = summary(fixture.close().unwrap());
    assert!(delta.changed.is_empty());
    assert!(delta.created.is_empty());
    assert_eq!(
        delta.deleted,
        vec![super::uri_document_identifier(&path_to_file_uri(
            &fixture.mirror
        ))]
    );
    assert!(!fixture.mirror.exists());
    assert!(!fixture.original.exists());
}

#[test]
fn physical_identity_and_non_owned_targets_cannot_modify_authored_inputs() {
    let fixture = Fixture::new("src/support.ts", Some(DISK));
    let uri = path_to_file_uri(&fixture.original);
    assert!(
        close_materialized_document(&uri, &uri, &fixture.root, fixture.storage.root())
            .unwrap()
            .is_none()
    );
    let wrong = fixture.root.join("owned-sibling.ts");
    fs::write(&wrong, b"unrelated authored bytes\n").unwrap();
    assert!(
        close_materialized_document(
            &uri,
            &path_to_file_uri(&wrong),
            &fixture.root,
            fixture.storage.root()
        )
        .is_err()
    );
    assert_eq!(fs::read(&wrong).unwrap(), b"unrelated authored bytes\n");
    assert_eq!(fs::read(&fixture.original).unwrap(), DISK);
    assert_eq!(fs::read(&fixture.mirror).unwrap(), OVERLAY);
}

#[cfg(unix)]
#[test]
fn linked_mirror_root_parent_and_leaf_refuse_restore_and_delete_without_touching_targets() {
    use std::os::unix::fs::symlink;
    for disk_backed in [true, false] {
        for linked in ["owner", "root", "parent", "leaf"] {
            let fixture = Fixture::new("src/support.ts", disk_backed.then_some(DISK));
            let foreign = tempfile::tempdir().unwrap();
            fs::create_dir_all(foreign.path().join("src")).unwrap();
            let target = match linked {
                "owner" => {
                    let directory = fixture.storage.root().parent().unwrap();
                    fs::remove_dir_all(directory).unwrap();
                    fs::create_dir_all(foreign.path().join("project/src")).unwrap();
                    symlink(foreign.path(), directory).unwrap();
                    foreign.path().join("project/src/support.ts")
                }
                "root" => {
                    let root = fixture.storage.root();
                    fs::remove_dir_all(root).unwrap();
                    symlink(foreign.path(), root).unwrap();
                    foreign.path().join("src/support.ts")
                }
                "parent" => {
                    let parent = fixture.mirror.parent().unwrap();
                    fs::remove_dir_all(parent).unwrap();
                    symlink(foreign.path(), parent).unwrap();
                    foreign.path().join("support.ts")
                }
                _ => {
                    let target = foreign.path().join("support.ts");
                    fs::remove_file(&fixture.mirror).unwrap();
                    symlink(&target, &fixture.mirror).unwrap();
                    target
                }
            };
            fs::write(&target, b"foreign target bytes\n").unwrap();
            let mut client = fixture.client();
            let buffers = client.document_texts.clone();
            let versions = client.overlay_versions.clone();
            let mappings = client.session_document_uris.clone();
            let reverse = client.external_document_uris.clone();
            assert!(
                client
                    .did_close(&path_to_file_uri(&fixture.original))
                    .is_err()
            );
            assert_eq!(client.document_texts, buffers);
            assert_eq!(client.overlay_versions, versions);
            assert_eq!(client.session_document_uris, mappings);
            assert_eq!(client.external_document_uris, reverse);
            assert!(
                client
                    .diagnostics
                    .contains_key(path_to_file_uri(&fixture.original).as_str())
            );
            assert!(!client.editor_lsp_documents_dirty);
            assert_eq!(fs::read(&target).unwrap(), b"foreign target bytes\n");
            if disk_backed {
                assert_eq!(fs::read(&fixture.original).unwrap(), DISK);
            } else {
                assert!(!fixture.original.exists());
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn replacing_hardlinked_mirror_preserves_other_inode_owners() {
    use std::os::unix::fs::MetadataExt;
    let fixture = Fixture::new("src/support.ts", Some(DISK));
    let foreign = tempfile::tempdir().unwrap();
    let target = foreign.path().join("linked.ts");
    fs::write(&target, b"other inode owner\n").unwrap();
    let modified = fs::metadata(&target).unwrap().modified().unwrap();
    fs::remove_file(&fixture.mirror).unwrap();
    fs::hard_link(&target, &fixture.mirror).unwrap();
    fixture.close().unwrap();
    assert_eq!(fs::read(&fixture.mirror).unwrap(), DISK);
    assert_eq!(fs::read(&target).unwrap(), b"other inode owner\n");
    assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), modified);
    assert_ne!(
        fs::metadata(&fixture.mirror).unwrap().ino(),
        fs::metadata(&target).unwrap().ino()
    );
    assert_eq!(fs::read(&fixture.original).unwrap(), DISK);
}

#[cfg(unix)]
#[test]
fn canonical_project_alias_may_restore_inside_its_same_owned_tree() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("src/support.ts", Some(DISK));
    let directory = tempfile::tempdir().unwrap();
    let alias = directory.path().join("project-alias");
    symlink(&fixture.root, &alias).unwrap();
    // Workspace constructors canonicalize legitimate aliases before retaining
    // the project root; the close guard must use the same owned boundary.
    let project_root = alias.canonicalize().unwrap();
    let external_uri = path_to_file_uri(&alias.join("src/support.ts"));
    let mapped_uri = path_to_file_uri(&fixture.storage.root().join("src/support.ts"));
    assert!(
        close_materialized_document(
            &external_uri,
            &mapped_uri,
            &project_root,
            fixture.storage.root()
        )
        .unwrap()
        .is_some()
    );
    assert_eq!(fs::read(&fixture.mirror).unwrap(), DISK);
    assert_eq!(fs::read(&fixture.original).unwrap(), DISK);
}

#[cfg(unix)]
#[test]
fn replaced_project_root_cannot_restore_or_delete_through_a_foreign_tree() {
    use std::os::unix::fs::symlink;
    for disk_backed in [true, false] {
        let fixture = Fixture::new("src/support.ts", disk_backed.then_some(DISK));
        let mut client = fixture.client();
        let buffers = client.document_texts.clone();
        let versions = client.overlay_versions.clone();
        let mappings = client.session_document_uris.clone();
        let reverse = client.external_document_uris.clone();
        let foreign = tempfile::tempdir().unwrap();
        let retired = tempfile::tempdir().unwrap();
        let saved_root = retired.path().join("original-project");
        let foreign_root = foreign.path().canonicalize().unwrap();
        let foreign_source = foreign_root.join("src/support.ts");
        let foreign_mirror = foreign_root.join("unrelated-mirror/src/support.ts");
        fs::create_dir_all(foreign_mirror.parent().unwrap()).unwrap();
        if disk_backed {
            fs::create_dir_all(foreign_source.parent().unwrap()).unwrap();
            fs::write(&foreign_source, b"foreign source must stay unchanged\n").unwrap();
        }
        fs::write(&foreign_mirror, b"foreign mirror must stay unchanged\n").unwrap();
        fs::rename(&fixture.root, &saved_root).unwrap();
        symlink(&foreign_root, &fixture.root).unwrap();
        assert!(
            client
                .did_close(&path_to_file_uri(&fixture.original))
                .is_err()
        );
        assert_eq!(client.document_texts, buffers);
        assert_eq!(client.overlay_versions, versions);
        assert_eq!(client.session_document_uris, mappings);
        assert_eq!(client.external_document_uris, reverse);
        assert!(!client.editor_lsp_documents_dirty);
        assert_eq!(
            fs::read(&foreign_mirror).unwrap(),
            b"foreign mirror must stay unchanged\n"
        );
        if disk_backed {
            assert_eq!(
                fs::read(&foreign_source).unwrap(),
                b"foreign source must stay unchanged\n"
            );
            assert_eq!(fs::read(saved_root.join("src/support.ts")).unwrap(), DISK);
        } else {
            assert!(!foreign_source.exists());
            assert!(!saved_root.join("src/support.ts").exists());
        }
        assert_eq!(fs::read(&fixture.mirror).unwrap(), OVERLAY);
        fs::remove_file(&fixture.root).unwrap();
        fs::rename(&saved_root, &fixture.root).unwrap();
    }
}
