//! Historical batch expectations replayed through the real native editor projection.
use super::{AmbientComponentProject, AmbientProjectError};
use crate::{
    server::ServerState,
    source_project::{SnapshotRefusal, SourceSnapshot, SourceSnapshotCache},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};
use vize_canon::CorsaBridgeConfig;

mod cost;
mod repeated;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pack() -> PathBuf {
    workspace().join("tests/_fixtures/differential/typechecker/ambient-global-slot-host-3011")
}

struct Project {
    dir: tempfile::TempDir,
    state: Arc<ServerState>,
    snapshots: SourceSnapshotCache,
}

impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let modules = root.join("node_modules");
        fs::create_dir(&modules).unwrap();
        let vue = [
            workspace().join("playground/node_modules/vue"),
            workspace().join("node_modules/vue"),
        ]
        .into_iter()
        .find_map(|p| p.canonicalize().ok())
        .expect("locked Vue installation");
        for (name, path) in [
            ("vue", vue.as_path()),
            ("@vue", &vue.parent().unwrap().join("@vue")),
        ] {
            #[cfg(unix)]
            std::os::unix::fs::symlink(path, modules.join(name)).unwrap();
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(path, modules.join(name)).unwrap();
        }
        let state = Arc::new(ServerState::new());
        state.set_workspace_root(root.to_path_buf());
        Self {
            dir,
            state,
            snapshots: SourceSnapshotCache::default(),
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
    fn uri(&self, name: &str) -> Url {
        Url::from_file_path(self.path(name)).unwrap()
    }
    fn write(&self, name: &str, text: &str) {
        let path = self.path(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn open(&self, name: &str, source: &str, lang: &str) -> Arc<SourceSnapshot> {
        let uri = self.uri(name);
        self.state
            .documents
            .open(uri.clone(), source.into(), 1, lang.into());
        self.snapshots.capture(&self.state.documents, &uri).unwrap()
    }
    fn change(&self, name: &str, source: &str, version: i32) -> Arc<SourceSnapshot> {
        let uri = self.uri(name);
        assert!(self.state.documents.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: source.into()
            }],
            version
        ));
        self.snapshots.capture(&self.state.documents, &uri).unwrap()
    }
    fn refresh(&self, name: &str) {
        assert!(
            self.state
                .invalidate_global_component_references([self.uri(name).as_str()])
        );
    }
    fn config(&self) -> CorsaBridgeConfig {
        let runtime = std::env::var_os("VIZE_TEST_TSGO_PATH")
            .map(PathBuf::from)
            .or_else(|| vize_carton::corsa_resolver::discover_corsa_in_ancestors(&workspace()))
            .expect("locked native TypeScript executable");
        CorsaBridgeConfig {
            corsa_path: Some(runtime),
            tsconfig_path: Some(self.path("tsconfig.json")),
            ..Default::default()
        }
    }
    fn start(&self) -> AmbientComponentProject {
        AmbientComponentProject::new(Arc::clone(&self.state), self.config()).unwrap()
    }
}

async fn wire(session: &AmbientComponentProject, snapshot: &SourceSnapshot) -> Value {
    serde_json::to_value(session.diagnostics(snapshot).await.unwrap()).unwrap()
}

#[test]
#[ignore = "requires the repository's pinned TypeScript and Vue packages"]
fn historical_slot_pack_maps_complete_native_responses() {
    let cases: Value =
        serde_json::from_slice(&fs::read(pack().join("cases.json")).unwrap()).unwrap();
    crate::runtime::block_on(async {
        for case in cases["cases"].as_array().unwrap() {
            let project = Project::new();
            for input in case["inputs"].as_array().unwrap() {
                project.write(
                    input["file"].as_str().unwrap(),
                    &fs::read_to_string(pack().join(input["source"].as_str().unwrap())).unwrap(),
                );
            }
            let name = "src/App.vue";
            let snapshot = project.open(
                name,
                &fs::read_to_string(project.path(name)).unwrap(),
                "vue",
            );
            let session = project.start();
            let expected: Vec<_> = case["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| expected_slot_response(entry, name, &snapshot))
                .collect();
            assert_eq!(
                wire(&session, &snapshot).await,
                Value::Array(expected.clone()),
                "{}",
                case["id"]
            );
            let hits = &session.bridge.cache_stats().hits;
            let before = hits.load(std::sync::atomic::Ordering::Relaxed);
            assert_eq!(
                wire(&session, &snapshot).await,
                Value::Array(expected),
                "retained {}",
                case["id"]
            );
            assert_eq!(hits.load(std::sync::atomic::Ordering::Relaxed), before + 1);
            session.shutdown().await.unwrap();
        }
    });
}

fn expected_slot_response(entry: &Value, name: &str, snapshot: &SourceSnapshot) -> Value {
    let line = u32::try_from(entry["line"].as_u64().unwrap() - 1).unwrap();
    let character = u32::try_from(entry["column"].as_u64().unwrap() - 1).unwrap();
    let end = character + 7;
    let positions = vize_l0::line_index::LineBreaks::Lsp;
    let start_byte = positions
        .position_to_offset(snapshot.source(), line, character)
        .unwrap();
    let end_byte = positions
        .position_to_offset(snapshot.source(), line, end)
        .unwrap();
    assert_eq!(entry["file"], name);
    assert_eq!(snapshot.source().get(start_byte..end_byte), Some("missing"));
    json!({
        "range": {
            "start": { "line": line, "character": character },
            "end": { "line": line, "character": end }
        },
        "code": entry["code"],
        "source": "vize/types",
        "severity": entry["severity"],
        "message": entry["message"]
    })
}

const PROP_HOST: &str = "<template><GlobalBadge :level=\"'x'\" /></template>\n";
const NUMBER: &str = "<script setup lang=\"ts\">defineProps<{ level: number }>()</script>\n<template><span /></template>\n";
const PROVIDER: &str = "import GlobalBadge from './components/GlobalBadge.vue'\ndeclare module 'vue' { export interface GlobalComponents { GlobalBadge: typeof GlobalBadge } }\nexport {}\n";

#[test]
#[ignore = "requires the repository's pinned TypeScript and Vue packages"]
fn declarations_created_edited_closed_and_deleted_update_the_same_native_session() {
    let p = Project::new();
    p.write(
        "tsconfig.json",
        &fs::read_to_string(pack().join("tsconfig.json.txt")).unwrap(),
    );
    const HOST: &str = "src/views/App.vue";
    const CHILD: &str = "src/components/GlobalBadge.vue";
    const DECL: &str = "src/components.d.ts";
    p.write(CHILD, NUMBER);
    p.write(HOST, PROP_HOST);
    let host = p.open(HOST, PROP_HOST, "vue");
    crate::runtime::block_on(async {
        let session = p.start();
        assert_eq!(wire(&session, &host).await, json!([]));
        p.write(DECL, PROVIDER);
        p.refresh(DECL);
        let invalid = json!([{"code":2322,"message":"Type 'string' is not assignable to type 'number'.",
            "range":{"start":{"line":0,"character":24},"end":{"line":0,"character":29}},"severity":1,"source":"vize/types"}]);
        assert_eq!(wire(&session, &host).await, invalid);
        p.open(DECL, "export {}\n", "typescript");
        assert_eq!(wire(&session, &host).await, json!([]));
        p.change(DECL, PROVIDER, 2);
        assert_eq!(wire(&session, &host).await, invalid);
        p.state.documents.close(&p.uri(DECL));
        assert_eq!(wire(&session, &host).await, invalid);
        p.open(CHILD, &NUMBER.replace("number", "string"), "vue");
        assert_eq!(wire(&session, &host).await, json!([]));
        p.state.documents.close(&p.uri(CHILD));
        assert_eq!(wire(&session, &host).await, invalid);
        // A closed dependency can change without a document revision or watcher event.
        let child_path = p.path(CHILD);
        let previous_time = fs::metadata(&child_path).unwrap().modified().unwrap();
        let edit = NUMBER.replace("number", "string");
        assert_eq!(edit.len(), NUMBER.len());
        p.write(CHILD, &edit);
        fs::File::options()
            .write(true)
            .open(&child_path)
            .unwrap()
            .set_modified(previous_time)
            .unwrap();
        assert_eq!(wire(&session, &host).await, json!([]));
        p.write(CHILD, NUMBER);
        fs::File::options()
            .write(true)
            .open(&child_path)
            .unwrap()
            .set_modified(previous_time)
            .unwrap();
        assert_eq!(wire(&session, &host).await, invalid);
        let other_name = "src/views/Other.vue";
        let other_text = "<template><GlobalBadge :level=\"1\" /></template>";
        p.write(other_name, other_text);
        let other = p.open(other_name, other_text, "vue");
        assert_eq!(wire(&session, &other).await, json!([]));
        assert_eq!(wire(&session, &host).await, invalid);
        p.state.documents.close(&p.uri(other_name));
        fs::remove_file(p.path(DECL)).unwrap();
        p.refresh(DECL);
        assert_eq!(wire(&session, &host).await, json!([]));
        p.open(DECL, PROVIDER, "typescript");
        assert_eq!(wire(&session, &host).await, invalid);
        p.state.documents.close(&p.uri(DECL));
        assert_eq!(wire(&session, &host).await, json!([]));
        p.change(HOST, "<template/>", 2);
        assert!(matches!(
            session.diagnostics(&host).await,
            Err(AmbientProjectError::Snapshot(SnapshotRefusal::Superseded))
        ));
        session.shutdown().await.unwrap();
    });
}

mod profile;
