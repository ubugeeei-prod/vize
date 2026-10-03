//! Sendable commands into one thread-local original Program/File lifetime.
#![expect(
    clippy::disallowed_types,
    reason = "consumer workers retain genuine snapshot Arcs"
)]

use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
};

use futures::channel::oneshot;
use tower_lsp::lsp_types::{Location, Position};

use super::{NavigationRefusal, SourceSnapshot, profile::Profile, retained};

pub(super) mod selected;

pub(super) const WORKER_LIMIT: usize = 16;
const REQUEST_LIMIT: usize = 16;

pub(super) enum Command {
    TemplateDefinition(
        Position,
        oneshot::Sender<Result<Vec<Location>, NavigationRefusal>>,
    ),
    #[cfg(test)]
    SelectedInspect(
        Position,
        oneshot::Sender<Result<selected::Inspection, NavigationRefusal>>,
    ),
    Definition(
        Position,
        oneshot::Sender<Result<Option<Location>, NavigationRefusal>>,
    ),
    References(
        Position,
        bool,
        oneshot::Sender<Result<Vec<Location>, NavigationRefusal>>,
    ),
    Stop,
    #[cfg(test)]
    Inspect(
        Position,
        oneshot::Sender<Result<Inspection, NavigationRefusal>>,
    ),
    #[cfg(test)]
    Pause(std::sync::mpsc::Sender<()>, Receiver<()>),
    #[cfg(test)]
    Panic,
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Inspection {
    pub(super) file: usize,
    pub(super) program: usize,
    pub(super) statements: usize,
    pub(super) declaration: Option<vize_l0::Span>,
    pub(super) parses: usize,
    pub(super) sfc: Option<SfcInspection>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SfcInspection {
    pub(super) observation: usize,
    pub(super) descriptor: usize,
    pub(super) programs: Vec<(usize, usize, vize_l0::Span)>,
    pub(super) expressions: Vec<(usize, vize_l0::Span)>,
    pub(super) productions: usize,
}

pub(super) struct Control {
    retired: AtomicBool,
    finished: AtomicBool,
    #[cfg(test)]
    pub(super) parses: AtomicUsize,
    #[cfg(test)]
    pub(super) sfc_productions: AtomicUsize,
    #[cfg(test)]
    pub(super) queries: AtomicUsize,
}

impl Control {
    pub(super) fn retired(&self) -> bool {
        self.retired.load(Ordering::Acquire)
    }
}

struct Slot(Arc<AtomicUsize>);
impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

struct Exit {
    control: Arc<Control>,
    #[cfg(test)]
    signal: std::sync::mpsc::Sender<()>,
}
impl Drop for Exit {
    fn drop(&mut self) {
        self.control.finished.store(true, Ordering::Release);
        #[cfg(test)]
        let _ = self.signal.send(());
    }
}

/// No native arena, AST or File is stored in this cross-thread handle.
pub(super) struct NavigationWorker {
    snapshot: Weak<SourceSnapshot>,
    profile: Profile,
    commands: SyncSender<Command>,
    control: Arc<Control>,
    #[cfg(test)]
    exited: parking_lot::Mutex<Receiver<()>>,
}

impl NavigationWorker {
    pub(super) fn spawn(
        snapshot: Arc<SourceSnapshot>,
        live: Arc<AtomicUsize>,
        profile: Profile,
    ) -> Result<Arc<Self>, NavigationRefusal> {
        live.fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
            (count < WORKER_LIMIT).then_some(count + 1)
        })
        .map_err(|_| NavigationRefusal::Capacity)?;
        let slot = Slot(live);
        let (commands, receiver) = sync_channel(REQUEST_LIMIT);
        let control = Arc::new(Control {
            retired: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            #[cfg(test)]
            parses: AtomicUsize::new(0),
            #[cfg(test)]
            sfc_productions: AtomicUsize::new(0),
            #[cfg(test)]
            queries: AtomicUsize::new(0),
        });
        #[cfg(test)]
        let (signal, exited) = std::sync::mpsc::channel();
        let owner = Arc::clone(&snapshot);
        let stop = Arc::clone(&control);
        std::thread::Builder::new()
            .name("native-file-navigation".into())
            .spawn(move || {
                // The live slot is released only after every native local drops.
                let _exit = Exit {
                    control: Arc::clone(&stop),
                    #[cfg(test)]
                    signal,
                };
                let _slot = slot;
                match profile {
                    Profile::Program(options) => retained::run(owner, options, receiver, stop),
                    Profile::Vue(configuration) => {
                        retained::vue::run(owner, configuration, receiver, stop);
                    }
                    Profile::SelectedVue(configuration) => {
                        retained::selected::run(owner, configuration, receiver, stop);
                    }
                }
            })
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?;
        Ok(Arc::new(Self {
            snapshot: Arc::downgrade(&snapshot),
            profile,
            commands,
            control,
            #[cfg(test)]
            exited: parking_lot::Mutex::new(exited),
        }))
    }

    pub(super) fn profile(&self) -> Profile {
        self.profile
    }

    pub(super) fn retire(&self) {
        self.control.retired.store(true, Ordering::Release);
        // A full mailbox already wakes the receiver. Never block or join here.
        let _ = self.commands.try_send(Command::Stop);
    }

    fn enqueue(&self, command: Command) -> Result<(), NavigationRefusal> {
        if self.control.retired() || self.control.finished.load(Ordering::Acquire) {
            return Err(NavigationRefusal::WorkerUnavailable);
        }
        self.commands
            .try_send(command)
            .map_err(|error| match error {
                TrySendError::Full(_) => NavigationRefusal::Busy,
                TrySendError::Disconnected(_) => NavigationRefusal::WorkerUnavailable,
            })
    }

    pub(super) async fn definition(
        &self,
        position: Position,
    ) -> Result<Option<Location>, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::Definition(position, reply))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }

    pub(super) async fn references(
        &self,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::References(position, include_declaration, reply))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }

    pub(super) fn belongs_to(&self, snapshot: &Arc<SourceSnapshot>) -> bool {
        Weak::ptr_eq(&self.snapshot, &Arc::downgrade(snapshot))
    }

    #[cfg(test)]
    pub(super) async fn inspect(
        &self,
        position: Position,
    ) -> Result<Inspection, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::Inspect(position, reply))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }

    #[cfg(test)]
    pub(super) async fn drain_barrier(
        &self,
        position: Position,
    ) -> Result<Inspection, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        // Only a test may block after releasing a parked, genuinely live worker.
        self.commands
            .send(Command::Inspect(position, reply))
            .unwrap();
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }

    #[cfg(test)]
    pub(super) fn pause(&self) -> std::sync::mpsc::Sender<()> {
        let (entered, entry) = std::sync::mpsc::channel();
        let (resume, resumed) = std::sync::mpsc::channel();
        self.enqueue(Command::Pause(entered, resumed)).unwrap();
        entry
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        resume
    }

    #[cfg(test)]
    pub(super) fn wait_exit(&self) {
        self.exited
            .lock()
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
    }

    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize) {
        (
            self.control.parses.load(Ordering::Acquire),
            self.control.queries.load(Ordering::Acquire),
        )
    }

    #[cfg(test)]
    pub(super) fn sfc_productions(&self) -> usize {
        self.control.sfc_productions.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(super) fn panic_worker(&self) {
        self.enqueue(Command::Panic).unwrap();
    }
}

impl Drop for NavigationWorker {
    fn drop(&mut self) {
        self.retire();
    }
}

pub(super) fn refused(receiver: Receiver<Command>, control: &Control, refusal: NavigationRefusal) {
    while !control.retired() {
        let Ok(command) = receiver.recv() else { break };
        if control.retired() {
            break;
        }
        match command {
            Command::TemplateDefinition(_, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(refusal.clone()));
                }
            }
            #[cfg(test)]
            Command::SelectedInspect(_, reply) => {
                let _ = reply.send(Err(refusal.clone()));
            }
            Command::Definition(_, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(refusal.clone()));
                }
            }
            Command::References(_, _, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(refusal.clone()));
                }
            }
            Command::Stop => break,
            #[cfg(test)]
            Command::Inspect(_, reply) => {
                let _ = reply.send(Err(refusal.clone()));
            }
            #[cfg(test)]
            Command::Pause(entered, resumed) => {
                let _ = entered.send(());
                let _ = resumed.recv();
            }
            #[cfg(test)]
            Command::Panic => panic!("actual native worker unwind"),
        }
    }
}
