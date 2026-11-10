//! Out-of-band retirement for an editor LSP transport that outlives its caller.

use corsa::runtime::block_on;
use corsa_lsp::LspClient;
use std::sync::Mutex;

type Interrupt = Box<dyn FnOnce() + Send + 'static>;

/// The current editor transport owned by one Corsa bridge.
///
/// A bridge request runs on a synchronous worker. Once its outer deadline has
/// elapsed, the worker cannot borrow the client again until the request
/// returns. Keeping one close handle here lets the deadline owner wake that
/// request and reap its child without moving or concurrently mutating the
/// worker-owned [`EditorLspSession`](super::EditorLspSession).
#[derive(Default)]
pub(crate) struct EditorLspInterrupt(Mutex<Option<Interrupt>>);

impl EditorLspInterrupt {
    pub(super) fn register(&self, client: &LspClient) {
        let client = client.clone();
        self.register_interrupt(Box::new(move || {
            if let Err(error) = block_on(client.close()) {
                tracing::warn!(%error, "cannot interrupt timed-out Corsa editor transport");
            }
        }));
    }

    pub(super) fn clear(&self) {
        self.lock().take();
    }

    /// Close the registered transport exactly once, outside the caller's
    /// executor thread. `LspClient` clones share one concurrency-safe shutdown
    /// coordinator, so the worker's normal retirement may join the same close.
    pub(crate) fn interrupt(&self) -> bool {
        let Some(interrupt) = self.lock().take() else {
            return false;
        };
        if let Err(error) = std::thread::Builder::new()
            .name("vize-corsa-timeout-close".into())
            .spawn(interrupt)
        {
            tracing::warn!(%error, "cannot start timed-out Corsa editor cleanup");
        }
        true
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Interrupt>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn register_interrupt(&self, interrupt: Interrupt) {
        self.lock().replace(interrupt);
    }

    #[cfg(test)]
    pub(crate) fn register_test_interrupt(&self, interrupt: impl FnOnce() + Send + 'static) {
        self.register_interrupt(Box::new(interrupt));
    }
}

#[cfg(test)]
mod tests {
    use super::EditorLspInterrupt;
    use std::{
        sync::{Arc, mpsc},
        time::Duration,
    };

    #[test]
    fn interrupt_is_one_shot_and_replacement_safe() {
        let control = Arc::new(EditorLspInterrupt::default());
        assert!(!control.interrupt());

        let (first_tx, first_rx) = mpsc::channel();
        control.register_test_interrupt(move || {
            let _ = first_tx.send(());
        });
        assert!(control.interrupt());
        assert!(!control.interrupt());
        first_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("first cleanup must run");

        let (second_tx, second_rx) = mpsc::channel();
        control.register_test_interrupt(move || {
            let _ = second_tx.send(());
        });
        assert!(control.interrupt());
        second_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("replacement cleanup must run");
    }
}
