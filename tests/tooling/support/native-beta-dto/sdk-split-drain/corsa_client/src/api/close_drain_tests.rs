//! Fast ownership controls for the local validation patch, not native SDK proof.
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::Shutdown,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use corsa_runtime::block_on;
use serde_json::{Value, json};

use super::ApiClient;
use crate::{CorsaError, ProjectSession, UpdateSnapshotParams};

static NEXT_SOCKET: AtomicUsize = AtomicUsize::new(0);

struct Peer {
    path: PathBuf,
    socket: UnixStream,
    requests: Arc<Mutex<Vec<Value>>>,
    done: mpsc::Receiver<()>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Peer {
    fn connect() -> (Self, ApiClient) {
        let path = std::env::temp_dir().join(format!(
            "corsa-drain-{}-{}.sock",
            std::process::id(),
            NEXT_SOCKET.fetch_add(1, Ordering::SeqCst)
        ));
        let listener = UnixListener::bind(&path).unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&requests);
        let (socket_tx, socket_rx) = mpsc::sync_channel(1);
        let (done_tx, done) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket_tx.send(socket.try_clone().unwrap()).unwrap();
            let mut reader = BufReader::new(socket.try_clone().unwrap());
            let mut head = 0;
            while let Some(request) = read_packet(&mut reader) {
                observed.lock().unwrap().push(request.clone());
                let result = match request["method"].as_str().unwrap() {
                    "initialize" => json!({"caseSensitivity": 1, "currentDirectory": "/owned"}),
                    "createSnapshot" | "updateSnapshot" => {
                        head += 1;
                        json!({"snapshot": head, "projects": [{
                            "id": "project", "configFileName": "/owned/tsconfig.json",
                            "compilerOptions": {}, "rootFiles": ["/owned/main.ts"]
                        }]})
                    }
                    "release" => Value::Null,
                    method => panic!("unexpected terminal-stage request: {method}"),
                };
                write_packet(
                    &mut socket,
                    &json!({"jsonrpc":"2.0", "id":request["id"], "result":result}),
                );
            }
            let _ = done_tx.send(());
        });
        let client = block_on(ApiClient::connect_pipe(&path)).unwrap();
        let socket = socket_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        (
            Self {
                path,
                socket,
                requests,
                done,
                worker: Some(worker),
            },
            client,
        )
    }

    fn snapshot_requests(&self) -> Vec<(String, Value)> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .map(|request| {
                (
                    request["method"].as_str().unwrap().to_owned(),
                    request["params"].clone(),
                )
            })
            .collect()
    }

    fn owner_close(&mut self) {
        self.socket.shutdown(Shutdown::Both).unwrap();
        self.done.recv_timeout(Duration::from_secs(2)).unwrap();
        self.worker.take().unwrap().join().unwrap();
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.socket.shutdown(Shutdown::Both);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

fn read_packet(reader: &mut BufReader<UnixStream>) -> Option<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            return None;
        }
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            length = Some(value.trim().parse::<usize>().unwrap());
        }
    }
    let mut body = vec![0; length.unwrap()];
    reader.read_exact(&mut body).unwrap();
    Some(serde_json::from_slice(&body).unwrap())
}

fn write_packet(socket: &mut UnixStream, packet: &Value) {
    let bytes = serde_json::to_vec(packet).unwrap();
    write!(socket, "Content-Length: {}\r\n\r\n", bytes.len()).unwrap();
    socket.write_all(&bytes).unwrap();
    socket.flush().unwrap();
}

#[test]
fn terminal_drain_releases_cached_head_before_peer_close_and_all_clones_refuse_new_work() {
    let (mut peer, client) = Peer::connect();
    let clone = client.clone();
    let mut cached = block_on(ProjectSession::open(
        client.clone(),
        "/owned/tsconfig.json",
        None,
    ))
    .unwrap();
    let old = cached.snapshot_lease();
    block_on(cached.refresh(None)).unwrap();
    let current = cached.snapshot_lease();
    block_on(old.release()).unwrap();
    block_on(cached.snapshot().release()).unwrap();
    block_on(current.release()).unwrap();
    assert_eq!(
        peer.snapshot_requests()
            .iter()
            .filter(|(method, _)| method == "release")
            .count(),
        1
    );

    block_on(clone.prepare_close()).unwrap();
    block_on(client.prepare_close()).unwrap();
    assert!(
        matches!(peer.done.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "the external peer is still alive during drain"
    );
    let before = peer.snapshot_requests();
    let releases = before
        .iter()
        .filter(|(method, _)| method == "release")
        .map(|(_, params)| params.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        releases,
        [
            json!({"handle":"1", "snapshot":1}),
            json!({"handle":"2", "snapshot":2})
        ]
    );
    assert!(matches!(
        block_on(client.initialize()),
        Err(CorsaError::Closed(_))
    ));
    assert!(matches!(
        block_on(clone.update_snapshot(UpdateSnapshotParams::default())),
        Err(CorsaError::Closed(_))
    ));
    assert!(matches!(
        block_on(client.raw_json_request("getSemanticDiagnostics", json!({}))),
        Err(CorsaError::Closed(_))
    ));
    let late_users = (0..4)
        .map(|_| {
            let client = client.clone();
            thread::spawn(move || {
                assert!(matches!(
                    block_on(client.initialize()),
                    Err(CorsaError::Closed(_))
                ))
            })
        })
        .collect::<Vec<_>>();
    for user in late_users {
        user.join().unwrap();
    }
    assert_eq!(
        peer.snapshot_requests(),
        before,
        "terminal clones issue no late API RPC"
    );
    drop((cached, old, current));
    peer.owner_close();
    block_on(client.close()).unwrap();
    block_on(clone.close()).unwrap();
    assert_eq!(peer.snapshot_requests(), before);
}

#[test]
fn outstanding_nonhead_lease_is_not_falsely_released_after_terminal_drain() {
    let (mut peer, client) = Peer::connect();
    let old = block_on(client.update_snapshot(UpdateSnapshotParams::default())).unwrap();
    let current = block_on(client.update_snapshot(UpdateSnapshotParams::default())).unwrap();
    block_on(current.release()).unwrap();
    block_on(client.prepare_close()).unwrap();
    let before = peer.snapshot_requests();
    assert!(
        matches!(block_on(old.release()), Err(CorsaError::Closed(_))),
        "a retained old lease is an ownership violation, not a successful release"
    );
    assert_eq!(peer.snapshot_requests(), before);
    let releases = before
        .iter()
        .filter(|(method, _)| method == "release")
        .map(|(_, params)| params.clone())
        .collect::<Vec<_>>();
    assert_eq!(releases, [json!({"handle":"2", "snapshot":2})]);
    // Dropping this invalid late lease deliberately retains the stock queue-closed warning.
    drop(old);
    peer.owner_close();
    block_on(client.close()).unwrap();
}
