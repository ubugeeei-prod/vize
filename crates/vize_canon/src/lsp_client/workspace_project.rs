//! Switching a long-lived client onto a materialized Canon project.

use std::path::Path;

use vize_l0::String;

use super::{
    CorsaProjectClient,
    lifecycle_setup::workspace_config_path,
    session::{ProjectSessionSpawnError, spawn_project_session},
};

impl CorsaProjectClient {
    pub(crate) fn synchronize_materialized_project(
        &mut self,
        project_root: &Path,
        config_path: Option<&Path>,
        changes: &crate::batch::virtual_project::MaterializedFileDelta,
    ) -> Result<(), String> {
        if changes.has_topology_changes() {
            self.activate_workspace_project_with_reload(project_root, config_path, true)?;
            self.refresh_materialized_files(&changes.changed, &changes.created, &changes.deleted)?;
        } else {
            self.activate_workspace_project_with_reload(project_root, config_path, false)?;
            if !changes.is_empty() {
                self.refresh_materialized_files(
                    &changes.changed,
                    &changes.created,
                    &changes.deleted,
                )?;
            }
        }
        Ok(())
    }

    /// Move both native query transports to an already-materialized Canon
    /// project. The mirror's tsconfig is the authority for native condition
    /// selection; merely opening a file under its `node_modules` tree would
    /// otherwise create an inferred project with default compiler options.
    /// Replace only the native project handle when package topology changes.
    /// Standard tsgo retains negative module-resolution state across a file
    /// summary refresh, while a new handle observes the already-materialized
    /// Canon snapshot without restarting the bridge process.
    fn activate_workspace_project_with_reload(
        &mut self,
        project_root: &Path,
        config_path: Option<&Path>,
        reload: bool,
    ) -> Result<(), String> {
        let project_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        let root_changed = self.project_root != project_root;
        let config_changed = explicit_config_changed(
            self.explicit_project_config.as_deref(),
            config_path,
            &project_root,
        );
        if !reload && !root_changed && !config_changed {
            return Ok(());
        }

        self.retire_original_diagnosing_session()?;

        let config_path = config_path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_config_path(&project_root));
        // The editor owner may still be live. Reap the previous API process
        // before the replacement can exist, including its initialization IPC.
        self.retire_project_session()?;
        let (session, capabilities) =
            match spawn_project_session(self.executable.as_str(), &project_root, &config_path) {
                Ok((session, capabilities)) => (Some(session), capabilities),
                Err(ProjectSessionSpawnError::Unavailable(reason)) => {
                    tracing::debug!(
                        reason = reason.as_str(),
                        "using standard tsgo editor-only Canon mirror session"
                    );
                    (None, std::sync::Arc::new(Default::default()))
                }
                Err(ProjectSessionSpawnError::Failed(error)) => return Err(error),
            };
        self.session = session;
        self.capabilities = capabilities;
        self.cwd = project_root.clone();
        self.project_root = project_root;
        if self.explicit_project_config.is_some() {
            self.explicit_project_config = Some(config_path);
        }
        self.materialized_project_session = false;
        self.clear_workspace_project_overlays();
        self.session_document_uris.clear();
        self.external_document_uris.clear();
        self.diagnostics.clear();
        if root_changed || config_changed {
            self.retire_editor_lsp()?;
        }
        Ok(())
    }

    fn clear_workspace_project_overlays(&mut self) {
        self.document_texts.clear();
        self.overlay_versions.clear();
        self.editor_lsp_documents_dirty = true;
    }
}

fn explicit_config_changed(current: Option<&Path>, requested: Option<&Path>, root: &Path) -> bool {
    current.is_some_and(|current| {
        requested
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_config_path(root))
            != current
    })
}

#[cfg(test)]
mod tests {
    use super::{CorsaProjectClient, explicit_config_changed};
    use std::path::Path;

    #[test]
    fn explicit_selection_changes_are_observed_before_same_root_reuse() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("playground/tsconfig.json");
        assert!(!explicit_config_changed(None, Some(&nested), root.path()));
        assert!(!explicit_config_changed(
            Some(&nested),
            Some(&nested),
            root.path()
        ));
        assert!(explicit_config_changed(Some(&nested), None, root.path()));
        assert!(explicit_config_changed(
            Some(&nested),
            Some(Path::new("/other/config.json")),
            root.path(),
        ));
    }

    #[test]
    fn project_reload_drops_overlays_before_the_editor_fallback_can_reopen_them() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.document_texts.insert(
            "file:///mirror/deleted.ts".into(),
            "export const stale = true;".into(),
        );
        client
            .overlay_versions
            .insert("file:///mirror/deleted.ts".into(), 3);

        client.clear_workspace_project_overlays();

        assert!(client.document_texts.is_empty());
        assert!(client.overlay_versions.is_empty());
        assert!(client.editor_lsp_documents_dirty);
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "../../tests/support/original_diagnosing_process.rs"]
mod native_process_control;

#[cfg(all(test, target_os = "linux"))]
mod process_tests {
    use super::CorsaProjectClient;
    use super::native_process_control as control;
    use std::{fs, io::Write, os::unix::fs::PermissionsExt, path::Path};
    use vize_l0::{FxHashMap, cstr};

    const FIXTURE: &str = include_str!(
        "../../../../tests/_fixtures/differential/lsp/native-project-retirement-3952/input.json"
    );
    const OBSERVER: &str = include_str!(
        "../../../../tests/_fixtures/differential/lsp/native-project-retirement-3952/observe-native-owner.py"
    );

    fn quote(path: &Path) -> vize_l0::String {
        cstr!("'{}'", path.to_str().unwrap().replace('\'', "'\"'\"'"))
    }

    #[test]
    fn native_project_replacement_reaps_previous_api_before_launch_and_keeps_whole_answer() {
        if !control::enabled() {
            return;
        }
        let fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
        let root = tempfile::tempdir().unwrap();
        control::write_fixture(root.path(), &fixture);
        assert_eq!(fixture["source"], control::fixture()["source"]);
        let native = control::runtime();
        let trace = tempfile::tempdir().unwrap();
        let observer = trace.path().join("observe.py");
        fs::write(&observer, OBSERVER).unwrap();
        let log = trace.path().join("native-owners.jsonl");
        // The existing .bin classification selects the genuine async API recipe.
        let bin = trace.path().join(".bin");
        fs::create_dir(&bin).unwrap();
        let launcher = bin.join("tsgo");
        fs::write(
            &launcher,
            cstr!(
                "#!/bin/sh\nexec python3 {} {} {} \"$@\"\n",
                quote(&observer),
                quote(&native),
                quote(&log),
            )
            .as_str(),
        )
        .unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
        let mut client =
            CorsaProjectClient::new_for_workspace(launcher.to_str(), root.path()).unwrap();
        assert!(
            client.has_project_session(),
            "actual native API is required"
        );
        let mut api_owners = vec![control::native_process(root.path(), &native, b"--api")];
        let uri = crate::file_uri::path_to_file_uri(&root.path().join("source.ts"));
        let source = fixture["source"].as_str().unwrap();
        let mut documents = FxHashMap::default();
        documents.insert(uri.clone(), source.into());
        let mut editor_owner = None;
        let mut diagnostic_reports = Vec::new();
        for _ in 0..3 {
            let report = client
                .diagnostics_via_editor_lsp(uri.as_str(), &documents)
                .unwrap();
            let report = serde_json::to_value(report).unwrap();
            assert_eq!(report, fixture["expected"]);
            diagnostic_reports.push(report);
            let observed = control::native_lsp(root.path(), &native);
            if let Some(previous) = &editor_owner {
                assert_eq!(
                    &observed, previous,
                    "topology reload reuses the native editor"
                );
            }
            editor_owner = Some(observed);
            client
                .activate_workspace_project_with_reload(root.path(), None, true)
                .unwrap();
            control::assert_reaped(api_owners.last().unwrap());
            api_owners.push(control::native_process(root.path(), &native, b"--api"));
            assert!(
                client.editor_lsp.is_some(),
                "topology reload retains the editor owner"
            );
        }
        let final_report = client
            .diagnostics_via_editor_lsp(uri.as_str(), &documents)
            .unwrap();
        let final_report = serde_json::to_value(final_report).unwrap();
        assert_eq!(final_report, fixture["expected"]);
        diagnostic_reports.push(final_report);
        assert_eq!(
            control::native_lsp(root.path(), &native),
            *editor_owner.as_ref().unwrap(),
            "the third topology reload retains the complete editor answer and owner",
        );
        client.document_texts = documents;
        client.activate_materialized_project_session().unwrap();
        assert!(client.has_project_session());
        assert!(
            client.editor_lsp.is_none(),
            "mode transition retires the old editor"
        );
        control::assert_reaped(editor_owner.as_ref().unwrap());
        control::assert_reaped(api_owners.last().unwrap());
        api_owners.push(control::native_process(root.path(), &native, b"--api"));
        client.shutdown().unwrap();
        control::assert_reaped(api_owners.last().unwrap());

        let launches: Vec<serde_json::Value> = fs::read_to_string(&log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let receipt = serde_json::json!({
            "schema": "vize.native-project-retirement-3952",
            "native": native,
            "editor": editor_owner.map(|owner| serde_json::json!({
                "pid": owner.pid, "birth": owner.birth, "executable": owner.executable,
            })),
            "physicalApiOwners": api_owners.iter().map(|owner| serde_json::json!({
                "pid": owner.pid, "birth": owner.birth, "executable": owner.executable,
            })).collect::<Vec<_>>(),
            "launches": launches,
            "expectedDiagnosticReport": fixture["expected"],
            "diagnosticReports": diagnostic_reports,
        });
        std::io::stdout()
            .write_all(cstr!("NATIVE_PROJECT_RETIREMENT_3952 {}\n", receipt).as_bytes())
            .unwrap();
        let api: Vec<_> = launches
            .iter()
            .filter(|entry| entry["role"] == "--api")
            .collect();
        assert_eq!(
            api.len(),
            5,
            "initial API, three reloads and mode transition"
        );
        for (launch, physical) in api.iter().zip(&api_owners) {
            assert_eq!(launch["pid"].as_u64(), Some(u64::from(physical.pid)));
            assert_eq!(
                launch["birth"].as_str().unwrap().parse::<u64>().unwrap(),
                physical.birth
            );
            assert_eq!(physical.executable, native);
            let previous = launch["previousNativeOwners"].as_array().unwrap();
            assert!(
                previous.iter().all(|owner| owner["role"] != "--api"),
                "previous physical API process was still present at replacement: {launch}"
            );
        }
        assert!(
            api.iter().skip(1).all(|launch| {
                launch["previousNativeOwners"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|owner| owner["role"] == "--lsp")
            }),
            "all replacements must exercise the live editor plus API ownership boundary"
        );
    }
}
