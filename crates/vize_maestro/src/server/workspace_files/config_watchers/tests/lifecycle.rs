//! Real late client ACKs survive shutdown and root release; only transport end cancels.

use super::*;

fn held_rpc(host: &mut Harness, retirement: bool, root: &Path) -> Request {
    let first_path = root.join("a.json");
    host.control.update(vec![first_path.clone()]);
    let first = host.next();
    let first_id = registration(&first, &[first_path]);
    if !retirement {
        return first;
    }
    host.reply(&first, true);
    let second_path = root.join("b.json");
    host.control.update(vec![second_path.clone()]);
    let second = host.next();
    registration(&second, &[second_path]);
    host.reply(&second, true);
    let retirement = host.next();
    retired(&retirement, &first_id);
    retirement
}

fn worker(host: &Harness) -> std::thread::JoinHandle<()> {
    host.control
        .worker
        .lock()
        .take()
        .expect("one real watch worker")
}

fn finish(worker: std::thread::JoinHandle<()>) {
    until(|| worker.is_finished());
    worker
        .join()
        .expect("watch worker must finish without a panic");
}

#[test]
fn late_register_and_unregister_ack_after_shutdown_keep_whole_frames_and_emit_no_more_rpc() {
    for retirement in [false, true] {
        let mut host = Harness::supported();
        let root = tempfile::tempdir().expect("workspace");
        let held = held_rpc(&mut host, retirement, root.path());
        let worker = worker(&host);
        host.control.update(vec![root.path().join("latest.json")]);
        assert_eq!(
            host.send(json!({"jsonrpc":"2.0","id":90,"method":"shutdown"})),
            Some(json!({"jsonrpc":"2.0","id":90,"result":null}))
        );
        assert!(
            !worker.is_finished(),
            "shutdown must retain the issued receiver"
        );
        assert!(host.control.shared.queue.lock().latest.is_none());
        let response = Response::from_ok(held.id().expect("request id").clone(), Value::Null);
        assert_eq!(
            serde_json::to_value(&response).expect("whole ACK"),
            json!({"jsonrpc":"2.0","id":held.id(),"result":null})
        );
        block_on(host.socket.send(response)).expect("late ACK remains valid");
        finish(worker);
        assert!(
            host.socket.next().now_or_never().is_none(),
            "shutdown must not issue retirement or queued replacement RPCs"
        );
        assert_eq!(host.send(json!({"jsonrpc":"2.0","method":"exit"})), None);
    }
}

#[test]
fn dropped_root_with_live_client_socket_still_accepts_late_ack_with_or_without_a_lease() {
    for acquire_lease in [false, true] {
        for retirement in [false, true] {
            let mut host = Harness::with_transport(
                json!({"dynamicRegistration":true,"relativePatternSupport":true}),
                acquire_lease,
            );
            let root = tempfile::tempdir().expect("workspace");
            let held = held_rpc(&mut host, retirement, root.path());
            let worker = worker(&host);
            let root_owner = RootOwner::downgrade(&host.control);
            let queue = SharedQueue::downgrade(&host.control.shared);
            let Harness {
                service,
                mut socket,
                control,
                _transport,
            } = host;
            drop(service);
            drop(control);
            assert!(root_owner.upgrade().is_none(), "no retained server owner");
            until(|| queue.upgrade().is_none());
            assert!(
                !worker.is_finished(),
                "root release is not transport termination"
            );
            let response = Response::from_ok(held.id().expect("request id").clone(), Value::Null);
            assert_eq!(
                serde_json::to_value(&response).expect("whole ACK"),
                json!({"jsonrpc":"2.0","id":held.id(),"result":null})
            );
            block_on(socket.send(response)).expect("a live socket may deliver the late ACK");
            finish(worker);
            assert_eq!(
                block_on(crate::runtime::timeout(
                    Duration::from_secs(2),
                    socket.next()
                ))
                .expect("the worker released its client sender"),
                None,
                "no post-drop client RPC"
            );
            // Only now is it safe to signal that no further response can dispatch.
            drop(socket);
            drop(_transport);
        }
    }
}

#[test]
fn actual_transport_close_releases_held_register_and_unregister_without_an_ack() {
    for retirement in [false, true] {
        let mut host = Harness::supported();
        let root = tempfile::tempdir().expect("workspace");
        let _held = held_rpc(&mut host, retirement, root.path());
        let worker = worker(&host);
        host.control.update(vec![root.path().join("latest.json")]);
        assert!(host.control.shared.queue.lock().latest.is_some());
        assert!(
            !worker.is_finished(),
            "the active transport keeps the response receiver"
        );
        let Harness {
            service,
            socket,
            control,
            _transport,
        } = host;
        // The live root deliberately remains: only the actual transport ends.
        drop(service);
        drop(socket);
        drop(_transport);
        finish(worker);
        assert!(control.shared.queue.lock().closed);
        assert!(control.shared.queue.lock().latest.is_none());
        assert_eq!(SharedQueue::strong_count(&control.shared), 1);
    }
}

#[test]
fn actual_transport_close_wakes_idle_worker_while_root_remains_alive() {
    let mut host = Harness::supported();
    let root = tempfile::tempdir().expect("workspace");
    let held = held_rpc(&mut host, false, root.path());
    let worker = worker(&host);
    host.reply(&held, true);
    until(|| SharedQueue::strong_count(&host.control.shared) == 2);
    assert!(
        !worker.is_finished(),
        "worker is waiting for the next configuration update"
    );
    assert!(!host.control.shared.queue.lock().closed);
    assert!(
        host.socket.next().now_or_never().is_none(),
        "no unrequested client RPC"
    );
    let Harness {
        service,
        socket,
        control,
        _transport,
    } = host;
    drop(service);
    drop(socket);
    drop(_transport);
    finish(worker);
    assert!(control.shared.queue.lock().closed);
    assert!(control.shared.queue.lock().latest.is_none());
    assert_eq!(SharedQueue::strong_count(&control.shared), 1);
}
