//! The actual entered framed read outlives an aborted/dropped outer query owner.
use super::*;

#[test]
fn initial_phase_and_diagnostic_are_committed_as_one_cache_entry() {
    use crate::corsa_bridge::{DiagnosticPhase, InitialApiDiagnosticsProvenanceOutcome};
    let peer = Peer::start(1);
    let bridge = peer.bridge();
    let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
        .unwrap()
        .unwrap();
    let mut read = Box::pin(bridge.read_initial_api_diagnostics(request));
    assert!(read.as_mut().now_or_never().is_none());
    peer.entered.recv_timeout(BOUND).unwrap();
    peer.release_read(false);
    let pending = block_on(read);
    let outcome =
        block_on(bridge.finish_initial_api_diagnostics_with_provenance(pending, |commit| commit()))
            .unwrap();
    let InitialApiDiagnosticsProvenanceOutcome::Complete(rows) = outcome else {
        panic!("complete provenance result required");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].phase, DiagnosticPhase::Semantic);
    let cached =
        block_on(bridge.with_client(|client| Ok(client.diagnostics[URI][0].phase))).unwrap();
    assert_eq!(cached, DiagnosticPhase::Semantic);
    let legacy = serde_json::to_value(
        block_on(bridge.with_client(|client| Ok(client.get_diagnostics(URI)))).unwrap(),
    )
    .unwrap();
    assert_eq!(legacy, json!([diagnostic()]));
    assert!(legacy[0].get("phase").is_none());
    assert_eq!(peer.finish(bridge), vec![OLD]);
}

#[test]
fn cancelled_outer_query_keeps_the_exact_bridge_until_the_entered_read_finishes() {
    let peer = Peer::start(1);
    let bridge = peer.bridge();
    let weak = Arc::downgrade(&bridge);
    let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
        .unwrap()
        .unwrap();
    let (cancel, registration) = futures::future::AbortHandle::new_pair();
    let mut query = Box::pin(futures::future::Abortable::new(
        bridge.read_owned_initial_api_diagnostics(request),
        registration,
    ));
    assert!(query.as_mut().now_or_never().is_none());
    peer.entered.recv_timeout(BOUND).unwrap();
    // SourceQueryProject retirement uses this same AbortHandle/Abortable path.
    cancel.abort();
    assert!(block_on(query).is_err());
    drop(bridge);
    let entered_owner = weak
        .upgrade()
        .expect("entered read retains its exact bridge");
    assert!(entered_owner.is_draining());
    peer.release_read(false);
    block_on(
        entered_owner
            .initial_diagnostics_worker
            .submit_ready_async(BOUND, |_| ()),
    )
    .unwrap();
    assert!(!entered_owner.is_draining());
    drop(entered_owner);
    assert!(
        weak.upgrade().is_none(),
        "completed closure cannot leave a bridge cycle"
    );
    // The peer closes after the final expected release; joining observes that EOF
    // without racing an explicit client close against its completed reader.
    assert_eq!(peer.worker.join().unwrap(), vec![OLD]);
}
