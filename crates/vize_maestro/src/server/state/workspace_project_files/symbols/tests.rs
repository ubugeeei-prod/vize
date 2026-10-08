#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "tests retain original std String sources and use Arc only for buffer lifetime accounting"
)]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::{FxHashMap, ServerState, Url, WorkspaceSymbolsService, collect_sources};
use crate::runtime::block_on;

const WIDGET: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/workspace-symbol-streaming-3952/Widget.vue.txt"
));
const MODULE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/workspace-symbol-streaming-3952/module.ts.txt"
));
const EXPECTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/workspace-symbol-streaming-3952/expected.json"
));

fn original_sources() -> Vec<(Url, String)> {
    let widget = Url::parse("file:///workspace/Widget.vue").unwrap();
    let module = Url::parse("file:///workspace/module.ts").unwrap();
    vec![(widget, WIDGET.into()), (module, MODULE.into())]
}

#[test]
fn complete_streamed_corpus_matches_original_objects_and_materialized_consumer() {
    let sources = original_sources();
    for query in ["", "internal", "café", "value", "missing"] {
        let actual = collect_sources(
            sources.iter().map(|(uri, _)| uri.clone()).collect(),
            &FxHashMap::<Url, String>::default(),
            &[],
            query,
            |path| {
                sources
                    .iter()
                    .find(|(uri, _)| uri.to_file_path().unwrap() == path)
                    .map(|(_, text)| text.clone())
            },
        );
        let original = WorkspaceSymbolsService::search_sources(&sources, query);
        assert_eq!(actual, original, "{query}");
        if query.is_empty() {
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::from_str::<serde_json::Value>(EXPECTED).unwrap()
            );
        }
    }
}

struct TrackedText {
    source: String,
    live: Arc<AtomicUsize>,
}
impl AsRef<str> for TrackedText {
    fn as_ref(&self) -> &str {
        &self.source
    }
}
impl Drop for TrackedText {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::SeqCst);
    }
}

#[test]
fn streaming_releases_each_closed_source_before_reading_the_next_and_ranks_once() {
    let sources = (0..120)
        .map(|index| {
            (
                Url::parse(&vize_l0::cstr!("file:///workspace/{index:03}.ts")).unwrap(),
                "export const probe = 1;".to_string(),
            )
        })
        .collect::<Vec<_>>();
    let mut uris = sources
        .iter()
        .map(|(uri, _)| uri.clone())
        .rev()
        .collect::<Vec<_>>();
    uris.push(uris[0].clone());
    let live = Arc::new(AtomicUsize::new(0));
    let mut read_order = Vec::new();
    let symbols = collect_sources(uris, &FxHashMap::default(), &[], "probe", |path| {
        assert_eq!(
            live.fetch_add(1, Ordering::SeqCst),
            0,
            "the previous closed source is still retained"
        );
        read_order.push(path.to_owned());
        Some(TrackedText {
            source: "export const probe = 1;".into(),
            live: Arc::clone(&live),
        })
    });
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert_eq!(
        read_order,
        sources
            .iter()
            .map(|(uri, _)| uri.to_file_path().unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(symbols.len(), 100);
    assert_eq!(
        symbols,
        WorkspaceSymbolsService::search_sources(&sources, "probe")
    );
}

async fn assert_original(state: &ServerState, query: &str) {
    let sources = state.discover_workspace_project_sources().await;
    assert_eq!(
        state.search_workspace_project_symbols(query).await,
        WorkspaceSymbolsService::search_sources(&sources, query)
    );
}

#[test]
fn streamed_inventory_preserves_roots_exclusions_errors_overlays_and_retired_namespaces() {
    block_on(async {
        let root = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let missing = other.path().join("not-created-yet");
        let widget = root.path().join("Widget.vue");
        std::fs::write(&widget, WIDGET).unwrap();
        for name in [
            ".git/Hidden.ts",
            "node_modules/Hidden.ts",
            "target/Hidden.vue",
        ] {
            let path = root.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "export const hidden = 1;").unwrap();
        }
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());
        state.set_workspace_folders(vec![missing.clone()]);
        let uri = Url::from_file_path(&widget).unwrap();
        assert_original(&state, "").await;
        assert!(state.workspace_project_files.paths.read().is_none());
        assert!(
            state
                .search_workspace_project_symbols("hidden")
                .await
                .is_empty()
        );
        std::fs::create_dir(&missing).unwrap();
        std::fs::write(missing.join("module.ts"), MODULE).unwrap();
        assert_original(&state, "").await;
        assert!(state.workspace_project_files.paths.read().is_some());
        assert_eq!(
            state.search_workspace_project_symbols("café").await.len(),
            1
        );
        state.documents.open(
            uri.clone(),
            WIDGET.replace("internalState", "dirtyState"),
            1,
            "vue".into(),
        );
        state.forget_workspace_vue_files(uri.as_str());
        assert_original(&state, "").await;
        assert_eq!(
            state
                .search_workspace_project_symbols("dirtyState")
                .await
                .len(),
            1
        );
        assert!(
            state
                .search_workspace_project_symbols("internalState")
                .await
                .is_empty()
        );
        // Open unsaved files outside discovered roots remain part of the surface.
        let outside = Url::from_file_path(other.path().join("outside.ts")).unwrap();
        state
            .documents
            .open(outside.clone(), MODULE.into(), 1, "typescript".into());
        assert_original(&state, "").await;
        assert_eq!(
            state.search_workspace_project_symbols("café").await.len(),
            2
        );
        state.documents.close(&uri);
        assert_original(&state, "").await;
        assert!(
            state
                .search_workspace_project_symbols("dirtyState")
                .await
                .is_empty()
        );
        state.track_workspace_vue_files(uri.as_str());
        assert_original(&state, "").await;
        assert_eq!(
            state
                .search_workspace_project_symbols("internalState")
                .await
                .len(),
            1
        );
        state.documents.close(&outside);
        std::fs::write(missing.join("module.ts"), "export const current = 2;").unwrap();
        assert_original(&state, "").await;
        assert!(
            state
                .search_workspace_project_symbols("café")
                .await
                .is_empty()
        );
        assert_eq!(
            state
                .search_workspace_project_symbols("current")
                .await
                .len(),
            1
        );
    });
}

#[test]
fn streamed_symbols_retry_complete_results_after_project_and_document_changes() {
    block_on(async {
        #[derive(Debug, PartialEq)]
        enum Change {
            Root,
            Folders,
            Created,
            Deleted,
            Changed,
            Closed,
            Reopened,
        }
        for change in [
            Change::Root,
            Change::Folders,
            Change::Created,
            Change::Deleted,
            Change::Changed,
            Change::Closed,
            Change::Reopened,
        ] {
            let root = tempfile::tempdir().unwrap();
            let next = tempfile::tempdir().unwrap();
            let file = root.path().join("module.ts");
            let added = next.path().join("added.ts");
            std::fs::write(&file, MODULE).unwrap();
            std::fs::write(&added, "export const added = 1;").unwrap();
            let uri = Url::from_file_path(&file).unwrap();
            let state = ServerState::new();
            state.set_workspace_root(root.path().to_owned());
            if change != Change::Deleted {
                state.documents.open(
                    uri.clone(),
                    "export const before = 1;".into(),
                    1,
                    "typescript".into(),
                );
            }
            assert_original(&state, "").await;
            let (reached, waiting) = futures::channel::oneshot::channel();
            let (resume, paused) = futures::channel::oneshot::channel();
            *state.workspace_project_files.read_pause.write() = Some((reached, paused));
            let mutate = async {
                waiting.await.unwrap();
                match change {
                    Change::Root => state.set_workspace_root(next.path().to_owned()),
                    Change::Folders => state.set_workspace_folders(vec![next.path().to_owned()]),
                    Change::Created => {
                        let path = root.path().join("created.ts");
                        std::fs::write(&path, "export const created = 1;").unwrap();
                        state.observe_workspace_project_membership(&path, true);
                    }
                    Change::Deleted => state.observe_workspace_project_membership(&file, false),
                    Change::Changed => {
                        state.documents.apply_changes(
                            &uri,
                            vec![tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                                range: None,
                                range_length: None,
                                text: "export const after = 1;".into(),
                            }],
                            2,
                        );
                    }
                    Change::Closed => state.documents.close(&uri),
                    Change::Reopened => {
                        state.documents.close(&uri);
                        state.documents.open(
                            uri.clone(),
                            "export const reopened = 1;".into(),
                            1,
                            "typescript".into(),
                        );
                    }
                }
                resume.send(()).unwrap();
            };
            let (actual, ()) = futures::join!(state.search_workspace_project_symbols(""), mutate);
            let sources = state.discover_workspace_project_sources().await;
            assert_eq!(
                actual,
                WorkspaceSymbolsService::search_sources(&sources, ""),
                "{change:?}"
            );
        }
    });
}

#[test]
fn failed_symbol_worker_preserves_complete_open_overlay_answer() {
    block_on(async {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("disk.ts"), MODULE).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());
        let open = original_sources();
        for (uri, text) in &open {
            state
                .documents
                .open(uri.clone(), text.clone(), 1, "vue".into());
        }
        state
            .workspace_project_files
            .symbol_worker_failure
            .store(true, Ordering::SeqCst);
        assert_eq!(
            state.search_workspace_project_symbols("").await,
            WorkspaceSymbolsService::search_sources(&open, "")
        );
        assert!(state.workspace_project_files.paths.read().is_some());
    });
}
