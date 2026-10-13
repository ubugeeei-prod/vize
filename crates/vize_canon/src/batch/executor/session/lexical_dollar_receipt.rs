//! Opt-in test custody from the client that diagnosed the lexical-dollar corpus.

use super::{CorsaProjectClient, VirtualProject};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use vize_l0::cstr;

pub(super) fn begin(client: &CorsaProjectClient) {
    if std::env::var_os("VIZE_LEXICAL_DOLLAR_CAPTURE_DIR").is_some() {
        client.begin_batch_test_receipt(true);
    }
}

pub(super) fn capture(
    client: &mut CorsaProjectClient,
    project: &VirtualProject,
    uris: &[vize_l0::String],
    raw: &impl std::fmt::Debug,
) {
    let Some(dir) = std::env::var_os("VIZE_LEXICAL_DOLLAR_CAPTURE_DIR").map(PathBuf::from) else {
        return;
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let uri = uris
        .iter()
        .find(|uri| uri.ends_with("App.vue.ts"))
        .expect("actual scanned authored App");
    let native = client.batch_test_receipt(uri);
    let config = project.generated_tsconfig_path();
    let authored = project.project_root().join("tsconfig.json");
    let receipt = json!({
        "native": native, "requestedUris": uris,
        "generatedConfigPath": config,
        "generatedConfigBytes": fs::read_to_string(&config).unwrap(),
        "authoredConfigPath": authored,
        "authoredConfigBytes": fs::read_to_string(&authored).unwrap(),
        "rawDiagnosticResponse": cstr!("{raw:#?}"),
        "generatedFiles": project.virtual_files_sorted().into_iter().map(|file| json!({
            "original": file.original_path, "virtual": file.virtual_path,
            "authoredSource": fs::read_to_string(&file.original_path).unwrap(),
            "code": file.content.as_str(), "sourceMap": cstr!("{:#?}", file.source_map),
        })).collect::<Vec<_>>(),
    });
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(cstr!("{sequence:03}-same-owner.json").as_str()),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    assert_eq!(
        receipt["native"]["selectedConfig"],
        json!(config),
        "{receipt}"
    );
    assert!(receipt["native"]["project"].is_object(), "{receipt}");
    assert!(
        receipt["native"]["snapshotProjects"].is_array(),
        "{receipt}"
    );
    assert!(
        receipt["native"]["normalizedRequestedOptions"].is_object(),
        "{receipt}"
    );
    assert_eq!(
        receipt["native"]["project"]["compilerOptions"],
        receipt["native"]["normalizedRequestedOptions"],
        "{receipt}"
    );
    assert_eq!(
        receipt["native"]["project"]["compilerOptions"]["strict"],
        json!(true),
        "{receipt}"
    );
}
