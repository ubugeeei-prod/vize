//! Deterministic open and watched-disk allocation carriers.

use super::{ServerState, Url};
use crate::ide::WorkspaceSymbolsService;
use crate::runtime::block_on;
use vize_l0::profiler::{
    allocation_snapshot, reset_allocation_counters, set_allocation_tracking_enabled,
};

#[test]
fn native_workspace_symbol_cache_allocation_law() {
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

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        let legacy_hits =
            WorkspaceSymbolsService::search_cached_open_sources_legacy(&state, "symbol_");
        std::hint::black_box(&legacy_hits);
        let legacy_hit_allocations = allocation_snapshot();
        set_allocation_tracking_enabled(false);

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        let bounded_hits = WorkspaceSymbolsService::search(&state, "symbol_");
        std::hint::black_box(&bounded_hits);
        let bounded_hit_allocations = allocation_snapshot();
        set_allocation_tracking_enabled(false);

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        let native_bounded_hits = state.search_workspace_project_symbols("symbol_").await;
        std::hint::black_box(&native_bounded_hits);
        let native_bounded_hit_allocations = allocation_snapshot();
        set_allocation_tracking_enabled(false);

        assert_eq!(legacy_hits, bounded_hits);
        assert_eq!(legacy_hits, native_bounded_hits);
        assert_eq!(bounded_hits.len(), 100);
        assert!(
            bounded_hit_allocations.requested_bytes()
                < legacy_hit_allocations.requested_bytes() / 4,
            "bounded cached selection must allocate far less than cloning every match"
        );
        assert!(
            bounded_hit_allocations.allocation_calls()
                < legacy_hit_allocations.allocation_calls() / 4,
            "bounded cached selection must make far fewer allocations than cloning every match"
        );
        assert!(
            native_bounded_hit_allocations.requested_bytes()
                < legacy_hit_allocations.requested_bytes() / 2,
            "the native worker must retain the bounded cached-selection advantage"
        );
        assert!(
            native_bounded_hit_allocations.allocation_calls()
                < legacy_hit_allocations.allocation_calls() / 2,
            "the native worker must retain the bounded allocation-count advantage"
        );
        println!(
            "workspace_symbol_hit_selection_allocation={}",
            serde_json::json!({
                "documents": DOCUMENTS,
                "declarationsPerDocument": DECLARATIONS,
                "matchingCandidates": DOCUMENTS * DECLARATIONS,
                "returnedSymbols": bounded_hits.len(),
                "legacyCloneAndSortAll": {
                    "allocationCalls": legacy_hit_allocations.allocation_calls(),
                    "requestedBytes": legacy_hit_allocations.requested_bytes(),
                },
                "boundedDirect": {
                    "allocationCalls": bounded_hit_allocations.allocation_calls(),
                    "requestedBytes": bounded_hit_allocations.requested_bytes(),
                },
                "boundedNativeWorker": {
                    "allocationCalls": native_bounded_hit_allocations.allocation_calls(),
                    "requestedBytes": native_bounded_hit_allocations.requested_bytes(),
                },
                "law": "cached matches are ranked as owner/index candidates and only the final wire-limited symbols are cloned",
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
