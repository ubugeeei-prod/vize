//! Overlapped content-mapper transforms. Responses stay in request order.

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Condvar, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{Value, json};
use vize_canon::{
    ContentMapperTypeSources, generate_vue_content_mapper_transform_with_type_sources,
};
use vize_l0::cstr;

use super::project::{
    CloseProjectParams, OpenProjectParams, ProjectRegistry, ProjectSettings, resolve_open_project,
};
use super::{InitializeParams, Request, TransformParams, read_frame};

mod wire;
#[cfg(all(test, target_os = "linux", target_pointer_width = "64"))]
use wire::encode_frame;
#[cfg(test)]
pub(super) use wire::wire_value;
use wire::{encode_error, encode_result, encode_transform};

// Actions measured both closure shapes at 4,094 levels in the optimized
// ci-opt profile with 32 MiB and an independent 64 MiB output control.
// Debug v-for replay at 2,048 levels also completed with 32/64 MiB. Keep the
// measured headroom independent of a host's inherited RUST_MIN_STACK setting.
const TRANSFORM_WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Oldest imported-type snapshot a starting transform may still join.
const TYPE_SOURCE_MAX_AGE: Duration = Duration::from_secs(1);

/// Imported type modules shared while transforms overlap. A host burst reads
/// each dependency once. An idle server, or a snapshot older than
/// [`TYPE_SOURCE_MAX_AGE`], reads the filesystem again.
#[derive(Default)]
struct SharedTypeSources {
    state: Mutex<TypeSourceGeneration>,
}

#[derive(Default)]
struct TypeSourceGeneration {
    active: usize,
    current: Option<(ContentMapperTypeSources, Instant)>,
}

impl SharedTypeSources {
    fn acquire(&self) -> ContentMapperTypeSources {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.active += 1;
        if let Some((sources, created)) = &state.current
            && created.elapsed() < TYPE_SOURCE_MAX_AGE
        {
            return sources.clone();
        }
        let sources = ContentMapperTypeSources::default();
        state.current = Some((sources.clone(), Instant::now()));
        sources
    }

    fn release(&self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.active = state.active.saturating_sub(1);
        if state.active == 0 {
            state.current = None;
        }
    }
}

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

/// Serve the protocol. `initialized` means the launcher already answered
/// `initialize`, so project and transform requests are accepted at once.
pub(super) fn serve<R: BufRead, W: Write + Send>(
    reader: &mut R,
    writer: &mut W,
    initialized: bool,
) -> io::Result<()> {
    let workers = thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1);
    serve_with_plan(
        reader,
        writer,
        thread::Builder::new(),
        std::iter::repeat_n(TRANSFORM_WORKER_STACK_BYTES, workers),
        initialized,
    )
    .map(|_| ())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkerPoolReceipt {
    requested_workers: usize,
    effective_workers: usize,
    spawn_failure: Option<vize_l0::String>,
}

fn capture_worker_pool(receipt: &WorkerPoolReceipt) -> io::Result<()> {
    let Some(path) = std::env::var_os("VIZE_CONTENT_MAPPER_WORKER_CAPTURE") else {
        return Ok(());
    };
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    serde_json::to_writer(file, receipt).map_err(io::Error::other)
}

#[cfg(all(test, target_os = "linux", target_pointer_width = "64"))]
fn serve_with_worker_plan<R: BufRead, W: Write + Send>(
    reader: &mut R,
    writer: &mut W,
    writer_builder: thread::Builder,
    worker_stacks: impl ExactSizeIterator<Item = usize>,
) -> io::Result<WorkerPoolReceipt> {
    serve_with_plan(reader, writer, writer_builder, worker_stacks, false)
}

fn serve_with_plan<R: BufRead, W: Write + Send>(
    reader: &mut R,
    writer: &mut W,
    writer_builder: thread::Builder,
    worker_stacks: impl ExactSizeIterator<Item = usize>,
    initialized: bool,
) -> io::Result<WorkerPoolReceipt> {
    let write_error = Mutex::new(None::<io::Error>);
    let progress = WriteProgress::new();
    let (job_tx, job_rx) = mpsc::channel::<TransformJob>();
    let job_rx = Mutex::new(job_rx);
    let (done_tx, done_rx) = mpsc::channel::<(u64, Vec<u8>)>();
    let type_sources = SharedTypeSources::default();
    let mut receipt = WorkerPoolReceipt {
        requested_workers: worker_stacks.len(),
        effective_workers: 0,
        spawn_failure: None,
    };
    let read_result = thread::scope(|scope| {
        let writer_error = &write_error;
        let writer_progress = &progress;
        writer_builder.spawn_scoped(scope, move || {
            if let Err(error) = write_ordered(writer, done_rx, writer_progress) {
                writer_progress.fail();
                *writer_error.lock().unwrap_or_else(PoisonError::into_inner) = Some(error);
            }
        })?;
        for stack_bytes in worker_stacks {
            let jobs = &job_rx;
            let done = done_tx.clone();
            let type_sources = &type_sources;
            if let Err(error) = thread::Builder::new()
                .stack_size(stack_bytes)
                .spawn_scoped(scope, move || worker(jobs, done, type_sources))
            {
                receipt.spawn_failure = Some(cstr!("{error}"));
                if receipt.effective_workers > 0 {
                    break;
                }
                // Scoped threads must see EOF before their automatic join.
                // Returning early with these senders alive deadlocks both
                // the writer and any workers that were already started.
                drop(job_tx);
                drop(done_tx);
                return Err(error);
            }
            receipt.effective_workers += 1;
        }
        let read = match std::num::NonZeroUsize::new(receipt.effective_workers) {
            None => Err(io::Error::other("content-mapper has no transform workers")),
            Some(inflight_limit) => capture_worker_pool(&receipt).and_then(|()| {
                read_loop(
                    reader,
                    &job_tx,
                    &done_tx,
                    &progress,
                    inflight_limit,
                    initialized,
                )
            }),
        };
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
    read_result.map(|()| receipt)
}

fn worker(
    jobs: &Mutex<Receiver<TransformJob>>,
    done: Sender<(u64, Vec<u8>)>,
    type_sources: &SharedTypeSources,
) {
    loop {
        let job = {
            let receiver = jobs.lock().unwrap_or_else(PoisonError::into_inner);
            receiver.recv()
        };
        let Ok(job) = job else { break };
        let seq = job.seq;
        let sources = type_sources.acquire();
        let bytes = match catch_unwind(AssertUnwindSafe(|| transform_frame(&job, &sources))) {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(error)) => encode_error(&job.id, -32603, &cstr!("{error}")).unwrap_or_default(),
            Err(_) => encode_error(&job.id, -32603, "Transform failed").unwrap_or_default(),
        };
        let sent = done.send((seq, bytes));
        // Release after the reply: the last holder frees the snapshot.
        type_sources.release();
        drop(sources);
        if sent.is_err() {
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
    inflight_limit: std::num::NonZeroUsize,
    mut initialized: bool,
) -> io::Result<()> {
    let mut projects = ProjectRegistry::default();
    let mut seq = 0_u64;
    loop {
        // Return admission credit only after the ordered reply is flushed.
        progress.wait_until(seq.saturating_sub(inflight_limit.get() as u64 - 1))?;
        let Some(body) = read_frame(reader)? else {
            break;
        };
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
        json!({"positionEncoding": "utf-8", "diagnosticSource": "vize"}),
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
            encode_result(&id, result)
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
    encode_result(&id, Value::Null)
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

fn transform_frame(job: &TransformJob, sources: &ContentMapperTypeSources) -> io::Result<Vec<u8>> {
    match generate_vue_content_mapper_transform_with_type_sources(
        Path::new(job.file_name.as_str()),
        job.content.as_str(),
        job.settings.transform_options,
        &job.settings.project_context,
        sources,
    ) {
        Ok(result) => encode_transform(&job.id, &result.with_semantic_diagnostics()),
        Err(error) => {
            let message = cstr!("Transform failed: {error}");
            encode_error(&job.id, -32603, &message)
        }
    }
}

#[cfg(all(test, target_os = "linux", target_pointer_width = "64"))]
mod tests;

#[cfg(test)]
mod type_source_tests {
    use super::SharedTypeSources;

    #[test]
    fn overlapping_transforms_share_type_sources_until_idle() {
        let shared = SharedTypeSources::default();
        let first = shared.acquire();
        let overlapping = shared.acquire();
        assert_eq!(first, overlapping);
        shared.release();
        let joined = shared.acquire();
        assert_eq!(first, joined);
        shared.release();
        shared.release();
        let after_idle = shared.acquire();
        assert_ne!(first, after_idle);
        shared.release();
    }
}
