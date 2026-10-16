//! Retired configurations retain exact process custody through close acknowledgment.
use futures::{
    FutureExt, StreamExt,
    channel::{mpsc, oneshot},
    executor::LocalPool,
    future::{BoxFuture, Shared},
    task::LocalSpawnExt,
};
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};
use vize_canon::{CorsaBridge, NativeRetirementOutcome};
use vize_l0::{String, cstr};

type Completion = Shared<BoxFuture<'static, Result<NativeRetirementOutcome, String>>>;
type Job = (
    Arc<CorsaBridge>,
    oneshot::Sender<Result<NativeRetirementOutcome, String>>,
);

#[derive(Clone)]
struct Owner {
    bridge: Arc<CorsaBridge>,
    completion: Completion,
}

#[derive(Default)]
struct State {
    owners: Vec<Owner>,
}

/// Root and package contexts share this ledger, including contexts already removed from caches.
#[derive(Clone, Default)]
pub(in crate::server::state) struct RetiredOwners(Arc<Mutex<State>>);

impl RetiredOwners {
    /// The caller still holds the cached-owner slot. Never close IPC on this path.
    pub(in crate::server::state) fn retain_and_close(&self, bridge: Arc<CorsaBridge>) {
        let mut state = self.0.lock();
        observe_completed(&mut state);
        if state
            .owners
            .iter()
            .any(|owner| Arc::ptr_eq(&owner.bridge, &bridge))
        {
            return;
        }
        let (reply, response) = oneshot::channel();
        let completion = async move {
            response.await.unwrap_or_else(|error| {
                Err(cstr!(
                    "Retired native close worker did not acknowledge completion: {error}"
                ))
            })
        }
        .boxed()
        .shared();
        state.owners.push(Owner {
            bridge: bridge.clone(),
            completion,
        });
        // A failed enqueue still leaves the exact owner and a failed completion
        // in the ledger. Final exit retries cleanup without erasing this error.
        match closer() {
            Ok(sender) => {
                if let Err(error) = sender.unbounded_send((bridge, reply)) {
                    let (_, reply) = error.into_inner();
                    let _ = reply.send(Err(cstr!("Retired native close executor stopped")));
                }
            }
            Err(error) => {
                let _ = reply.send(Err(error.clone()));
            }
        }
    }

    pub(in crate::server::state) async fn close_for_exit(&self) -> Result<(), String> {
        let owners = {
            let mut state = self.0.lock();
            observe_completed(&mut state);
            state.owners.clone()
        };
        let outcomes = futures::future::join_all(owners.iter().map(|owner| async {
            let prior = owner.completion.clone().await;
            if prior.as_ref().is_ok_and(|outcome| outcome.custody_complete) {
                return Ok(());
            }
            // Ordinary request/close failures remain visible. A retry may
            // establish exact terminal custody later, without relabeling the
            // old protocol result as a graceful success.
            tracing::warn!(prior = ?prior, "retrying incomplete retired native custody");
            let retry = owner
                .bridge
                .shutdown_for_retirement()
                .await
                .map_err(|error| {
                    cstr!(
                        "Retired native cleanup retry failed: {error}; previous attempt: {prior:?}"
                    )
                })?;
            publish(&retry);
            if retry.custody_complete {
                Ok(())
            } else {
                Err(cstr!(
                    "Retired native owner lacks terminal custody: {:?}",
                    retry.receipt
                ))
            }
        }))
        .await;
        let mut state = self.0.lock();
        let mut failures = Vec::new();
        for (owner, outcome) in owners.iter().zip(outcomes) {
            match outcome {
                Ok(()) => state
                    .owners
                    .retain(|entry| !Arc::ptr_eq(&entry.bridge, &owner.bridge)),
                Err(error) => failures.push(error),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(cstr!(
                "Retired native ownership is incomplete: {failures:?}"
            ))
        }
    }
}

fn observe_completed(state: &mut State) {
    state
        .owners
        .retain(|owner| !matches!(owner.completion.clone().now_or_never(), Some(Ok(outcome)) if outcome.custody_complete));
}

fn publish(outcome: &NativeRetirementOutcome) {
    tracing::info!(receipt = %outcome.receipt, "retired native owner terminal custody");
}

/// One nonblocking executor serves all retired epochs. Configuration discovery
/// keeps its own workers, and an abandoned RPC cannot block another owner close.
fn closer() -> Result<&'static mpsc::UnboundedSender<Job>, &'static String> {
    static CLOSER: OnceLock<Result<mpsc::UnboundedSender<Job>, String>> = OnceLock::new();
    CLOSER
        .get_or_init(|| {
            let (sender, mut jobs) = mpsc::unbounded::<Job>();
            std::thread::Builder::new()
                .name("vize-retired-native-close".into())
                .spawn(move || {
                    let mut pool = LocalPool::new();
                    let spawner = pool.spawner();
                    pool.run_until(async move {
                        while let Some((bridge, reply)) = jobs.next().await {
                            if let Err(error) = spawner.spawn_local(async move {
                                let outcome =
                                    bridge.shutdown_for_retirement().await.map_err(|error| {
                                        cstr!("Retired native owner close failed: {error}")
                                    });
                                match &outcome {
                                    Ok(outcome) => publish(outcome),
                                    Err(error) => tracing::warn!(
                                        "retired native close attempt failed: {error}"
                                    ),
                                }
                                let _ = reply.send(outcome);
                            }) {
                                tracing::error!("cannot schedule retired native close: {error}");
                            }
                        }
                    });
                    pool.run();
                })
                .map(|_| sender)
                .map_err(|error| cstr!("Cannot start retired native close executor: {error}"))
        })
        .as_ref()
}
