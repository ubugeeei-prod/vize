//! Overlapped content-mapper transforms. Responses stay in request order.

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Condvar, Mutex, PoisonError};
use std::thread;

use serde::Serialize;
use serde_json::{Value, json};
use vize_canon::generate_vue_content_mapper_transform_with_project_options;
use vize_l0::cstr;

use super::project::{
    CloseProjectParams, OpenProjectParams, ProjectRegistry, ProjectSettings, resolve_open_project,
};
use super::{InitializeParams, Request, TransformParams, read_frame};

// Actions measured both closure shapes at 4,094 levels in the optimized
// ci-opt profile with 32 MiB and an independent 64 MiB output control.
// Debug v-for replay at 2,048 levels also completed with 32/64 MiB. Keep the
// measured headroom independent of a host's inherited RUST_MIN_STACK setting.
const TRANSFORM_WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

struct TransformJob {
    seq: u64,
    id: Value,
    file_name: vize_l0::String,
    content: vize_l0::String,
    settings: ProjectSettings,
}

struct WriteProgress {
    state: Mutex<WriteState>,
    ready: Condvar,
}

struct WriteState {
    flushed: u64,
    failed: bool,
}

impl WriteProgress {
    fn new() -> Self {
        Self {
            state: Mutex::new(WriteState {
                flushed: 0,
                failed: false,
            }),
            ready: Condvar::new(),
        }
    }

    fn record(&self, flushed: u64) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.flushed = flushed;
        self.ready.notify_all();
    }

    fn fail(&self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.failed = true;
        self.ready.notify_all();
    }

    fn wait_until(&self, flushed: u64) -> io::Result<()> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        while state.flushed < flushed && !state.failed {
            state = self
                .ready
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        if state.failed && state.flushed < flushed {
            Err(io::Error::other("content-mapper writer stopped"))
        } else {
            Ok(())
        }
    }
}

pub(super) fn serve<R: BufRead, W: Write + Send>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let write_error = Mutex::new(None::<io::Error>);
    let progress = WriteProgress::new();
    let (job_tx, job_rx) = mpsc::channel::<TransformJob>();
    let job_rx = Mutex::new(job_rx);
    let (done_tx, done_rx) = mpsc::channel::<(u64, Vec<u8>)>();
    let read_result = thread::scope(|scope| {
        let writer_error = &write_error;
        let writer_progress = &progress;
        scope.spawn(move || {
            if let Err(error) = write_ordered(writer, done_rx, writer_progress) {
                writer_progress.fail();
                *writer_error.lock().unwrap_or_else(PoisonError::into_inner) = Some(error);
            }
        });
        let workers = thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1);
        for _ in 0..workers {
            let jobs = &job_rx;
            let done = done_tx.clone();
            thread::Builder::new()
                .stack_size(TRANSFORM_WORKER_STACK_BYTES)
                .spawn_scoped(scope, move || worker(jobs, done))?;
        }
        let read = read_loop(reader, &job_tx, &done_tx, &progress);
        drop(job_tx);
        drop(done_tx);
        read
    });
    if let Some(error) = write_error
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()
    {
        return Err(error);
    }
    read_result
}

fn worker(jobs: &Mutex<Receiver<TransformJob>>, done: Sender<(u64, Vec<u8>)>) {
    loop {
        let job = {
            let receiver = jobs.lock().unwrap_or_else(PoisonError::into_inner);
            receiver.recv()
        };
        let Ok(job) = job else { break };
        let seq = job.seq;
        let bytes = match catch_unwind(AssertUnwindSafe(|| transform_frame(&job))) {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(error)) => encode_error(&job.id, -32603, &cstr!("{error}")).unwrap_or_default(),
            Err(_) => encode_error(&job.id, -32603, "Transform failed").unwrap_or_default(),
        };
        if done.send((seq, bytes)).is_err() {
            break;
        }
    }
}

fn write_ordered<W: Write>(
    writer: &mut W,
    done: Receiver<(u64, Vec<u8>)>,
    progress: &WriteProgress,
) -> io::Result<()> {
    let mut next = 0_u64;
    let mut pending = BTreeMap::<u64, Vec<u8>>::new();
    while let Ok((seq, bytes)) = done.recv() {
        pending.insert(seq, bytes);
        while let Some(frame) = pending.remove(&next) {
            writer.write_all(&frame)?;
            writer.flush()?;
            next += 1;
            progress.record(next);
        }
    }
    Ok(())
}

fn read_loop<R: BufRead>(
    reader: &mut R,
    jobs: &Sender<TransformJob>,
    done: &Sender<(u64, Vec<u8>)>,
    progress: &WriteProgress,
) -> io::Result<()> {
    let mut initialized = false;
    let mut projects = ProjectRegistry::default();
    let mut seq = 0_u64;
    while let Some(body) = read_frame(reader)? {
        let request = match serde_json::from_slice::<Request>(&body) {
            Ok(request) => request,
            Err(error) => {
                let message = cstr!("Parse error: {error}");
                let frame = encode_error(&Value::Null, -32700, &message)?;
                reply(done, &mut seq, &frame)?;
                continue;
            }
        };
        let id = request.id.unwrap_or(Value::Null);
        match request.method.as_str() {
            "initialize" => {
                progress.wait_until(seq)?;
                let frame = initialize(id, request.params, &mut initialized)?;
                reply(done, &mut seq, &frame)?;
            }
            "openProject" | "closeProject" | "transform" if !initialized => {
                let frame = encode_error(&id, -32002, "Content mapper is not initialized")?;
                reply(done, &mut seq, &frame)?;
            }
            "openProject" => {
                progress.wait_until(seq)?;
                let frame = open_project(id, request.params, &mut projects)?;
                reply(done, &mut seq, &frame)?;
            }
            "closeProject" => {
                progress.wait_until(seq)?;
                let frame = close_project(id, request.params, &mut projects)?;
                reply(done, &mut seq, &frame)?;
            }
            "transform" => dispatch_transform(id, request.params, &projects, jobs, done, &mut seq)?,
            _ => {
                let message = cstr!("Method not found: {}", request.method);
                let frame = encode_error(&id, -32601, &message)?;
                reply(done, &mut seq, &frame)?;
            }
        }
    }
    Ok(())
}

fn reply(done: &Sender<(u64, Vec<u8>)>, seq: &mut u64, frame: &[u8]) -> io::Result<()> {
    done.send((*seq, frame.to_vec()))
        .map_err(|_| io::Error::other("content-mapper writer stopped"))?;
    *seq += 1;
    Ok(())
}

fn initialize(id: Value, params: Value, initialized: &mut bool) -> io::Result<Vec<u8>> {
    let params = match serde_json::from_value::<InitializeParams>(params) {
        Ok(params) => params,
        Err(error) => {
            return encode_error(&id, -32602, &cstr!("Invalid initialize params: {error}"));
        }
    };
    if !params
        .position_encodings
        .iter()
        .any(|encoding| encoding == "utf-8")
    {
        return encode_error(&id, -32602, "Vize requires the utf-8 position encoding");
    }
    *initialized = true;
    encode_result(
        &id,
        &json!({"positionEncoding": "utf-8", "diagnosticSource": "vize"}),
    )
}

fn open_project(id: Value, params: Value, projects: &mut ProjectRegistry) -> io::Result<Vec<u8>> {
    let params = match serde_json::from_value::<OpenProjectParams>(params) {
        Ok(params) => params,
        Err(error) => {
            let message = cstr!("Invalid openProject params: {error}");
            return encode_error(&id, -32602, &message);
        }
    };
    match resolve_open_project(&params) {
        Ok((settings, result)) => {
            projects.open(params.project_handle, settings);
            encode_result(&id, &result)
        }
        Err(error) => {
            projects.close(params.project_handle.as_str());
            let message = cstr!("Failed to load content mapper configuration: {error}");
            encode_error(&id, -32603, &message)
        }
    }
}

fn close_project(id: Value, params: Value, projects: &mut ProjectRegistry) -> io::Result<Vec<u8>> {
    let params = match serde_json::from_value::<CloseProjectParams>(params) {
        Ok(params) => params,
        Err(error) => {
            let message = cstr!("Invalid closeProject params: {error}");
            return encode_error(&id, -32602, &message);
        }
    };
    projects.close(params.project_handle.as_str());
    encode_result(&id, &Value::Null)
}

fn dispatch_transform(
    id: Value,
    params: Value,
    projects: &ProjectRegistry,
    jobs: &Sender<TransformJob>,
    done: &Sender<(u64, Vec<u8>)>,
    seq: &mut u64,
) -> io::Result<()> {
    let params = match serde_json::from_value::<TransformParams>(params) {
        Ok(params) => params,
        Err(error) => {
            let message = cstr!("Invalid transform params: {error}");
            return reply(done, seq, &encode_error(&id, -32602, &message)?);
        }
    };
    let Some(settings) = projects.settings(params.project_handle.as_str()).cloned() else {
        let message = cstr!("Unknown content mapper project: {}", params.project_handle);
        return reply(done, seq, &encode_error(&id, -32602, &message)?);
    };
    let job = TransformJob {
        seq: *seq,
        id,
        file_name: params.file_name,
        content: params.content,
        settings,
    };
    *seq += 1;
    jobs.send(job)
        .map_err(|_| io::Error::other("content-mapper workers stopped"))
}

fn transform_frame(job: &TransformJob) -> io::Result<Vec<u8>> {
    match generate_vue_content_mapper_transform_with_project_options(
        Path::new(job.file_name.as_str()),
        job.content.as_str(),
        job.settings.transform_options,
        job.settings.typed_router_root.as_deref(),
    ) {
        Ok(result) => encode_result(&job.id, &result),
        Err(error) => {
            let message = cstr!("Transform failed: {error}");
            encode_error(&job.id, -32603, &message)
        }
    }
}

fn encode_result(id: &Value, result: &impl Serialize) -> io::Result<Vec<u8>> {
    #[derive(Serialize)]
    struct Envelope<'a, T> {
        id: &'a Value,
        jsonrpc: &'static str,
        result: &'a T,
    }
    encode_frame(&Envelope {
        id,
        jsonrpc: "2.0",
        result,
    })
}

fn encode_error(id: &Value, code: i32, message: &str) -> io::Result<Vec<u8>> {
    encode_frame(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message},
    }))
}

fn encode_frame(value: &impl Serialize) -> io::Result<Vec<u8>> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    let mut frame = Vec::with_capacity(body.len() + 32);
    write!(&mut frame, "Content-Length: {}\r\n\r\n", body.len())?;
    frame.extend_from_slice(&body);
    Ok(frame)
}
