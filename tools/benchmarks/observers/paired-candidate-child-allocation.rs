//! Private untimed overlay only. Preserve the native command result and bytes.
use std::{
    fs::OpenOptions,
    io::{self, Write},
    process::{Child, Command, ExitStatus, Output, Stdio},
};

use serde::Serialize;
use vize_l0::profiler::{AllocationSnapshot, allocation_snapshot};

#[derive(Serialize)]
struct RawCounters {
    alloc_calls: String,
    alloc_zeroed_calls: String,
    alloc_failures: String,
    alloc_zeroed_failures: String,
    alloc_bytes: String,
    alloc_zeroed_bytes: String,
    dealloc_calls: String,
    dealloc_bytes: String,
    realloc_calls: String,
    realloc_failures: String,
    realloc_old_bytes: String,
    realloc_new_bytes: String,
    requested_bytes: String,
    released_bytes: String,
    net_bytes_signed: String,
}

impl From<AllocationSnapshot> for RawCounters {
    fn from(value: AllocationSnapshot) -> Self {
        Self {
            alloc_calls: value.alloc_calls.to_string(),
            alloc_zeroed_calls: value.alloc_zeroed_calls.to_string(),
            alloc_failures: value.alloc_failures.to_string(),
            alloc_zeroed_failures: value.alloc_zeroed_failures.to_string(),
            alloc_bytes: value.alloc_bytes.to_string(),
            alloc_zeroed_bytes: value.alloc_zeroed_bytes.to_string(),
            dealloc_calls: value.dealloc_calls.to_string(),
            dealloc_bytes: value.dealloc_bytes.to_string(),
            realloc_calls: value.realloc_calls.to_string(),
            realloc_failures: value.realloc_failures.to_string(),
            realloc_old_bytes: value.realloc_old_bytes.to_string(),
            realloc_new_bytes: value.realloc_new_bytes.to_string(),
            requested_bytes: value.requested_bytes().to_string(),
            released_bytes: value.released_bytes().to_string(),
            net_bytes_signed: value.net_bytes().to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorRecord {
    stage: &'static str,
    kind: String,
    raw_os_error: Option<i32>,
    message: String,
}

impl ErrorRecord {
    fn new(stage: &'static str, error: &io::Error) -> Self {
        Self {
            stage,
            kind: format!("{:?}", error.kind()),
            raw_os_error: error.raw_os_error(),
            message: error.to_string(),
        }
    }
}

#[derive(Serialize)]
struct StatusRecord {
    success: bool,
    code: Option<i32>,
    display: String,
    #[cfg(unix)]
    signal: Option<i32>,
}

impl From<ExitStatus> for StatusRecord {
    fn from(status: ExitStatus) -> Self {
        Self {
            success: status.success(),
            code: status.code(),
            display: status.to_string(),
            #[cfg(unix)]
            signal: {
                use std::os::unix::process::ExitStatusExt;
                status.signal()
            },
        }
    }
}

// Keep the bracket's probes allocation-free on the usual Running path.
enum Probe {
    Running,
    Exited(ExitStatus),
    Error(io::Error),
}

impl Probe {
    fn take(child: &mut Child) -> Self {
        match child.try_wait() {
            Ok(None) => Self::Running,
            Ok(Some(status)) => Self::Exited(status),
            Err(error) => Self::Error(error),
        }
    }

    fn record(&self, stage: &'static str) -> ProbeRecord {
        match self {
            Self::Running => ProbeRecord {
                state: "running",
                status: None,
                error: None,
            },
            Self::Exited(status) => ProbeRecord {
                state: "exited",
                status: Some((*status).into()),
                error: None,
            },
            Self::Error(error) => ProbeRecord {
                state: "error",
                status: None,
                error: Some(ErrorRecord::new(stage, error)),
            },
        }
    }
}

#[derive(Serialize)]
struct ProbeRecord {
    state: &'static str,
    status: Option<StatusRecord>,
    error: Option<ErrorRecord>,
}

#[derive(Serialize)]
struct Observation<'a> {
    schema: &'static str,
    process_id: u32,
    child_id: Option<u32>,
    pre_spawn: RawCounters,
    bracket_before: Option<ProbeRecord>,
    while_running: Option<RawCounters>,
    bracket_after: Option<ProbeRecord>,
    after_wait: Option<RawCounters>,
    result_stage: &'static str,
    command_error: Option<ErrorRecord>,
    status: Option<StatusRecord>,
    stdout: Option<&'a [u8]>,
    stderr: Option<&'a [u8]>,
    scope: &'static str,
}

#[derive(Debug)]
pub(super) struct ObservationWriteError {
    pub(super) stage: &'static str,
    pub(super) error: io::Error,
}

impl ObservationWriteError {
    fn at(stage: &'static str, error: io::Error) -> Self {
        Self { stage, error }
    }
}

/// Returns the original native result and a separate receipt-write result.
/// Callers must pass the first value unchanged to the original consumers.
/// The controller must reject a missing/failed receipt separately.
pub(super) fn output(
    command: &mut Command,
) -> (io::Result<Output>, Result<(), ObservationWriteError>) {
    let destination = std::env::var_os("VIZE_CHILD_ALLOCATION_OBSERVATION");
    // Match Command::output defaults at this seam: no input, both streams piped.
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let pre_spawn = allocation_snapshot();
    let mut child_id = None;
    let mut before = None;
    let mut after = None;
    let mut running = None;
    let mut after_wait = None;
    let result_stage;
    let result = match command.spawn() {
        Err(error) => {
            result_stage = "spawn";
            Err(error)
        }
        Ok(mut child) => {
            child_id = Some(child.id());
            let first = Probe::take(&mut child);
            if matches!(first, Probe::Running) {
                let snapshot = allocation_snapshot();
                let second = Probe::take(&mut child);
                if matches!(second, Probe::Running) {
                    running = Some(snapshot);
                }
                after = Some(second);
            }
            before = Some(first);
            // Standard library drains both pipes without a custom sampler/thread.
            let output = child.wait_with_output();
            after_wait = Some(allocation_snapshot());
            result_stage = if output.is_ok() {
                "completed"
            } else {
                "wait_with_output"
            };
            output
        }
    };
    // Serialize only after all raw snapshots, leaving counter state untouched.
    let (status, stdout, stderr, command_error) = match &result {
        Ok(output) => (
            Some(output.status.into()),
            Some(output.stdout.as_slice()),
            Some(output.stderr.as_slice()),
            None,
        ),
        Err(error) => (
            None,
            None,
            None,
            Some(ErrorRecord::new(result_stage, error)),
        ),
    };
    let observation = Observation {
        schema: "vize.childAllocationWindow.v1",
        process_id: std::process::id(),
        child_id,
        pre_spawn: pre_spawn.into(),
        bracket_before: before.as_ref().map(|probe| probe.record("try_wait_before")),
        while_running: running.map(Into::into),
        bracket_after: after.as_ref().map(|probe| probe.record("try_wait_after")),
        after_wait: after_wait.map(Into::into),
        result_stage,
        command_error,
        status,
        stdout,
        stderr,
        scope: "global profiling-window requested/released counters; independent relaxed loads, not an atomic cross-field snapshot; signed net is not live/usable heap or RSS; child-process allocations excluded; no reset/pause/trim; while_running exists only when both try_wait probes returned None; standard spawn/wait pipe allocations remain included; raw stdout/stderr byte arrays are complete",
    };
    let receipt = (|| {
        let destination = destination.ok_or_else(|| {
            ObservationWriteError::at(
                "environment",
                io::Error::other("child allocation observation path is required for this overlay"),
            )
        })?;
        let bytes = serde_json::to_vec_pretty(&observation)
            .map_err(|error| ObservationWriteError::at("serialize", io::Error::other(error)))?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|error| ObservationWriteError::at("create_new", error))?;
        file.write_all(&bytes)
            .map_err(|error| ObservationWriteError::at("write_json", error))?;
        file.write_all(b"\n")
            .map_err(|error| ObservationWriteError::at("write_newline", error))
    })();
    (result, receipt)
}
