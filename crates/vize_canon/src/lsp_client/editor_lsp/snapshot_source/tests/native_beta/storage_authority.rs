//! Observe the source-selected mirror before close removes virtual members.

use super::hash;
use crate::{file_uri::file_uri_to_path, lsp_client::CorsaProjectClient};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub(super) fn capture(
    client: &mut CorsaProjectClient,
    authored_root: &Path,
    documents: &[(&str, &str)],
) -> Value {
    // This existing getter also exists in the pinned before source. No raw
    // RPC URI selects or broadens the expected project root.
    let root = client.editor_lsp_project_root();
    let canonical_root = root.canonicalize().unwrap();
    let metadata = fs::symlink_metadata(&root).unwrap();
    assert!(metadata.is_dir());
    assert!(!metadata.file_type().is_symlink());
    let files: Vec<_> = documents.iter().map(|(external, expected)| {
        let mapped = client.session_document_uri(external);
        let path = file_uri_to_path(&mapped).unwrap();
        let relative = path.strip_prefix(&root).unwrap();
        assert_eq!(root.join(relative), path);
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes, expected.as_bytes());
        json!({"externalUri":external,"mappedUri":mapped,"relativePath":relative,"bytes":bytes.len(),"sha256":hash(&bytes)})
    }).collect();
    let config_path = root.join("tsconfig.json");
    let config_bytes = fs::read(&config_path).unwrap();
    json!({
        "schema":1,
        "scope":"source getter and materialized file reads before close; no root inferred from observed RPC",
        "authoredRoot":authored_root,
        "clientProjectRoot":client.project_root,
        "materializedRoot":root,
        "canonicalMaterializedRoot":canonical_root,
        "rootIsDirectory":metadata.is_dir(),
        "rootIsSymlink":metadata.file_type().is_symlink(),
        "documents":files,
        "config":{"path":config_path,"bytes":config_bytes.len(),"sha256":hash(&config_bytes),"text":std::str::from_utf8(&config_bytes).unwrap()}
    })
}

pub(in super::super) fn completion_multiset(packet: &serde_json::Value) -> serde_json::Value {
    let mut result = packet.clone();
    result
        .get_mut("items")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .sort_by_cached_key(|item| serde_json::to_vec(item).unwrap());
    result
}
