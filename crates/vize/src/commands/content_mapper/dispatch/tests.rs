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

struct AdmissionReader {
    frames: Vec<Cursor<Vec<u8>>>,
    current: usize,
    announced: bool,
    admitted: mpsc::Sender<usize>,
}

impl io::BufRead for AdmissionReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        while self.current < self.frames.len()
            && self.frames[self.current].position()
                == self.frames[self.current].get_ref().len() as u64
        {
            self.current += 1;
            self.announced = false;
        }
        if self.current == self.frames.len() {
            return Ok(&[]);
        }
        if !self.announced {
            self.admitted.send(self.current).unwrap();
            self.announced = true;
        }
        io::BufRead::fill_buf(&mut self.frames[self.current])
    }

    fn consume(&mut self, amount: usize) {
        if amount > 0 {
            io::BufRead::consume(&mut self.frames[self.current], amount);
        }
    }
}

impl io::Read for AdmissionReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let available = io::BufRead::fill_buf(self)?;
        let count = available.len().min(output.len());
        output[..count].copy_from_slice(&available[..count]);
        io::BufRead::consume(self, count);
        Ok(count)
    }
}

fn admission_reader(body: &[u8], count: usize) -> (AdmissionReader, mpsc::Receiver<usize>) {
    let mut frame = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    frame.extend_from_slice(body);
    let (admitted, observed) = mpsc::channel();
    (
        AdmissionReader {
            frames: (0..count).map(|_| Cursor::new(frame.clone())).collect(),
            current: 0,
            announced: false,
            admitted,
        },
        observed,
    )
}

#[derive(Default)]
struct WriterGate {
    state: std::sync::Mutex<(usize, bool)>,
    ready: std::sync::Condvar,
}

impl WriterGate {
    fn release(&self) {
        self.state.lock().unwrap().0 += 1;
        self.ready.notify_all();
    }

    fn close(&self) {
        self.state.lock().unwrap().1 = true;
        self.ready.notify_all();
    }
}

struct FailWriterOnDrop<'a>(&'a WriterGate);

impl Drop for FailWriterOnDrop<'_> {
    fn drop(&mut self) {
        self.0.close();
    }
}

struct GatedWriter<'a> {
    gate: &'a WriterGate,
    written: mpsc::Sender<Vec<u8>>,
}

impl io::Write for GatedWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut state = self.gate.state.lock().unwrap();
        while state.0 == 0 && !state.1 {
            state = self.gate.ready.wait(state).unwrap();
        }
        if state.1 {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "controlled writer failure",
            ));
        }
        state.0 -= 1;
        self.written.send(bytes.to_vec()).unwrap();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn admission_waits_before_reading_unknown_malformed_or_uninitialized_requests() {
    for body in [
        &br#"{"id":1,"method":"unknown"}"#[..],
        &b"{"[..],
        &br#"{"id":1,"method":"transform","params":{}}"#[..],
    ] {
        for window in [1, 2, 4] {
            let (mut reader, observed) = admission_reader(body, window + 2);
            let gate = WriterGate::default();
            let (written, frames) = mpsc::channel();
            let mut writer = GatedWriter {
                gate: &gate,
                written,
            };
            thread::scope(|scope| {
                let _wake_on_failure = FailWriterOnDrop(&gate);
                let read = scope.spawn(|| {
                    serve_with_worker_plan(
                        &mut reader,
                        &mut writer,
                        thread::Builder::new(),
                        std::iter::repeat_n(TRANSFORM_WORKER_STACK_BYTES, window),
                    )
                });
                for seq in 0..window {
                    assert_eq!(observed.recv_timeout(Duration::from_secs(5)).unwrap(), seq);
                }
                assert!(
                    matches!(
                        observed.recv_timeout(Duration::from_millis(100)),
                        Err(mpsc::RecvTimeoutError::Timeout)
                    ),
                    "mapper read beyond the effective-worker admission window"
                );
                let mut expected = None;
                for _ in 0..window + 2 {
                    gate.release();
                    let frame = frames.recv_timeout(Duration::from_secs(5)).unwrap();
                    if let Some(expected) = &expected {
                        assert_eq!(&frame, expected);
                    } else {
                        expected = Some(frame);
                    }
                }
                read.join().unwrap().unwrap();
            });
        }
    }
}

#[test]
fn writer_failure_wakes_admission_without_reading_another_frame() {
    let (mut reader, observed) = admission_reader(br#"{"id":1,"method":"unknown"}"#, 2);
    let gate = WriterGate::default();
    let (written, _frames) = mpsc::channel();
    let mut writer = GatedWriter {
        gate: &gate,
        written,
    };
    thread::scope(|scope| {
        let _wake_on_failure = FailWriterOnDrop(&gate);
        let read = scope.spawn(|| {
            serve_with_worker_plan(
                &mut reader,
                &mut writer,
                thread::Builder::new(),
                std::iter::once(TRANSFORM_WORKER_STACK_BYTES),
            )
        });
        assert_eq!(observed.recv_timeout(Duration::from_secs(5)).unwrap(), 0);
        gate.close();
        let error = read.join().unwrap().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(error.to_string(), "controlled writer failure");
        assert!(observed.try_recv().is_err());
    });
}

#[test]
fn admission_preserves_metadata_barriers_errors_and_complete_transform_frames() {
    let packets = [
        json!({"id":1,"method":"initialize","params":{"positionEncodings":["utf-8"]}}),
        json!({"id":2,"method":"openProject","params":{"configFileName":"","projectHandle":"bounded"}}),
        json!({"id":4,"method":"unknown"}),
        json!({"id":5,"method":"transform","params":{"fileName":"/project/Bounded.vue","projectHandle":"bounded","content":"<template><p>雪🌸</p></template>"}}),
        json!({"id":6,"method":"closeProject","params":{"projectHandle":"bounded"}}),
        json!({"id":7,"method":"transform","params":{"fileName":"/project/Bounded.vue","projectHandle":"bounded","content":"<template>closed</template>"}}),
    ];
    let mut input = Vec::new();
    for (index, packet) in packets.iter().enumerate() {
        if index == 2 {
            input.extend_from_slice(b"Content-Length: 1\r\n\r\n{");
        }
        input.extend(super::encode_frame(packet).unwrap());
    }
    let (_, expected) = bounded_plan(
        thread::Builder::new(),
        vec![TRANSFORM_WORKER_STACK_BYTES],
        input.clone(),
    )
    .unwrap();
    let (_, actual) = bounded_plan(
        thread::Builder::new(),
        vec![TRANSFORM_WORKER_STACK_BYTES; 2],
        input,
    )
    .unwrap();
    assert_eq!(actual, expected);
    let mut reader = Cursor::new(actual);
    let mut replies = Vec::<Value>::new();
    while let Some(body) = super::read_frame(&mut reader).unwrap() {
        replies.push(serde_json::from_slice(&body).unwrap());
    }
    assert_eq!(
        replies
            .iter()
            .map(|reply| reply["id"].clone())
            .collect::<Vec<_>>(),
        vec![
            json!(1),
            json!(2),
            Value::Null,
            json!(4),
            json!(5),
            json!(6),
            json!(7)
        ]
    );
    assert_eq!(replies[2]["error"]["code"], -32700);
    assert_eq!(replies[3]["error"]["code"], -32601);
    assert!(!replies[4]["result"]["text"].as_str().unwrap().is_empty());
    assert_eq!(replies[5]["result"], Value::Null);
    assert_eq!(replies[6]["error"]["code"], -32602);
}
