//! The single native diagnostics consumer.

use super::{
    DiagnosticsJobCause, MaestroServer, PendingInitialDiagnostics, refresh_initial_importers,
};
use parking_lot::Mutex;
use std::sync::mpsc;
use std::time::Instant;

pub(super) fn run_worker(
    worker: &MaestroServer,
    receiver: mpsc::Receiver<()>,
    pending: &Mutex<PendingInitialDiagnostics>,
) {
    while receiver.recv().is_ok() {
        loop {
            let preparation = { pending.lock().take_preparation() };
            if let Some((uri, job)) = preparation {
                crate::runtime::block_on(async {
                    let project = worker
                        .for_initial_document(&uri, job.version, job.continuation.as_ref())
                        .await;
                    project
                        .prepare_initial_native_document(&uri, job.version)
                        .await;
                });
                continue;
            }
            let Some(not_before) = pending.lock().next_not_before() else {
                break;
            };
            let delay = not_before.saturating_duration_since(Instant::now());
            if !delay.is_zero() {
                match receiver.recv_timeout(delay) {
                    Ok(()) => continue,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
            let Some((uri, job)) = pending.lock().take_ready(Instant::now()) else {
                continue;
            };
            tracing::info!(
                "starting initial type diagnostics for {} version {}",
                uri,
                job.version
            );
            let started = Instant::now();
            crate::runtime::block_on(async {
                let phase = vize_canon::corsa_bridge::EditorPreparationPhase::start(
                    "initial_type_configuration_admission",
                    1,
                );
                let project = match job.cause {
                    DiagnosticsJobCause::Open => {
                        worker
                            .for_initial_document(&uri, job.version, job.continuation.as_ref())
                            .await
                    }
                    DiagnosticsJobCause::Edit => worker.for_document(&uri).await,
                };
                phase.finish();
                match job.cause {
                    DiagnosticsJobCause::Open => {
                        project
                            .publish_initial_api_diagnostics_if_version(
                                &uri,
                                job.version,
                                job.continuation.as_ref(),
                            )
                            .await;
                    }
                    DiagnosticsJobCause::Edit => {
                        project
                            .publish_changed_diagnostics_if_version(&uri, job.version, job.sync)
                            .await;
                    }
                }
                tracing::info!(
                    "finished initial type diagnostics for {} version {}",
                    uri,
                    job.version
                );
                // Opening an unsaved dependency changes its importers too.
                refresh_initial_importers(&project, &uri, job.version).await;
            });
            pending.lock().last_pass = Some(started.elapsed());
        }
    }
}
