use super::CorsaProjectClient;
use corsa::{
    api::{DocumentIdentifier, FileChangeSummary, FileChanges},
    runtime::block_on,
};
use std::path::PathBuf;
use vize_l0::{String, cstr};

use crate::file_uri::path_to_file_uri;

impl CorsaProjectClient {
    /// The new project already opened the complete materialized filesystem.
    /// Replaying an old directory deletion into that owner can delete its
    /// newly cached children. Only a retained independent editor needs the
    /// transition; activation already cleared the old project overlay state.
    pub(super) fn refresh_after_project_replacement(
        &mut self,
        retained_independent_editor: bool,
        changed: &[PathBuf],
        created: &[PathBuf],
        deleted: &[PathBuf],
    ) -> Result<(), String> {
        // Preserve rejection of every unrepresentable delta path even when
        // its old transition has no remaining transport owner.
        if materialized_file_changes(changed, created, deleted)?.is_none() {
            return Ok(());
        }
        if let Some(editor) = self.editor_lsp.as_mut() {
            if retained_independent_editor {
                editor.refresh_materialized_files(changed, created, deleted)?;
            } else {
                // A fresh editor still rejects exactly the URI spellings the
                // ordinary watched-file delivery would have rejected.
                for path in changed.iter().chain(created).chain(deleted) {
                    path_to_file_uri(path)
                        .parse::<lsp_types::Uri>()
                        .map_err(|error| cstr!("Invalid materialized file URI: {error}"))?;
                }
            }
        }
        Ok(())
    }

    /// Refresh an on-disk project session after materialized files change.
    ///
    /// Paths are forwarded as filesystem identifiers so Corsa can invalidate
    /// the affected dependency graph without replacing the project session.
    pub fn refresh_materialized_files(
        &mut self,
        changed: &[PathBuf],
        created: &[PathBuf],
        deleted: &[PathBuf],
    ) -> Result<(), String> {
        self.purge_deleted_materialized_overlays(deleted)?;
        let Some(file_changes) = materialized_file_changes(changed, created, deleted)? else {
            return Ok(());
        };

        self.clear_diagnostics_cache();
        // The editor transport owns a separate native project. Give it the
        // same exact disk revision, then use its response-backed readiness
        // barrier before queries. Content edits do not require a new process.
        if let Some(editor) = self.editor_lsp.as_mut() {
            editor.refresh_materialized_files(changed, created, deleted)?;
        }
        if !self.has_project_session() {
            return Ok(());
        }
        block_on(self.project_session_mut()?.refresh(Some(file_changes)))
            .map_err(|error| cstr!("Failed to refresh materialized Corsa files: {error}"))
    }

    fn purge_deleted_materialized_overlays(&mut self, deleted: &[PathBuf]) -> Result<(), String> {
        let mut deleted = deleted.to_vec();
        deleted.sort();
        deleted.dedup();
        for path in deleted {
            let uri = path_to_file_uri(&path);
            if self.document_texts.contains_key(uri.as_str()) {
                self.delete_overlay_document(uri.as_str())?;
                continue;
            }
            self.overlay_versions.remove(uri.as_str());
            self.forget_session_document_uri(uri.as_str());
            self.release_external_document_owner(uri.as_str(), uri.as_str());
            self.diagnostics.remove(uri.as_str());
        }
        Ok(())
    }
}

fn materialized_file_changes(
    changed: &[PathBuf],
    created: &[PathBuf],
    deleted: &[PathBuf],
) -> Result<Option<FileChanges>, String> {
    if changed.is_empty() && created.is_empty() && deleted.is_empty() {
        return Ok(None);
    }

    Ok(Some(FileChanges::Summary(FileChangeSummary {
        changed: document_identifiers(changed)?,
        created: document_identifiers(created)?,
        deleted: document_identifiers(deleted)?,
    })))
}

fn document_identifiers(paths: &[PathBuf]) -> Result<Vec<DocumentIdentifier>, String> {
    let mut paths = paths.to_vec();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .map(|path| {
            let path_str = path.to_str().ok_or_else(|| {
                cstr!(
                    "Corsa cannot represent non-UTF-8 materialized path {:?}",
                    path.as_os_str()
                )
            })?;
            Ok(DocumentIdentifier::from(path_str))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{CorsaProjectClient, materialized_file_changes};
    use corsa::api::{DocumentIdentifier, FileChanges};
    use std::path::PathBuf;

    #[test]
    fn groups_deduplicated_materialized_paths_by_change_kind() {
        let changed = vec![
            PathBuf::from("/workspace/b.ts"),
            PathBuf::from("/workspace/a.ts"),
            PathBuf::from("/workspace/a.ts"),
        ];

        let created = vec![PathBuf::from("/workspace/created.ts")];
        let deleted = vec![PathBuf::from("/workspace/deleted.ts")];

        let FileChanges::Summary(summary) = materialized_file_changes(&changed, &created, &deleted)
            .expect("paths should be representable")
            .expect("changes should exist")
        else {
            panic!("expected a file-change summary");
        };

        assert_eq!(
            summary.changed,
            vec![
                DocumentIdentifier::from("/workspace/a.ts"),
                DocumentIdentifier::from("/workspace/b.ts"),
            ]
        );
        assert_eq!(
            summary.created,
            vec![DocumentIdentifier::from("/workspace/created.ts")]
        );
        assert_eq!(
            summary.deleted,
            vec![DocumentIdentifier::from("/workspace/deleted.ts")]
        );
    }

    #[test]
    fn skips_refresh_when_no_materialized_paths_changed() {
        assert!(
            materialized_file_changes(&[], &[], &[])
                .expect("empty changes should be valid")
                .is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_distinct_non_utf8_paths_instead_of_merging_them() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let first = PathBuf::from(OsString::from_vec(b"/workspace/\x80.ts".to_vec()));
        let second = PathBuf::from(OsString::from_vec(b"/workspace/\x81.ts".to_vec()));
        assert_ne!(first, second);
        assert_eq!(first.to_string_lossy(), second.to_string_lossy());

        let error = materialized_file_changes(&[first.clone(), second], &[], &[])
            .expect_err("lossy paths must not collapse into one refresh entry");
        assert!(error.contains("cannot represent non-UTF-8 materialized path"));

        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        for retained_editor in [false, true] {
            for (changed, created, deleted) in [
                (vec![first.clone()], vec![], vec![]),
                (vec![], vec![first.clone()], vec![]),
                (vec![], vec![], vec![first.clone()]),
            ] {
                assert_eq!(
                    client
                        .refresh_after_project_replacement(
                            retained_editor,
                            &changed,
                            &created,
                            &deleted
                        )
                        .unwrap_err(),
                    error,
                );
            }
        }
    }

    #[test]
    fn deleted_materialized_files_are_removed_from_every_overlay_identity_map() {
        let root = tempfile::tempdir().unwrap();
        let deleted = root.path().join("mirror/Deleted.vue.ts");
        let uri = crate::file_uri::path_to_file_uri(&deleted);
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client
            .document_texts
            .insert(uri.clone(), "export const stale = true;".into());
        client.overlay_versions.insert(uri.clone(), 9);
        client
            .session_document_uris
            .insert(uri.clone(), uri.clone());
        client
            .external_document_uris
            .insert(uri.clone(), uri.clone());

        client
            .refresh_materialized_files(&[], &[], std::slice::from_ref(&deleted))
            .unwrap();

        assert!(!client.document_texts.contains_key(uri.as_str()));
        assert!(!client.overlay_versions.contains_key(uri.as_str()));
        assert!(!client.session_document_uris.contains_key(uri.as_str()));
        assert!(!client.external_document_uris.contains_key(uri.as_str()));
        assert!(client.editor_lsp_documents_dirty);
    }

    #[cfg(unix)]
    #[test]
    fn only_retained_editor_owners_receive_the_complete_old_disk_transition() {
        use crate::file_uri::path_to_file_uri;
        use serde_json::{Value, json};
        use std::{fs, os::unix::fs::PermissionsExt};

        enum Refresh {
            Retained,
            ReplacedWithRetainedEditor,
            ReplacedWithFreshEditor,
        }
        for refresh in [
            Refresh::Retained,
            Refresh::ReplacedWithRetainedEditor,
            Refresh::ReplacedWithFreshEditor,
        ] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let script = root.join("peer.ts");
            let transcript = root.join("whole-packets.jsonl");
            let peer = include_str!("editor_lsp/native_shutdown/peer.ts.txt")
                .replace("LOG_PATH", &serde_json::to_string(&transcript).unwrap())
                .replace("REPLY_MODE", "\"ok\"");
            fs::write(&script, peer).unwrap();
            fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
            let mut client =
                CorsaProjectClient::batch_editor_peer_for_test(&script, &root).unwrap();
            let uri = path_to_file_uri(&root.join("probe.ts"));
            let documents = vize_l0::FxHashMap::from_iter([(
                uri.clone(),
                vize_l0::String::from("export const value = 1;"),
            )]);
            assert_eq!(
                serde_json::to_value(client.diagnostics_via_editor_lsp(&uri, &documents).unwrap())
                    .unwrap(),
                json!({"kind":"full","items":[]}),
            );
            let changed = vec![root.join("changed.ts"), root.join("changed.ts")];
            let created = vec![root.join("node_modules/ui/UiButton.d.vue.ts")];
            let deleted = vec![root.join("node_modules/ui")];
            match refresh {
                Refresh::Retained => {
                    client.refresh_materialized_files(&changed, &created, &deleted)
                }
                Refresh::ReplacedWithRetainedEditor => {
                    client.refresh_after_project_replacement(true, &changed, &created, &deleted)
                }
                Refresh::ReplacedWithFreshEditor => {
                    client.refresh_after_project_replacement(false, &changed, &created, &deleted)
                }
            }
            .unwrap();
            assert_eq!(
                serde_json::to_value(client.diagnostics_via_editor_lsp(&uri, &documents).unwrap())
                    .unwrap(),
                json!({"kind":"full","items":[]}),
            );
            client.shutdown().unwrap();
            let rows = fs::read_to_string(&transcript)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str::<Value>(line).unwrap())
                .collect::<Vec<_>>();
            let notifications = rows
                .iter()
                .filter(|row| row["wholePacket"]["method"] == "workspace/didChangeWatchedFiles")
                .map(|row| row["wholePacket"].clone())
                .collect::<Vec<_>>();
            let expected = if matches!(refresh, Refresh::ReplacedWithFreshEditor) {
                vec![]
            } else {
                vec![
                    json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[
                        {"uri":path_to_file_uri(changed.first().unwrap()),"type":2},
                        {"uri":path_to_file_uri(changed.last().unwrap()),"type":2},
                        {"uri":path_to_file_uri(created.first().unwrap()),"type":1},
                        {"uri":path_to_file_uri(deleted.first().unwrap()),"type":3}
                    ]}}),
                ]
            };
            assert_eq!(notifications, expected);
            let methods = rows
                .iter()
                .filter_map(|row| row["wholePacket"]["method"].as_str())
                .filter(|method| {
                    matches!(
                        *method,
                        "textDocument/documentSymbol"
                            | "textDocument/diagnostic"
                            | "workspace/didChangeWatchedFiles"
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                methods,
                if matches!(refresh, Refresh::ReplacedWithFreshEditor) {
                    vec![
                        "textDocument/documentSymbol",
                        "textDocument/diagnostic",
                        "textDocument/diagnostic",
                    ]
                } else {
                    vec![
                        "textDocument/documentSymbol",
                        "textDocument/diagnostic",
                        "workspace/didChangeWatchedFiles",
                        "textDocument/documentSymbol",
                        "textDocument/diagnostic",
                    ]
                }
            );
            assert_eq!(rows.last().unwrap(), &json!({"event":"stdin-eof"}));
        }
    }
}
