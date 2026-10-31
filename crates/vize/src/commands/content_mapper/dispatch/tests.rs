use std::io::{self, Cursor};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use serde_json::{Value, json};

use super::{TRANSFORM_WORKER_STACK_BYTES, WorkerPoolReceipt, serve_with_worker_plan};

const IMPOSSIBLE_STACK: usize = usize::MAX / 4;

#[test]
fn result_frames_preserve_recursive_legacy_field_order() {
    #[derive(Serialize)]
    struct Opaque {
        zeta: u8,
        alpha: u8,
    }
    #[derive(Serialize)]
    struct Packet<'a> {
        text: &'a str,
        extension: &'a str,
        mappings: [[u32; 6]; 1],
        diagnostics: [u8; 0],
        opaque: Opaque,
    }
    let packet = Packet {
        text: "雪🌸",
        extension: ".tsx",
        mappings: [[1, 2, 3, 4, 5, 6]],
        diagnostics: [],
        opaque: Opaque { zeta: 2, alpha: 1 },
    };
    let body = r#"{"id":7,"jsonrpc":"2.0","result":{"diagnostics":[],"extension":".tsx","mappings":[[1,2,3,4,5,6]],"opaque":{"alpha":1,"zeta":2},"text":"雪🌸"}}"#;
    let mut expected = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    expected.extend_from_slice(body.as_bytes());
    assert_eq!(super::encode_result(&json!(7), packet).unwrap(), expected);
}

fn bounded_plan(
    writer: thread::Builder,
    stacks: Vec<usize>,
    input: Vec<u8>,
) -> io::Result<(WorkerPoolReceipt, Vec<u8>)> {
    let (done, result) = mpsc::channel();
    let _join = thread::spawn(move || {
        let mut output = Vec::new();
        let observed = serve_with_worker_plan(
            &mut Cursor::new(input),
            &mut output,
            writer,
            stacks.into_iter(),
        );
        let _ = done.send(observed.map(|receipt| (receipt, output)));
    });
    result
        .recv_timeout(Duration::from_secs(5))
        .expect("worker spawn failure must not leave a scoped join blocked")
}

#[test]
fn worker_spawn_failure_drops_channels_before_scoped_join() {
    assert!(bounded_plan(thread::Builder::new(), vec![IMPOSSIBLE_STACK], Vec::new()).is_err());
}

#[test]
fn writer_spawn_failure_returns_io_error() {
    assert!(
        bounded_plan(
            thread::Builder::new().stack_size(IMPOSSIBLE_STACK),
            vec![TRANSFORM_WORKER_STACK_BYTES],
            Vec::new(),
        )
        .is_err()
    );
}

#[test]
fn partial_worker_pool_preserves_complete_transform_replies() {
    let mut input = Vec::new();
    for packet in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"positionEncodings":["utf-8"]}}),
        json!({"jsonrpc":"2.0","id":2,"method":"openProject","params":{"configFileName":"","projectHandle":"partial"}}),
        json!({"jsonrpc":"2.0","id":3,"method":"transform","params":{"fileName":"/project/Partial.vue","projectHandle":"partial","content":"<template><p>ok</p></template>"}}),
    ] {
        input.extend(super::encode_frame(&packet).unwrap());
    }
    let (control, expected) = bounded_plan(
        thread::Builder::new(),
        vec![TRANSFORM_WORKER_STACK_BYTES],
        input.clone(),
    )
    .unwrap();
    assert_eq!(
        (control.requested_workers, control.effective_workers),
        (1, 1)
    );
    assert!(control.spawn_failure.is_none());
    let (degraded, observed) = bounded_plan(
        thread::Builder::new(),
        vec![TRANSFORM_WORKER_STACK_BYTES, IMPOSSIBLE_STACK],
        input,
    )
    .unwrap();
    assert_eq!(
        (degraded.requested_workers, degraded.effective_workers),
        (2, 1)
    );
    assert!(degraded.spawn_failure.is_some());
    assert_eq!(observed, expected);
    let mut reader = Cursor::new(observed);
    for id in 1..=3 {
        let packet: Value =
            serde_json::from_slice(&super::read_frame(&mut reader).unwrap().unwrap()).unwrap();
        assert_eq!(packet["id"], id);
        assert!(packet.get("error").is_none());
        if id == 3 {
            assert!(!packet["result"]["text"].as_str().unwrap().is_empty());
        }
    }
    assert!(super::read_frame(&mut reader).unwrap().is_none());
    println!("partial pool: requested=2 effective=1; complete transform replies match normal pool");
}
