//! Reuse the existing blocking navigation boundary across requests.

use std::sync::Arc;

use futures::{
    SinkExt, StreamExt,
    channel::{mpsc, oneshot},
    lock::Mutex,
};

type Work = Box<dyn FnOnce() + Send + 'static>;

struct Connection {
    sender: mpsc::Sender<Work>,
    identity: Arc<()>,
}

/// One lazy worker per inventory. The channel has one sender and no extra
/// buffer slots, so it retains at most one waiting command beside the running
/// command. Backpressure yields the LSP executor instead of spawning threads.
#[derive(Default)]
pub(super) struct Worker {
    sender: Mutex<Option<Connection>>,
}

impl Worker {
    pub(super) async fn run<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        let mut sender = self.sender.lock().await;
        if sender
            .as_ref()
            .is_none_or(|connection| connection.sender.is_closed())
        {
            let (send, mut receive) = mpsc::channel::<Work>(0);
            if let Err(error) = std::thread::Builder::new()
                .name("vize-project-navigation".into())
                .spawn(move || {
                    crate::runtime::block_on(async {
                        while let Some(work) = receive.next().await {
                            work();
                        }
                    });
                })
            {
                tracing::warn!("failed to spawn project navigation discovery: {error}");
                return None;
            }
            *sender = Some(Connection {
                sender: send,
                identity: Arc::new(()),
            });
        }
        let connection = sender.as_mut()?;
        let identity = Arc::clone(&connection.identity);
        let (result, receive) = oneshot::channel();
        let command: Work = Box::new(move || {
            let _ = result.send(work());
        });
        let sent = connection.sender.send(command).await;
        drop(sender);
        sent.ok()?;
        match receive.await {
            Ok(result) => Some(result),
            Err(_) => {
                // A panic can drop the result sender before the native
                // receiver finishes unwinding. Retire that exact connection
                // now; a late failed reply must not retire a newer worker.
                let mut sender = self.sender.lock().await;
                if sender
                    .as_ref()
                    .is_some_and(|connection| Arc::ptr_eq(&connection.identity, &identity))
                {
                    *sender = None;
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests;
