//! The actual entered framed read outlives an aborted/dropped outer query owner.
use super::*;

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
    block_on(peer.client.close()).unwrap();
    peer.worker.join().unwrap();
}
