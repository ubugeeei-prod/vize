//! The background thread behind the session cache's disk sweep.
#![expect(
    clippy::disallowed_types,
    reason = "the sweep thread observes the shared editor session weakly"
)]

use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Weak};
use std::time::Instant;

use super::EditorMirrorSession;
use super::vue_dependencies_alias::recover_lock;

impl EditorMirrorSession {
    /// Validate captured disk inputs in the background for an owner whose
    /// client cannot watch files. Idempotent; the thread ends with the session.
    pub(crate) fn sweep_unwatched_disk_inputs(self: &Arc<Self>) {
        let mut sweeper = recover_lock(&self.sweeper);
        if sweeper.is_some() {
            return;
        }
        let (sender, receiver) = sync_channel::<()>(1);
        let session = Arc::downgrade(self);
        let spawned = std::thread::Builder::new()
            .name("vize-disk-sweep".into())
            .spawn(move || run(&session, &receiver));
        if spawned.is_ok() {
            *sweeper = Some(sender);
            drop(sweeper);
            self.cache().enable_disk_sweep();
        }
    }

    /// Ask for the next sweep; a running or recent one absorbs the request.
    pub(in crate::corsa_bridge) fn request_disk_sweep(&self) {
        if let Some(sender) = recover_lock(&self.sweeper).as_ref() {
            // A full channel already holds a pending sweep.
            let _ = sender.try_send(());
        }
    }
}

fn run(session: &Weak<EditorMirrorSession>, receiver: &std::sync::mpsc::Receiver<()>) {
    while receiver.recv().is_ok() {
        let Some(session) = session.upgrade() else {
            return;
        };
        let Some(inputs) = session.cache().begin_disk_sweep(Instant::now()) else {
            continue;
        };
        let phase = super::preparation_trace::Phase::start("disk_sweep", 1);
        let unchanged = inputs.unchanged();
        phase.finish();
        session
            .cache()
            .finish_disk_sweep(&inputs, unchanged, Instant::now());
    }
}
