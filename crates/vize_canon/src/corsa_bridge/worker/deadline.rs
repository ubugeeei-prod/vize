//! One shared deadline thread; replies come directly from the backend worker.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{OnceLock, mpsc};
use std::time::Instant;

use futures::channel::oneshot;

use super::WorkerError;

struct Deadline {
    at: Instant,
    wake: oneshot::Sender<()>,
}

impl PartialEq for Deadline {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}
impl Eq for Deadline {}
impl PartialOrd for Deadline {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Deadline {
    fn cmp(&self, other: &Self) -> Ordering {
        other.at.cmp(&self.at)
    }
}

pub(super) fn wake_at(at: Instant) -> Result<oneshot::Receiver<()>, WorkerError> {
    static SENDER: OnceLock<Option<mpsc::Sender<Deadline>>> = OnceLock::new();
    let sender = SENDER.get_or_init(|| {
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("vize-corsa-deadlines".into())
            .spawn(move || run(receiver))
            .ok()
            .map(|_| sender)
    });
    let (wake, receiver) = oneshot::channel();
    sender
        .as_ref()
        .ok_or(WorkerError::Stopped)?
        .send(Deadline { at, wake })
        .map_err(|_| WorkerError::Stopped)?;
    Ok(receiver)
}

fn run(receiver: mpsc::Receiver<Deadline>) {
    let mut deadlines = BinaryHeap::<Deadline>::new();
    loop {
        while deadlines
            .peek()
            .is_some_and(|deadline| deadline.at <= Instant::now() || deadline.wake.is_canceled())
        {
            if let Some(deadline) = deadlines.pop() {
                let _ = deadline.wake.send(());
            }
        }
        let received = match deadlines.peek() {
            Some(next) => receiver.recv_timeout(next.at.saturating_duration_since(Instant::now())),
            None => receiver
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };
        match received {
            Ok(deadline) => deadlines.push(deadline),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}
