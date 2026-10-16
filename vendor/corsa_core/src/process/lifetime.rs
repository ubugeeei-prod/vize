//! Non-owning terminal evidence survives failed constructors and owner Drop.
use crate::fast::CompactString;
use crate::{ProcessObserver, ProcessReceipt};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone, Debug)]
pub struct ProcessCloseReceipt {
    pub pid: u32,
    pub status: std::process::ExitStatus,
    pub timed_out: bool,
    pub kill_requested: bool,
    pub reaped: bool,
}

#[derive(Clone, Debug)]
pub struct ProcessCloseObservation {
    pub spawn_sequence: u64,
    pub expected_pid: u32,
    pub context: CompactString,
    pub terminal: Option<ProcessCloseReceipt>,
    pub raw_errors: Vec<String>,
    pub close_error: Option<String>,
    pub stderr_receipts: Option<Vec<ProcessReceipt>>,
}

impl ProcessCloseObservation {
    pub fn custody_complete(&self) -> bool {
        if self.close_error.is_some()
            || !self
                .terminal
                .as_ref()
                .is_some_and(|terminal| terminal.pid == self.expected_pid && terminal.reaped)
        {
            return false;
        }
        self.stderr_receipts.as_ref().is_none_or(|receipts| {
            receipts.len() == 1
                && receipts.iter().all(|receipt| {
                    receipt.pid == self.expected_pid
                        && receipt.completed
                        && receipt.reaped
                        && receipt.stderr_complete
                        && receipt.stderr_error.is_none()
                        && receipt.process_error.is_none()
                        && self.terminal.as_ref().is_some_and(|terminal| {
                            receipt.exit_status == Some(terminal.status)
                                && receipt.kill_requested == terminal.kill_requested
                                && receipt.timed_out == terminal.timed_out
                        })
                })
        })
    }
}

/// This Arc owns evidence only; it never owns a Child, pipe, or transport.
#[derive(Clone, Debug)]
pub struct ProcessTerminalObserver(Arc<Terminal>);

#[derive(Debug)]
struct Terminal {
    observation: Mutex<ProcessCloseObservation>,
    stderr_observer: Mutex<Option<ProcessObserver>>,
    done: Condvar,
}

impl ProcessTerminalObserver {
    pub(super) fn new(pid: u32, context: CompactString) -> Self {
        Self::with_sequence(pid, context, 0)
    }

    fn with_sequence(pid: u32, context: CompactString, spawn_sequence: u64) -> Self {
        Self(Arc::new(Terminal {
            observation: Mutex::new(ProcessCloseObservation {
                spawn_sequence,
                expected_pid: pid,
                context,
                terminal: None,
                raw_errors: Vec::new(),
                close_error: None,
                stderr_receipts: None,
            }),
            stderr_observer: Mutex::new(None),
            done: Condvar::new(),
        }))
    }

    pub fn snapshot(&self) -> ProcessCloseObservation {
        let observation = self
            .0
            .observation
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        self.snapshot_stderr(observation)
    }

    fn snapshot_stderr(&self, mut observation: ProcessCloseObservation) -> ProcessCloseObservation {
        // No live observer escapes the snapshot. Release metadata locks before
        // copying stderr so the published terminal and bytes are immutable.
        let observer = self
            .0
            .stderr_observer
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        observation.stderr_receipts = observer.map(|observer| observer.receipts());
        observation
    }

    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// Bounded metadata wait; owns no process and holds no handshake lock.
    pub fn wait_for_terminal(&self, deadline: std::time::Instant) -> ProcessCloseObservation {
        let mut observation = self
            .0
            .observation
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while observation.terminal.is_none() {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            let (next, timeout) = self
                .0
                .done
                .wait_timeout(observation, remaining)
                .unwrap_or_else(|error| error.into_inner());
            observation = next;
            if timeout.timed_out() {
                break;
            }
        }
        let snapshot = observation.clone();
        drop(observation);
        self.snapshot_stderr(snapshot)
    }

    pub(super) fn observe_stderr(&self, observer: Option<ProcessObserver>) {
        *self
            .0
            .stderr_observer
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = observer;
    }

    /// Retain an SDK setup/protocol error without inventing a terminal result.
    pub fn record_error(&self, error: impl Into<String>) {
        self.0
            .observation
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .raw_errors
            .push(error.into());
    }

    pub(super) fn finish(&self, receipt: ProcessCloseReceipt, error: Option<String>) {
        let mut observation = self
            .0
            .observation
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if observation.expected_pid == receipt.pid {
            observation.terminal = Some(receipt);
            observation.close_error = error.clone();
            if let Some(error) = error {
                observation.raw_errors.push(error);
            }
        } else {
            let expected = observation.expected_pid;
            observation.raw_errors.push(format!(
                "terminal PID mismatch: expected {expected}, actual {}",
                receipt.pid
            ));
        }
        self.0.done.notify_all();
    }
}

#[derive(Default, Debug)]
struct Collection {
    observed: u64,
    published: u64,
    pending: Vec<ProcessTerminalObserver>,
}

#[derive(Clone, Default, Debug)]
pub struct ProcessLifetimeObserver(Arc<Mutex<Collection>>);

impl PartialEq for ProcessLifetimeObserver {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ProcessLifetimeObserver {}
impl std::hash::Hash for ProcessLifetimeObserver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&Arc::as_ptr(&self.0), state);
    }
}

#[derive(Clone, Debug)]
pub struct ProcessLifetimeSnapshot {
    pub observed: u64,
    pub published: u64,
    pub pending: Vec<ProcessCloseObservation>,
    /// Exact tokens whose terminal evidence is present in this snapshot.
    pub completed: Vec<ProcessTerminalObserver>,
}

#[derive(Clone, Debug)]
pub enum ProcessOwnership {
    Owned(ProcessTerminalObserver),
    /// Socket attachments have no independently owned process.
    Attached,
}

impl ProcessLifetimeObserver {
    pub fn unpublished(&self) -> Vec<ProcessTerminalObserver> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .pending
            .clone()
    }
    pub(super) fn observe(&self, pid: u32, context: CompactString) -> ProcessTerminalObserver {
        let mut collection = self.0.lock().unwrap_or_else(|error| error.into_inner());
        collection.observed += 1;
        let observer = ProcessTerminalObserver::with_sequence(pid, context, collection.observed);
        collection.pending.push(observer.clone());
        observer
    }

    pub fn snapshot(&self) -> ProcessLifetimeSnapshot {
        let collection = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let mut pending = Vec::with_capacity(collection.pending.len());
        let mut completed = Vec::new();
        for observer in &collection.pending {
            let observation = observer.snapshot();
            if observation.custody_complete() {
                completed.push(observer.clone());
            }
            pending.push(observation);
        }
        ProcessLifetimeSnapshot {
            observed: collection.observed,
            published: collection.published,
            pending,
            completed,
        }
    }

    /// Prune exact completed observers only after their evidence was published.
    pub fn acknowledge_publication(&self, published: &[ProcessTerminalObserver]) {
        let mut collection = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let before = collection.pending.len();
        collection.pending.retain(|observer| {
            !(observer.snapshot().custody_complete()
                && published
                    .iter()
                    .any(|entry| Arc::ptr_eq(&entry.0, &observer.0)))
        });
        collection.published += (before - collection.pending.len()) as u64;
    }
}
