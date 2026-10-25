//! Share one foreground-owned completion without retaining a state self-cycle.
#![expect(
    clippy::disallowed_types,
    reason = "foreground request views share one graceful shutdown completion"
)]

use std::{future::Future, io, panic::AssertUnwindSafe, sync::Arc};

use futures::{FutureExt, channel::oneshot, future::BoxFuture, future::Shared};
use parking_lot::Mutex;
use tower_lsp::jsonrpc::Result;

type Completion = Shared<BoxFuture<'static, Result<()>>>;
type Work = BoxFuture<'static, Result<()>>;
type Reply = oneshot::Sender<Result<()>>;

/// The server owns this coordinator; ServerState never points back to it.
/// Only the first shutdown starts one named driver, never server startup or
/// diagnostic workers. Its work continues if every request waiter cancels.
#[derive(Clone, Default)]
pub(super) struct Shutdown(Arc<Mutex<Option<Completion>>>);

impl Shutdown {
    pub(super) async fn run(
        &self,
        work: impl Future<Output = Result<()>> + Send + 'static,
    ) -> Result<()> {
        self.run_with(work, launch).await
    }

    async fn run_with(
        &self,
        work: impl Future<Output = Result<()>> + Send + 'static,
        start: impl FnOnce(Work, Reply) -> io::Result<()>,
    ) -> Result<()> {
        let completion = {
            let mut pending = self.0.lock();
            pending
                .get_or_insert_with(|| {
                    let (reply, response) = oneshot::channel();
                    match start(work.boxed(), reply) {
                        Ok(()) => response
                            .map(|result| {
                                result.unwrap_or_else(|_| {
                                    Err(tower_lsp::jsonrpc::Error::internal_error())
                                })
                            })
                            .boxed()
                            .shared(),
                        Err(error) => {
                            tracing::error!("failed to start owned shutdown: {error}");
                            futures::future::ready(Err(tower_lsp::jsonrpc::Error::internal_error()))
                                .boxed()
                                .shared()
                        }
                    }
                })
                .clone()
        };
        completion.await
    }
}

fn launch(work: Work, reply: Reply) -> io::Result<()> {
    std::thread::Builder::new()
        .name("vize-owned-shutdown".into())
        .spawn(move || {
            let outcome =
                std::panic::catch_unwind(AssertUnwindSafe(|| crate::runtime::block_on(work)))
                    .unwrap_or_else(|_| {
                        tracing::error!("owned shutdown driver panicked");
                        Err(tower_lsp::jsonrpc::Error::internal_error())
                    });
            let _ = reply.send(outcome);
        })
        .map(|_| ())
}

#[cfg(test)]
mod tests;
