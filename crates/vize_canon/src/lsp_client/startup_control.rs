//! Final-exit admission and exact owned handles for unfinished native startup.
#![expect(
    clippy::disallowed_types,
    reason = "typed SDK clients share owned process custody"
)]

use corsa::{
    api::{ApiClient, ApiSpawnConfig},
    process::{ProcessLifetimeObserver, ProcessLifetimeSnapshot, ProcessTerminalObserver},
    runtime::block_on,
};
use corsa_lsp::{LspClient, LspSpawnConfig, process::ProcessObserver};
use serde_json::{Value, json};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use vize_l0::{String, cstr};

#[derive(Clone, Default)]
pub(crate) struct StartupControl {
    state: Arc<Mutex<State>>,
    changed: Arc<Condvar>,
    lifetime: ProcessLifetimeObserver,
}

#[derive(Default)]
struct State {
    sealed: bool,
    next: u64,
    pending: Vec<(u64, Handle)>,
    first_failure: Option<String>,
    in_progress: usize,
    spawning: usize,
    aborting: bool,
}

#[derive(Clone)]
enum Handle {
    Api(ApiClient),
    Editor(LspClient, ProcessObserver),
}

pub(crate) struct Registration {
    control: StartupControl,
    id: u64,
}

/// Counts admitted constructors without holding a lock during SDK setup/Drop.
struct SpawnAdmission {
    control: StartupControl,
    active: bool,
}

struct AbortAdmission(StartupControl);

impl Drop for AbortAdmission {
    fn drop(&mut self) {
        self.0.lock().aborting = false;
        self.0.changed.notify_all();
    }
}

impl SpawnAdmission {
    fn publish(mut self, handle: Handle) -> Result<Registration, String> {
        let mut state = self.control.lock();
        let registration = self.control.register(&mut state, handle);
        state.spawning -= 1;
        self.active = false;
        let sealed = state.sealed;
        drop(state);
        self.control.changed.notify_all();
        if sealed {
            // The sealed registration retains its exact typed owner for the
            // abort lane. No initialize request or fallback can start now.
            Err(cstr!("Native startup refused after final server close"))
        } else {
            Ok(registration)
        }
    }
}

impl Drop for SpawnAdmission {
    fn drop(&mut self) {
        if self.active {
            self.control.lock().spawning -= 1;
            self.control.changed.notify_all();
        }
    }
}

pub(super) fn spawn_api(
    control: Option<&StartupControl>,
    config: ApiSpawnConfig,
) -> Result<(ApiClient, Option<Registration>), String> {
    match control {
        Some(control) => {
            let (client, registration) = control.spawn_api(config)?;
            Ok((client, Some(registration)))
        }
        None => block_on(ApiClient::spawn(config))
            .map(|client| (client, None))
            .map_err(|error| cstr!("Cannot spawn native API: {error}")),
    }
}

impl StartupControl {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn check_admission(&self) -> Result<(), String> {
        if self.lock().sealed {
            Err(cstr!("Native startup refused after final server close"))
        } else {
            Ok(())
        }
    }

    fn admit_spawn(&self) -> Result<SpawnAdmission, String> {
        self.publish_completed(None);
        let mut state = self.lock();
        if state.sealed {
            return Err(cstr!("Native startup refused after final server close"));
        }
        state.spawning += 1;
        Ok(SpawnAdmission {
            control: self.clone(),
            active: true,
        })
    }

    pub(crate) fn spawn_api(
        &self,
        mut config: ApiSpawnConfig,
    ) -> Result<(ApiClient, Registration), String> {
        let admission = self.admit_spawn()?;
        // The admitted lease, rather than a held State mutex, fences the whole
        // transport-only spawn. Fallible setup may run real child RAII cleanup.
        config.command = config
            .command
            .with_lifetime_observer(self.lifetime.clone(), "api");
        let client = block_on(ApiClient::spawn(config))
            .map_err(|error| cstr!("Cannot spawn native API: {error}"))?;
        let registration = admission.publish(Handle::Api(client.clone()))?;
        Ok((client, registration))
    }

    pub(crate) fn spawn_editor(
        &self,
        config: LspSpawnConfig,
        observer: ProcessObserver,
    ) -> Result<(LspClient, Registration), String> {
        self.spawn_editor_with(config, observer, |config| {
            block_on(LspClient::spawn(config))
        })
    }

    fn spawn_editor_with(
        &self,
        mut config: LspSpawnConfig,
        observer: ProcessObserver,
        spawn: impl FnOnce(LspSpawnConfig) -> corsa::Result<LspClient>,
    ) -> Result<(LspClient, Registration), String> {
        let admission = self.admit_spawn()?;
        config.command = config
            .command
            .with_lifetime_observer(self.lifetime.clone(), "editor-lsp");
        let client =
            spawn(config).map_err(|error| cstr!("Cannot spawn native editor LSP: {error}"))?;
        let registration = admission.publish(Handle::Editor(client.clone(), observer))?;
        Ok((client, registration))
    }

    fn register(&self, state: &mut State, handle: Handle) -> Registration {
        let id = state.next;
        state.next += 1;
        state.pending.push((id, handle));
        Registration {
            control: self.clone(),
            id,
        }
    }

    pub(crate) async fn abort_pending(&self, deadline: std::time::Instant) -> Result<(), String> {
        {
            let mut state = self.lock();
            state.sealed = true;
            if state.pending.is_empty()
                && state.in_progress == 0
                && state.spawning == 0
                && !state.aborting
            {
                return state.first_failure.clone().map_or(Ok(()), Err);
            }
        }
        let timer = crate::corsa_bridge::worker::deadline::wake_at(deadline)
            .map_err(|error| cstr!("Cannot bound native startup abort: {error:?}"))?;
        let (reply, result) = futures::channel::oneshot::channel();
        let control = self.clone();
        self.lock().in_progress += 1;
        if let Err(error) = std::thread::Builder::new()
            .name("vize-native-startup-close".into())
            .spawn(move || {
                let result = control.abort_pending_sync(deadline);
                control.lock().in_progress -= 1;
                let _ = reply.send(result);
            })
        {
            let mut state = self.lock();
            state.in_progress -= 1;
            return Err(state
                .first_failure
                .get_or_insert_with(|| cstr!("Cannot run native startup abort: {error}"))
                .clone());
        }
        match futures::future::select(result, timer).await {
            futures::future::Either::Left((Ok(result), _)) => result,
            futures::future::Either::Left((Err(error), _)) => self.remember_failure(cstr!("Native startup abort worker did not acknowledge completion: {error}")),
            futures::future::Either::Right(_) => self.remember_failure(cstr!("Native startup abort outran the existing final-exit deadline; exact owner remains pending")),
        }
    }

    fn remember_failure(&self, error: String) -> Result<(), String> {
        let mut state = self.lock();
        Err(state.first_failure.get_or_insert(error).clone())
    }

    fn abort_pending_sync(&self, deadline: std::time::Instant) -> Result<(), String> {
        let pending = {
            let mut state = self.lock();
            state.sealed = true;
            while state.spawning != 0 || state.aborting {
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err(state.first_failure.get_or_insert_with(|| cstr!("Native admitted startup did not hand off exact custody before the existing final-exit deadline")).clone());
                }
                let (next, _) = self
                    .changed
                    .wait_timeout(state, remaining)
                    .unwrap_or_else(|error| error.into_inner());
                state = next;
            }
            state.aborting = true;
            std::mem::take(&mut state.pending)
        };
        let _admission = AbortAdmission(self.clone());
        for (id, handle) in pending {
            let (result, role, stderr) = match &handle {
                Handle::Api(client) => (
                    block_on(client.abort_startup()),
                    "api",
                    json!({"custody":"inherited-server-stderr"}),
                ),
                Handle::Editor(client, observer) => {
                    let result = client.abort_startup();
                    let receipts = observer.receipts();
                    let stderr = json!({"custody":"observed-editor", "wholeReceipts":cstr!("{receipts:?}"), "bytes":receipts.iter().map(|receipt| &receipt.stderr).collect::<Vec<_>>()});
                    (result, "editor-lsp", stderr)
                }
            };
            let receipt = match &result {
                Ok(Some(receipt)) => {
                    json!({"pid":receipt.pid,"status":cstr!("{:?}",receipt.status),"exitCode":receipt.status.code(),"timedOut":receipt.timed_out,"killRequested":receipt.kill_requested,"reaped":receipt.reaped})
                }
                Ok(None) => Value::Null,
                Err(error) => json!({"error":cstr!("{error}")}),
            };
            let evidence = json!({"controlledStartupAbort":true,"registration":id,"role":role,"terminal":receipt,"stderr":stderr});
            tracing::info!(receipt = %evidence, "controlled native startup termination");
            let mut failure = match &result {
                Ok(Some(receipt))
                    if receipt.reaped
                        && !receipt.timed_out
                        && (receipt.kill_requested || receipt.status.success()) =>
                {
                    None
                }
                _ => Some(cstr!(
                    "Native startup abort did not acknowledge owned process reaping: {evidence}"
                )),
            };
            if let Handle::Editor(_, observer) = &handle {
                let receipts = observer.receipts();
                if receipts.len() != 1
                    || receipts.iter().any(|receipt| {
                        !receipt.completed
                            || !receipt.reaped
                            || !receipt.stderr_complete
                            || receipt.stderr_error.is_some()
                            || receipt.process_error.is_some()
                            || receipt.timed_out
                            || !receipt.stderr.is_empty()
                    })
                {
                    failure = Some(cstr!(
                        "Controlled editor startup abort retained incomplete or unexpected native evidence: {evidence}"
                    ));
                }
            }
            let mut state = self.lock();
            if let Some(error) = failure {
                state.pending.push((id, handle));
                if state.first_failure.is_none() {
                    state.first_failure = Some(error);
                }
            }
        }
        let failure = self.lock().first_failure.clone();
        failure.map_or(Ok(()), Err)
    }

    pub(crate) fn startup_custody_pending(&self) -> bool {
        let state = self.lock();
        state.spawning != 0 || state.aborting || state.in_progress != 0 || !state.pending.is_empty()
    }

    /// Ordinary recovery streams complete custody to the existing diagnostic
    /// sink before pruning it. As with ordinary diagnostics, disabled logging
    /// offers no durable audit guarantee. Sealed owners keep their rows for
    /// the final typed retirement receipt.
    pub(crate) fn publish_completed(&self, retirement_error: Option<&str>) {
        if self.lock().sealed {
            return;
        }
        let snapshot = self.lifetime.snapshot();
        if snapshot.completed.is_empty() {
            return;
        }
        let children = snapshot
            .pending
            .iter()
            .filter(|owner| owner.custody_complete())
            .map(super::retirement::observation_json)
            .collect::<Vec<_>>();
        let receipt = json!({"children":children,"retirementError":retirement_error,"observedCount":snapshot.observed,"publishedCount":snapshot.published});
        tracing::info!(receipt = %receipt, "recovered native owner terminal custody");
        self.lifetime.acknowledge_publication(&snapshot.completed);
    }

    pub(crate) fn lifetime_snapshot(&self) -> ProcessLifetimeSnapshot {
        self.lifetime.snapshot()
    }
    pub(crate) fn unpublished_observers(&self) -> Vec<ProcessTerminalObserver> {
        self.lifetime.unpublished()
    }

    pub(crate) fn acknowledge_publication(&self, observers: &[ProcessTerminalObserver]) {
        self.lifetime.acknowledge_publication(observers);
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        let mut state = self.control.lock();
        // Failed/pending finalization retains owners for retry. No late
        // constructor completion can erase an unacknowledged abort.
        let removed = if !state.sealed {
            state
                .pending
                .iter()
                .position(|(id, _)| *id == self.id)
                .map(|position| state.pending.remove(position))
        } else {
            None
        };
        // The last SDK clone may run cleanup. Never run it under the admission
        // mutex that the async finalizer needs in order to seal startup.
        drop(state);
        drop(removed);
    }
}

#[cfg(all(test, unix))]
mod tests;
