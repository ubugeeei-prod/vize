//! Query-independent workspace-symbol allocation carrier.

use super::uri;
use vize_l0::profiler::{
    allocation_snapshot, reset_allocation_counters, set_allocation_tracking_enabled,
};
use vize_maestro::ide::WorkspaceSymbolsService;
use vize_maestro::server::ServerState;

pub(super) fn assert_workspace_symbol_cache_allocation_law() {
    const DOCUMENTS: usize = 32;
    const DECLARATIONS: usize = 96;
    const WARM_READS: u64 = 32;
    let mut script = String::from("<script setup>\n");
    for index in 0..DECLARATIONS {
        script.push_str(vize_l0::cstr!("const symbol_{index:03} = {index}\n").as_str());
    }
    script.push_str("</script>\n<style>.root { color: red; }</style>");

    let state = ServerState::new();
    for index in 0..DOCUMENTS {
        state.documents.open(
            uri(vize_l0::cstr!("file:///fixture/symbols/Component{index:03}.vue").as_str()),
            script.clone(),
            1,
            String::from("vue"),
        );
    }

    reset_allocation_counters();
    set_allocation_tracking_enabled(true);
    let cold_result = WorkspaceSymbolsService::search(&state, "definitely_missing_symbol");
    std::hint::black_box(&cold_result);
    let cold = allocation_snapshot();
    set_allocation_tracking_enabled(false);
    assert!(cold_result.is_empty());

    reset_allocation_counters();
    set_allocation_tracking_enabled(true);
    for _ in 0..WARM_READS {
        let warm_result = WorkspaceSymbolsService::search(&state, "definitely_missing_symbol");
        assert!(warm_result.is_empty());
        std::hint::black_box(&warm_result);
    }
    let warm = allocation_snapshot();
    set_allocation_tracking_enabled(false);

    assert!(cold.allocation_calls() > 0);
    assert!(cold.requested_bytes() > 0);
    assert!(
        warm.requested_bytes() < cold.requested_bytes() / 10,
        "all warm queries together must allocate far less than one cold index build"
    );
    assert_eq!(cold.allocation_failures(), 0);
    assert_eq!(warm.allocation_failures(), 0);
    println!(
        "workspace_symbol_cache_allocation={}",
        serde_json::json!({
            "documents": DOCUMENTS,
            "declarationsPerDocument": DECLARATIONS,
            "warmReads": WARM_READS,
            "coldCacheMiss": {
                "allocationCalls": cold.allocation_calls(),
                "requestedBytes": cold.requested_bytes(),
            },
            "warmCacheHits": {
                "allocationCalls": warm.allocation_calls(),
                "requestedBytes": warm.requested_bytes(),
            },
            "law": "all repeated absent queries reuse the query-independent open-document symbol index",
        })
    );
}
