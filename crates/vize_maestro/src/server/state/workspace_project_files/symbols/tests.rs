#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "tests retain original std String sources and use Arc only for buffer lifetime accounting"
)]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::{ServerState, Url, WorkspaceSymbolsService, collect_sources};
use crate::runtime::block_on;
use tower_lsp::lsp_types::{FileChangeType, FileEvent};

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
            &[],
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
    let symbols = collect_sources(uris, &[], &[], "probe", |path| {
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

#[test]
fn watched_disk_index_reuses_one_generation_and_rebuilds_after_change() {
    block_on(async {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("disk.ts");
        std::fs::write(&path, "export const oldSymbol = 1;").unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());
        state.set_corsa_disk_watcher_registered();

        let old = state.search_workspace_project_symbols("oldSymbol").await;
        assert_eq!(old.len(), 1);
        let seeded = Arc::clone(state.workspace_symbol_cache.get(&uri).unwrap().value());

        // The watcher contract makes this generation authoritative until its
        // notification arrives, so another query must not touch the disk.
        std::fs::write(&path, "export const newSymbol = 2;").unwrap();
        let warm = state.search_workspace_project_symbols("oldSymbol").await;
        assert_eq!(warm.len(), 1);
        assert!(Arc::ptr_eq(
            &seeded,
            state.workspace_symbol_cache.get(&uri).unwrap().value()
        ));

        state.observe_workspace_project_file_events(&[FileEvent {
            uri: uri.clone(),
            typ: FileChangeType::CHANGED,
        }]);
        assert!(
            state
                .search_workspace_project_symbols("oldSymbol")
                .await
                .is_empty()
        );
        let changed = state.search_workspace_project_symbols("newSymbol").await;
        assert_eq!(changed.len(), 1);
        assert!(!Arc::ptr_eq(
            &seeded,
            state.workspace_symbol_cache.get(&uri).unwrap().value()
        ));
    });
}

#[test]
fn unwatched_disk_search_keeps_fresh_read_semantics() {
    block_on(async {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("disk.ts");
        std::fs::write(&path, "export const before = 1;").unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());

        assert_eq!(
            state.search_workspace_project_symbols("before").await.len(),
            1
        );
        assert!(state.workspace_symbol_cache.is_empty());

        std::fs::write(&path, "export const after = 2;").unwrap();
        assert!(
            state
                .search_workspace_project_symbols("before")
                .await
                .is_empty()
        );
        assert_eq!(
            state.search_workspace_project_symbols("after").await.len(),
            1
        );
        assert!(state.workspace_symbol_cache.is_empty());
    });
}

#[cfg(feature = "micro-profiling")]
#[test]
fn native_workspace_symbol_cache_allocation_law() {
    use vize_l0::profiler::{
        allocation_snapshot, reset_allocation_counters, set_allocation_tracking_enabled,
    };

    const DOCUMENTS: usize = 32;
    const DECLARATIONS: usize = 96;
    const WARM_READS: u64 = 32;
    let mut source = String::from("<script setup>\n");
    for index in 0..DECLARATIONS {
        source.push_str(vize_l0::cstr!("const symbol_{index:03} = {index}\n").as_str());
    }
    source.push_str("</script>\n<style>.root { color: red; }</style>");

    let state = ServerState::new();
    for index in 0..DOCUMENTS {
        let uri = Url::parse(
            vize_l0::cstr!("file:///workspace/symbol-profile/Component{index:03}.vue").as_str(),
        )
        .unwrap();
        state
            .documents
            .open(uri, source.clone(), 1, String::from("vue"));
    }

    block_on(async {
        let _paths = state.current_project_paths().await;
        let _uris = state.documents.uris();

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        let cold = state
            .search_workspace_project_symbols("definitely_missing_symbol")
            .await;
        let cold_allocations = allocation_snapshot();
        set_allocation_tracking_enabled(false);
        assert!(cold.is_empty());

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        for _ in 0..WARM_READS {
            let warm = state
                .search_workspace_project_symbols("definitely_missing_symbol")
                .await;
            assert!(warm.is_empty());
        }
        let warm_allocations = allocation_snapshot();
        set_allocation_tracking_enabled(false);

        assert!(cold_allocations.allocation_calls() > 0);
        assert!(cold_allocations.requested_bytes() > 0);
        assert!(
            warm_allocations.requested_bytes() < cold_allocations.requested_bytes() / 4,
            "all native warm queries together must allocate less than one cold index build"
        );
        assert_eq!(cold_allocations.allocation_failures(), 0);
        assert_eq!(warm_allocations.allocation_failures(), 0);
        println!(
            "native_workspace_symbol_cache_allocation={}",
            serde_json::json!({
                "documents": DOCUMENTS,
                "declarationsPerDocument": DECLARATIONS,
                "warmReads": WARM_READS,
                "coldCacheMiss": {
                    "allocationCalls": cold_allocations.allocation_calls(),
                    "requestedBytes": cold_allocations.requested_bytes(),
                },
                "warmCacheHits": {
                    "allocationCalls": warm_allocations.allocation_calls(),
                    "requestedBytes": warm_allocations.requested_bytes(),
                },
                "law": "native project-wide worker shares open buffers and reuses their query-independent symbol indexes",
            })
        );

        let disk_root = tempfile::tempdir().unwrap();
        for index in 0..DOCUMENTS {
            std::fs::write(
                disk_root
                    .path()
                    .join(vize_l0::cstr!("Component{index:03}.vue").as_str()),
                &source,
            )
            .unwrap();
        }
        let disk_state = ServerState::new();
        disk_state.set_workspace_root(disk_root.path().to_owned());
        disk_state.set_corsa_disk_watcher_registered();
        let _paths = disk_state.current_project_paths().await;

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        let cold = disk_state
            .search_workspace_project_symbols("definitely_missing_symbol")
            .await;
        let cold_disk_allocations = allocation_snapshot();
        set_allocation_tracking_enabled(false);
        assert!(cold.is_empty());

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        for _ in 0..WARM_READS {
            let warm = disk_state
                .search_workspace_project_symbols("definitely_missing_symbol")
                .await;
            assert!(warm.is_empty());
        }
        let warm_disk_allocations = allocation_snapshot();
        set_allocation_tracking_enabled(false);

        assert!(cold_disk_allocations.allocation_calls() > 0);
        assert!(cold_disk_allocations.requested_bytes() > 0);
        assert!(
            warm_disk_allocations.requested_bytes() < cold_disk_allocations.requested_bytes() / 4,
            "all watched-disk warm queries together must allocate less than one cold index build"
        );
        assert_eq!(cold_disk_allocations.allocation_failures(), 0);
        assert_eq!(warm_disk_allocations.allocation_failures(), 0);
        println!(
            "native_disk_workspace_symbol_cache_allocation={}",
            serde_json::json!({
                "documents": DOCUMENTS,
                "declarationsPerDocument": DECLARATIONS,
                "warmReads": WARM_READS,
                "coldCacheMiss": {
                    "allocationCalls": cold_disk_allocations.allocation_calls(),
                    "requestedBytes": cold_disk_allocations.requested_bytes(),
                },
                "warmCacheHits": {
                    "allocationCalls": warm_disk_allocations.allocation_calls(),
                    "requestedBytes": warm_disk_allocations.requested_bytes(),
                },
                "law": "watched native project files reuse text-free symbol indexes without disk reads or reparsing",
            })
        );
    });
}
