//! Observe the source-selected mirror before close removes virtual members.

use super::hash;
use crate::{
    file_uri::{file_uri_to_path, path_to_file_uri},
    lsp_client::CorsaProjectClient,
};
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
    if canonical_root == authored_root.canonicalize().unwrap() {
        return logical_capture(client, authored_root, documents);
    }
    let metadata = fs::symlink_metadata(&root).unwrap();
    assert!(metadata.is_dir());
    assert!(!metadata.file_type().is_symlink());
    assert_eq!(documents.len(), 3);
    let authored_path = file_uri_to_path(&path_to_file_uri(authored_root)).unwrap();
    let mirror_path = file_uri_to_path(&path_to_file_uri(&root)).unwrap();
    let files: Vec<_> = documents.iter().zip(["main.ts", "support.ts", "Virtual.vue.ts"]).map(|((external, expected), relative)| {
        assert_eq!(file_uri_to_path(external).unwrap(), authored_path.join(relative));
        let mapped = client.session_document_uri(external);
        let path = file_uri_to_path(&mapped).unwrap();
        assert_eq!(path, mirror_path.join(relative));
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

fn logical_capture(
    client: &mut CorsaProjectClient,
    authored_root: &Path,
    documents: &[(&str, &str)],
) -> Value {
    assert_eq!(documents.len(), 3);
    let root = client.editor_lsp_project_root();
    assert_eq!(
        root.canonicalize().unwrap(),
        authored_root.canonicalize().unwrap()
    );
    let authored = file_uri_to_path(&path_to_file_uri(authored_root)).unwrap();
    let files: Vec<_> = documents.iter().zip(["main.ts", "support.ts", "Virtual.vue.ts"])
        .map(|((external, expected), relative)| {
            let path = authored.join(relative);
            assert_eq!(file_uri_to_path(external).unwrap(), path);
            let mapped = client.session_document_uri(external);
            assert_eq!(mapped.as_str(), *external);
            let text = client.document_texts.get(*external).unwrap();
            assert_eq!(text.as_str(), *expected);
            let disk = if relative == "Virtual.vue.ts" {
                assert!(!path.exists());
                Value::Null
            } else {
                let bytes = fs::read(&path).unwrap();
                assert_eq!(bytes, expected.as_bytes());
                json!({"path":path,"bytes":bytes.len(),"sha256":hash(&bytes)})
            };
            json!({"externalUri":external,"mappedUri":mapped,"relativePath":relative,
                "storage":"retained-buffer","text":text,"bytes":text.len(),"sha256":hash(text.as_bytes()),"disk":disk})
        }).collect();
    let mut retained = client.document_texts.iter().collect::<Vec<_>>();
    retained.sort_unstable_by(|left, right| left.0.cmp(right.0));
    let retained: Vec<_> = retained.into_iter().map(|(uri, text)|
        json!({"uri":uri,"text":text,"bytes":text.len(),"sha256":hash(text.as_bytes())})).collect();
    let config_path = authored_root.join("tsconfig.json");
    let config_bytes = fs::read(&config_path).unwrap();
    json!({"schema":2,"mode":"logical-editor-revision",
        "scope":"source-selected original logical root and complete retained buffers; no disk materialization",
        "authoredRoot":authored_root,"clientProjectRoot":client.project_root,
        "logicalRoot":root,"canonicalLogicalRoot":root.canonicalize().unwrap(),
        "documents":files,"retainedDocuments":retained,
        "config":{"path":config_path,"bytes":config_bytes.len(),"sha256":hash(&config_bytes),"text":std::str::from_utf8(&config_bytes).unwrap()}})
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
