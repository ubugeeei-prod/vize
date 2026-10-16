mod support;

use std::path::Path;

use corsa::{
    api::{ApiClient, ApiMode, UpdateSnapshotParams},
    runtime::block_on,
};
use serde_json::{Value, json};

#[test]
fn real_corsa_api_summary_matches_pinned_baseline() {
    block_on(async {
        let baseline = load_baseline();
        let mut observed_case_sensitivity = None;
        let mut observed_source_bytes = None;
        for mode in [ApiMode::SyncMsgpackStdio, ApiMode::AsyncJsonRpcStdio] {
            let Some(config) = support::real_api_config(mode) else {
                return;
            };
            let client = ApiClient::spawn(config).await.unwrap();
            let dataset = support::real_dataset();
            let dataset_wire = dataset.display().to_string();
            let workspace_root = support::workspace_root();
            let init = client.initialize().await.unwrap();
            let config = client
                .parse_config_file(dataset_wire.as_str())
                .await
                .unwrap();
            let snapshot = client
                .update_snapshot(UpdateSnapshotParams {
                    open_project: Some(dataset_wire),
                    file_changes: None,
                    overlay_changes: None,
                })
                .await
                .unwrap();
            let project = &snapshot.projects[0];
            let primary_file = config
                .file_names
                .iter()
                .find(|file| !file.ends_with(".d.ts"))
                .cloned()
                .unwrap_or_else(|| config.file_names[0].clone());
            let source = client
                .get_source_file(
                    snapshot.handle.clone(),
                    project.id.clone(),
                    primary_file.as_str(),
                )
                .await
                .unwrap()
                .unwrap();
            let string_type = client
                .get_string_type(snapshot.handle.clone(), project.id.clone())
                .await
                .unwrap();
            let rendered = client
                .type_to_string(
                    snapshot.handle.clone(),
                    project.id.clone(),
                    string_type.id,
                    None,
                    None,
                )
                .await
                .unwrap();
            let source_bytes = source.as_bytes().len();
            if let Some(previous) =
                observed_case_sensitivity.replace(init.use_case_sensitive_file_names)
            {
                assert_eq!(previous, init.use_case_sensitive_file_names);
            }
            if let Some(previous) = observed_source_bytes.replace(source_bytes) {
                assert_eq!(previous, source_bytes);
            }
            // These values vary with the runner environment, so we assert local invariants
            // above and keep the shared JSON baseline focused on the stable project summary.
            assert_eq!(
                summary(SummaryInput {
                    workspace_root: &workspace_root,
                    current_directory: &init.current_directory,
                    files: &config.file_names,
                    project_count: snapshot.projects.len(),
                    config_file_name: &project.config_file_name,
                    root_files: project.root_files.len(),
                    primary_file: primary_file.as_str(),
                    string_type_flags: string_type.flags,
                    rendered: rendered.as_str(),
                }),
                baseline
            );
            snapshot.release().await.unwrap();
            client.close().await.unwrap();
        }
    });
}

fn load_baseline() -> Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/real_corsa_api_baseline.json");
    serde_json::from_str(std::fs::read_to_string(path).unwrap().as_str()).unwrap()
}

struct SummaryInput<'a> {
    workspace_root: &'a Path,
    current_directory: &'a str,
    files: &'a [String],
    project_count: usize,
    config_file_name: &'a str,
    root_files: usize,
    primary_file: &'a str,
    string_type_flags: u32,
    rendered: &'a str,
}

fn summary(input: SummaryInput<'_>) -> Value {
    json!({
        "currentDirectory": normalize_path(input.workspace_root, input.current_directory),
        "fileCount": input.files.len(),
        "firstFile": normalize_path(input.workspace_root, &input.files[0]),
        "lastFile": normalize_path(input.workspace_root, input.files.last().unwrap()),
        "projectCount": input.project_count,
        "rootFiles": input.root_files,
        "configFileName": normalize_path(input.workspace_root, input.config_file_name),
        "primaryFile": normalize_path(input.workspace_root, input.primary_file),
        "stringTypeFlags": input.string_type_flags,
        "rendered": input.rendered,
    })
}

fn normalize_path(workspace_root: &Path, value: &str) -> String {
    if value == workspace_root.display().to_string() {
        return ".".to_owned();
    }
    workspace_root
        .join("")
        .as_path()
        .to_str()
        .and_then(|root| value.strip_prefix(root))
        .map(|relative| relative.trim_start_matches('/').to_owned())
        .unwrap_or_else(|| value.to_owned())
}
