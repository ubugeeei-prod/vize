#![cfg(unix)]

use super::{
    InboundEvent, JsonRpcConnection, JsonRpcConnectionOptions, RawMessage, RpcHandlerMap,
    read_frame, write_frame,
};
use corsa_core::{CorsaEvent, CorsaObserver};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::BufReader, os::unix::net::UnixStream, thread};

#[derive(Default)]
struct EventCollector {
    events: Mutex<Vec<CorsaEvent>>,
}

impl CorsaObserver for EventCollector {
    fn on_event(&self, event: &CorsaEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}

#[test]
fn routes_request_and_response() {
    let (client_socket, server_socket) = UnixStream::pair().unwrap();
    let client = JsonRpcConnection::try_spawn(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();
    let server = JsonRpcConnection::try_spawn(
        BufReader::new(server_socket.try_clone().unwrap()),
        server_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();
    let events = server.subscribe();
    let waiter = thread::spawn(move || match events.recv().unwrap() {
        InboundEvent::Request { id, method, params } => {
            assert_eq!(method.as_str(), "ping");
            assert_eq!(params, json!({"value": 1}));
            server.respond(id, json!({"pong": true})).unwrap();
        }
        _ => panic!("unexpected event"),
    });
    let response: serde_json::Value =
        corsa_runtime::block_on(client.request("ping", json!({"value": 1}))).unwrap();
    waiter.join().unwrap();
    assert_eq!(response, json!({"pong": true}));
}

#[test]
fn params_less_send_apis_omit_params_on_the_wire() {
    let (client_socket, mut server_socket) = UnixStream::pair().unwrap();
    let client = JsonRpcConnection::try_spawn(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();
    let server_reader = server_socket.try_clone().unwrap();
    let server = thread::spawn(move || {
        let mut reader = BufReader::new(server_reader);
        let request_payload = read_frame(&mut reader).unwrap();
        let request: serde_json::Value = serde_json::from_slice(&request_payload).unwrap();
        assert_eq!(request["method"], json!("shutdown"));
        assert!(!request.as_object().unwrap().contains_key("params"));

        let response = RawMessage::response(
            serde_json::from_value(request["id"].clone()).unwrap(),
            json!({"ok": true}),
        );
        write_frame(&mut server_socket, &serde_json::to_vec(&response).unwrap()).unwrap();

        let notification_payload = read_frame(&mut reader).unwrap();
        let notification: serde_json::Value =
            serde_json::from_slice(&notification_payload).unwrap();
        assert_eq!(notification["method"], json!("exit"));
        assert!(!notification.as_object().unwrap().contains_key("params"));
    });

    let response: serde_json::Value =
        corsa_runtime::block_on(client.request_without_params("shutdown")).unwrap();
    assert_eq!(response, json!({"ok": true}));
    client.notify_without_params("exit").unwrap();
    server.join().unwrap();
    corsa_runtime::block_on(client.close()).unwrap();
}

#[test]
fn request_times_out_when_no_response_arrives() {
    let (client_socket, _server_socket) = UnixStream::pair().unwrap();
    let observer = Arc::new(EventCollector::default());
    let client = JsonRpcConnection::try_spawn_with_options(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
        JsonRpcConnectionOptions::new()
            .with_request_timeout(Some(Duration::from_millis(10)))
            .with_observer(observer.clone()),
    )
    .unwrap();
    let error =
        corsa_runtime::block_on(client.request_value("ping", json!({"value": 1}))).unwrap_err();
    assert!(matches!(
        error,
        crate::CorsaError::Timeout(message) if message.contains("jsonrpc request `ping`")
    ));
    assert_eq!(
        observer.events.lock().unwrap().as_slice(),
        &[CorsaEvent::JsonRpcRequestTimedOut {
            method: "ping".into(),
            timeout: Duration::from_millis(10),
        }]
    );
}

#[test]
fn unhandled_request_without_subscriber_returns_method_not_found() {
    let (client_socket, server_socket) = UnixStream::pair().unwrap();
    let client = JsonRpcConnection::try_spawn_with_options(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
        JsonRpcConnectionOptions::new().with_request_timeout(Some(Duration::from_secs(5))),
    )
    .unwrap();
    let server = JsonRpcConnection::try_spawn(
        BufReader::new(server_socket.try_clone().unwrap()),
        server_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();

    let error =
        corsa_runtime::block_on(server.request_value("missing", json!({"value": 1}))).unwrap_err();

    assert!(matches!(
        error,
        crate::CorsaError::Rpc(error)
            if error.code == -32601 && error.message.contains("method not found: missing")
    ));

    let _ = corsa_runtime::block_on(client.close());
    let _ = corsa_runtime::block_on(server.close());
}

#[test]
fn close_joins_reader_after_peer_closes() {
    let (client_socket, server_socket) = UnixStream::pair().unwrap();
    let client = JsonRpcConnection::try_spawn(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();
    drop(server_socket);

    corsa_runtime::block_on(client.close()).unwrap();
}

#[test]
fn close_uses_bounded_reader_join_when_peer_stays_open() {
    let (client_socket, _server_socket) = UnixStream::pair().unwrap();
    let client = JsonRpcConnection::try_spawn(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();
    let started = std::time::Instant::now();

    corsa_runtime::block_on(client.close()).unwrap();

    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn request_handler_panic_returns_error_and_keeps_connection_alive() {
    let (client_socket, server_socket) = UnixStream::pair().unwrap();
    let observer = Arc::new(EventCollector::default());
    let mut handlers = RpcHandlerMap::default();
    handlers.insert(
        "boom".into(),
        Arc::new(
            |_| -> std::result::Result<serde_json::Value, crate::RpcResponseError> {
                panic!("jsonrpc test handler panic");
            },
        ),
    );
    handlers.insert("echo".into(), Arc::new(Ok));
    let client = JsonRpcConnection::try_spawn_with_options(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        handlers,
        JsonRpcConnectionOptions::new()
            .with_request_timeout(Some(Duration::from_secs(5)))
            .with_observer(observer.clone()),
    )
    .unwrap();
    let server = JsonRpcConnection::try_spawn(
        BufReader::new(server_socket.try_clone().unwrap()),
        server_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();

    let error =
        corsa_runtime::block_on(server.request_value("boom", json!({"value": 1}))).unwrap_err();
    assert!(matches!(
        error,
        crate::CorsaError::Rpc(error)
            if error.code == -32603
                && error.message.contains("local JSON-RPC handler `boom` panicked")
    ));

    let response: serde_json::Value =
        corsa_runtime::block_on(server.request_value("echo", json!({"value": 2}))).unwrap();
    assert_eq!(response, json!({"value": 2}));
    assert_eq!(
        observer.events.lock().unwrap().as_slice(),
        &[CorsaEvent::JsonRpcLocalHandlerPanicked {
            method: "boom".into(),
        }]
    );

    let _ = corsa_runtime::block_on(client.close());
    let _ = corsa_runtime::block_on(server.close());
}

#[test]
fn notification_handler_panic_does_not_fail_pending_requests() {
    let (client_socket, server_socket) = UnixStream::pair().unwrap();
    let mut handlers = RpcHandlerMap::default();
    handlers.insert(
        "boom".into(),
        Arc::new(
            |_| -> std::result::Result<serde_json::Value, crate::RpcResponseError> {
                panic!("jsonrpc test handler panic");
            },
        ),
    );
    let client = JsonRpcConnection::try_spawn_with_options(
        BufReader::new(client_socket.try_clone().unwrap()),
        client_socket,
        handlers,
        JsonRpcConnectionOptions::new().with_request_timeout(Some(Duration::from_secs(5))),
    )
    .unwrap();
    let server = JsonRpcConnection::try_spawn(
        BufReader::new(server_socket.try_clone().unwrap()),
        server_socket,
        RpcHandlerMap::default(),
    )
    .unwrap();
    let server_events = server.subscribe();
    let pending_client = client.clone();
    let waiter = thread::spawn(move || {
        corsa_runtime::block_on(pending_client.request_value("never", json!({})))
    });

    let id = match server_events
        .recv_timeout(Duration::from_secs(1))
        .expect("server should receive pending request")
    {
        InboundEvent::Request { id, method, .. } => {
            assert_eq!(method.as_str(), "never");
            id
        }
        _ => panic!("unexpected event"),
    };

    server.notify("boom", json!({})).unwrap();
    server.respond(id, json!({"ok": true})).unwrap();
    assert_eq!(waiter.join().unwrap().unwrap(), json!({"ok": true}));

    let _ = corsa_runtime::block_on(client.close());
    let _ = corsa_runtime::block_on(server.close());
}
