//! Optional, test-only evidence; production and Tier-L timing builds omit it.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{CorsaProjectClient, VirtualProject};
use vize_l0::cstr;

pub(super) fn begin(client: &CorsaProjectClient) {
    client.begin_batch_test_receipt(std::env::var_os("VIZE_NESTED_BATCH_CAPTURE_DIR").is_some());
}

pub(super) fn capture(
    client: &mut CorsaProjectClient,
    project: &VirtualProject,
    uris: &[vize_l0::String],
    raw: &impl std::fmt::Debug,
) {
    let Some(dir) = std::env::var_os("VIZE_NESTED_BATCH_CAPTURE_DIR").map(PathBuf::from) else {
        return;
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let uri = uris.iter().find(|uri| uri.ends_with("App.vue.ts")).unwrap();
    let native = client.batch_test_receipt(uri);
    let config = project.generated_tsconfig_path();
    let receipt = serde_json::json!({
        "native": native, "requestedUris": uris,
        "generatedConfigPath": config,
        "generatedConfigBytes": fs::read_to_string(&config).unwrap(),
        "rootConfigExists": project.virtual_root().join("tsconfig.json").exists(),
        "authoredConfigBytes": fs::read_to_string(project.project_root().join("playground/tsconfig.json")).unwrap(),
        "rawDiagnosticResponse": cstr!("{raw:#?}"),
        "generatedFiles": project.virtual_files_sorted().into_iter().map(|file| serde_json::json!({
            "original": file.original_path, "virtual": file.virtual_path, "code": file.content.as_str(),
        })).collect::<Vec<_>>(),
    });
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(cstr!("{sequence:03}-native.json").as_str()),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    assert_eq!(
        receipt["native"]["selectedConfig"],
        serde_json::json!(config),
        "the diagnosing native process must select the exact generated config: {receipt}"
    );
}
