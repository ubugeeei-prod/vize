//! Clock and identity laws; these controls do not simulate a native RPC success.

use super::*;
use crate::{ide::IdeContext, runtime::block_on};
use std::{sync::atomic::Ordering, time::Duration};

fn fixture() -> (
    ServerState,
    Url,
    InitialDiagnosticsScheduler,
    std::sync::mpsc::Receiver<()>,
) {
    let state = ServerState::new();
    let uri = Url::parse("file:///foreground/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), "<template />".into(), 1, "vue".into());
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let scheduler = InitialDiagnosticsScheduler {
        sender: Some(sender),
        sync_sender: None,
        worker_owned: false,
        pending: parking_lot::Mutex::new(Default::default()).into(),
    };
    (state, uri, scheduler, receiver)
}

fn insert(scheduler: &InitialDiagnosticsScheduler, uri: &Url, now: Instant) {
    scheduler
        .pending
        .lock()
        .insert(uri.clone(), 1, now + Duration::from_secs(1));
}

fn finish_preparation(scheduler: &InitialDiagnosticsScheduler, uri: &Url) {
    let mut pending = scheduler.pending.lock();
    let (_, sync) = pending.take_sync().unwrap();
    pending.complete_sync(uri, sync.sequence);
    assert_eq!(pending.take_preparation().unwrap().0, *uri);
}

fn record(ticket: &ForegroundOpen, state: &ServerState, uri: &Url) {
    let ctx = IdeContext::new(state, uri, 0)
        .unwrap()
        .with_foreground_preparation(Some(ticket.preparation()));
    // Production calls this only after the strict same-document open succeeded.
    ctx.record_foreground_preparation();
}

fn advance(
    scheduler: &InitialDiagnosticsScheduler,
    ticket: &ForegroundOpen,
    stamp: CorsaRequestStamp,
    state: &ServerState,
    now: Instant,
) -> bool {
    let prepared = ticket.prepared.swap(false, Ordering::AcqRel);
    scheduler.foreground_ready(ticket, stamp, prepared, state, now)
}

#[test]
fn exact_completed_open_advances_once_and_wakes_after_the_scope_is_released() {
    let (state, uri, scheduler, receiver) = fixture();
    let now = Instant::now();
    insert(&scheduler, &uri, now);
    let ticket = scheduler.foreground_open(&uri, &state).unwrap();
    finish_preparation(&scheduler, &uri);
    record(&ticket, &state, &uri);
    let stamp = block_on(async {
        let scope = state.corsa_request_scope().await;
        let stamp = scope.stamp();
        assert!(state.try_corsa_request_scope().is_none());
        drop(scope);
        stamp
    });
    let released = state.try_corsa_request_scope().unwrap();
    drop(released);
    assert!(advance(&scheduler, &ticket, stamp, &state, now));
    assert!(receiver.try_recv().is_ok());
    assert!(!advance(&scheduler, &ticket, stamp, &state, now));
    let mut pending = scheduler.pending.lock();
    pending.jobs.verify();
    let (actual_uri, job) = pending.take_ready(now).unwrap();
    assert_eq!(
        (actual_uri, job.version, job.sequence),
        (uri, 1, ticket.sequence)
    );
    assert_eq!(job.not_before, now);
    assert_eq!(job.cause, DiagnosticsJobCause::Open);
    assert!(pending.jobs.is_empty());
}

#[test]
fn absent_or_foreign_context_marker_cannot_certify_preparation() {
    for foreign in [false, true] {
        let (state, uri, scheduler, receiver) = fixture();
        let now = Instant::now();
        insert(&scheduler, &uri, now);
        let ticket = scheduler.foreground_open(&uri, &state).unwrap();
        finish_preparation(&scheduler, &uri);
        let other = Url::parse("file:///foreground/Imported.vue").unwrap();
        state
            .documents
            .open(other.clone(), "<template />".into(), 1, "vue".into());
        if foreign {
            record(&ticket, &state, &other);
        }
        assert!(!ticket.prepared.load(Ordering::Acquire));
        assert!(!advance(
            &scheduler,
            &ticket,
            state.corsa_request_stamp(),
            &state,
            now
        ));
        assert!(receiver.try_recv().is_err());
        assert_eq!(
            scheduler.pending.lock().jobs[&uri].not_before,
            now + Duration::from_secs(1)
        );
    }
}

#[test]
fn pending_sync_running_sync_preparation_and_elapsed_grace_stay_protected() {
    for phase in ["queued", "sync", "preparation", "expired"] {
        let (state, uri, scheduler, receiver) = fixture();
        let now = Instant::now();
        insert(&scheduler, &uri, now);
        let ticket = scheduler.foreground_open(&uri, &state).unwrap();
        if phase != "queued" {
            let mut pending = scheduler.pending.lock();
            let (_, sync) = pending.take_sync().unwrap();
            if phase != "sync" {
                pending.complete_sync(&uri, sync.sequence);
            }
            if phase == "expired" {
                pending.take_preparation().unwrap();
            }
        }
        record(&ticket, &state, &uri);
        let at = if phase == "expired" {
            now + Duration::from_secs(1)
        } else {
            now
        };
        assert!(
            !advance(&scheduler, &ticket, state.corsa_request_stamp(), &state, at),
            "{phase}"
        );
        assert!(receiver.try_recv().is_err());
        let pending = scheduler.pending.lock();
        assert_eq!(pending.jobs[&uri].not_before, now + Duration::from_secs(1));
        pending.jobs.verify();
    }
}

#[test]
fn old_ticket_never_advances_missing_edit_retry_or_reopened_sequence() {
    for cause in ["missing", "edit", "retry", "reopen"] {
        let (state, uri, scheduler, receiver) = fixture();
        let now = Instant::now();
        insert(&scheduler, &uri, now);
        let ticket = scheduler.foreground_open(&uri, &state).unwrap();
        finish_preparation(&scheduler, &uri);
        record(&ticket, &state, &uri);
        let mut pending = scheduler.pending.lock();
        pending.jobs.remove(&uri);
        match cause {
            "edit" => pending.insert_with_continuation(
                uri.clone(),
                1,
                now + Duration::from_millis(150),
                DiagnosticsJobCause::Edit,
                None,
                None,
            ),
            "retry" => pending.insert_retry(uri.clone(), 1, now + Duration::from_secs(1)),
            "reopen" => pending.insert(uri.clone(), 1, now + Duration::from_secs(1)),
            _ => {}
        }
        let expected = pending
            .jobs
            .get(&uri)
            .map(|job| (job.cause, job.sequence, job.not_before));
        drop(pending);
        assert!(
            !advance(
                &scheduler,
                &ticket,
                state.corsa_request_stamp(),
                &state,
                now
            ),
            "{cause}"
        );
        assert!(receiver.try_recv().is_err());
        let pending = scheduler.pending.lock();
        assert_eq!(
            pending
                .jobs
                .get(&uri)
                .map(|job| (job.cause, job.sequence, job.not_before)),
            expected
        );
        pending.jobs.verify();
    }
}

#[test]
fn the_original_transaction_stamp_refuses_changed_closed_reopened_and_retired_worlds() {
    for change in ["edit", "close", "reopen", "environment", "retired"] {
        let (state, uri, scheduler, receiver) = fixture();
        let now = Instant::now();
        insert(&scheduler, &uri, now);
        let ticket = scheduler.foreground_open(&uri, &state).unwrap();
        finish_preparation(&scheduler, &uri);
        record(&ticket, &state, &uri);
        let stamp = block_on(async { state.corsa_request_scope().await.stamp() });
        match change {
            "edit" => {
                state.documents.apply_changes(
                    &uri,
                    vec![tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: "<template>new</template>".into(),
                    }],
                    2,
                );
            }
            "close" => state.documents.close(&uri),
            "reopen" => {
                state.documents.close(&uri);
                state.documents.open(
                    uri.clone(),
                    "<template>new</template>".into(),
                    1,
                    "vue".into(),
                );
            }
            "environment" => state.set_workspace_root("/new-owner".into()),
            "retired" => state.retire_project_contexts(),
            _ => {}
        }
        assert!(!stamp.is_current(&state), "{change}");
        assert!(
            !advance(&scheduler, &ticket, stamp, &state, now),
            "{change}"
        );
        assert!(receiver.try_recv().is_err());
        assert_eq!(
            scheduler.pending.lock().jobs[&uri].not_before,
            now + Duration::from_secs(1)
        );
    }
}

#[test]
fn only_an_existing_open_version_with_a_live_worker_produces_a_ticket() {
    let (state, uri, mut scheduler, _receiver) = fixture();
    let now = Instant::now();
    assert!(scheduler.foreground_open(&uri, &state).is_none());
    insert(&scheduler, &uri, now);
    assert!(scheduler.foreground_open(&uri, &state).is_some());
    state.documents.close(&uri);
    assert!(scheduler.foreground_open(&uri, &state).is_none());
    state
        .documents
        .open(uri.clone(), "<template />".into(), 2, "vue".into());
    assert!(scheduler.foreground_open(&uri, &state).is_none());
    scheduler
        .pending
        .lock()
        .insert_retry(uri.clone(), 2, now + Duration::from_secs(1));
    assert!(scheduler.foreground_open(&uri, &state).is_none());
    scheduler
        .pending
        .lock()
        .insert(uri.clone(), 2, now + Duration::from_secs(1));
    scheduler.sender = None;
    assert!(scheduler.foreground_open(&uri, &state).is_none());
}

#[test]
fn document_dispatch_requires_success_marker_and_the_original_scope_then_releases_it() {
    for control in [
        "prepared",
        "unmarked",
        "prior-error",
        "error",
        "edit",
        "environment",
    ] {
        let (state, uri, scheduler, receiver) = fixture();
        state.set_workspace_root("/foreground".into());
        let (service, _socket) = tower_lsp::LspService::new(crate::server::MaestroServer::new);
        let mut server = crate::server::MaestroServer::diagnostic_worker(
            service.inner().client.clone(),
            state.into(),
        );
        let state = server.state.clone();
        server.initial_diagnostics = Some(scheduler.clone());
        let now = Instant::now();
        insert(&scheduler, &uri, now);
        finish_preparation(&scheduler, &uri);
        let response = block_on(async {
            let view = server.for_document_admitted(&uri).await;
            let preparation = view.foreground_preparation().unwrap();
            let ctx = IdeContext::new(&state, &uri, 0)
                .unwrap()
                .with_foreground_preparation(Some(preparation));
            if control == "prior-error" {
                let previous: tower_lsp::jsonrpc::Result<()> = view
                    .native_request(async {
                        ctx.record_foreground_preparation();
                        Err(tower_lsp::jsonrpc::Error::internal_error())
                    })
                    .await;
                assert!(previous.is_err());
                assert!(receiver.try_recv().is_err());
            }
            view.native_request(async {
                assert!(state.try_corsa_request_scope().is_none());
                if !matches!(control, "unmarked" | "prior-error") {
                    ctx.record_foreground_preparation();
                }
                match control {
                    "error" => return Err(tower_lsp::jsonrpc::Error::internal_error()),
                    "edit" => {
                        state.documents.apply_changes(
                            &uri,
                            vec![tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                                range: None,
                                range_length: None,
                                text: "<template>changed</template>".into(),
                            }],
                            2,
                        );
                    }
                    "environment" => state.set_workspace_root("/replacement".into()),
                    _ => {}
                }
                Ok(serde_json::json!({"response":"whole original value"}))
            })
            .await
        });
        assert!(state.try_corsa_request_scope().is_some());
        if matches!(control, "prepared" | "unmarked" | "prior-error") {
            assert_eq!(
                response.unwrap(),
                serde_json::json!({"response":"whole original value"})
            );
        } else {
            assert!(response.is_err(), "{control}");
        }
        let mut pending = scheduler.pending.lock();
        pending.jobs.verify();
        if control == "prepared" {
            assert!(receiver.try_recv().is_ok());
            let (_, job) = pending.take_ready(Instant::now()).unwrap();
            assert!(job.not_before < now + Duration::from_secs(1));
        } else {
            assert!(receiver.try_recv().is_err(), "{control}");
            assert_eq!(
                pending.jobs[&uri].not_before,
                now + Duration::from_secs(1),
                "{control}"
            );
        }
    }

    // A second request on the SAME view waits for scope without clearing or
    // inheriting the first request's preparation marker.
    use std::future::Future;
    let (state, uri, scheduler, receiver) = fixture();
    state.set_workspace_root("/foreground".into());
    let (service, _socket) = tower_lsp::LspService::new(crate::server::MaestroServer::new);
    let mut server = crate::server::MaestroServer::diagnostic_worker(
        service.inner().client.clone(),
        state.into(),
    );
    let state = server.state.clone();
    server.initial_diagnostics = Some(scheduler.clone());
    let now = Instant::now();
    insert(&scheduler, &uri, now);
    finish_preparation(&scheduler, &uri);
    block_on(async {
        let view = server.for_document_admitted(&uri).await;
        let preparation = view.foreground_preparation().unwrap();
        let ctx = IdeContext::new(&state, &uri, 0)
            .unwrap()
            .with_foreground_preparation(Some(preparation));
        let (release, blocked) = futures::channel::oneshot::channel();
        let mut first = Box::pin(view.native_request(async {
            ctx.record_foreground_preparation();
            blocked.await.unwrap();
            Ok("prepared response")
        }));
        let mut waiting = Box::pin(view.native_request(async { Ok("unmarked response") }));
        let waker = futures::task::noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);
        assert!(first.as_mut().poll(&mut cx).is_pending());
        assert!(preparation.1.load(Ordering::Acquire));
        assert!(waiting.as_mut().poll(&mut cx).is_pending());
        assert!(preparation.1.load(Ordering::Acquire));
        assert!(state.try_corsa_request_scope().is_none());
        release.send(()).unwrap();
        assert_eq!(first.await.unwrap(), "prepared response");
        assert!(!preparation.1.load(Ordering::Acquire));
        assert!(receiver.try_recv().is_ok());
        assert_eq!(waiting.await.unwrap(), "unmarked response");
        assert!(receiver.try_recv().is_err());
        assert!(state.try_corsa_request_scope().is_some());
    });
    let mut pending = scheduler.pending.lock();
    pending.jobs.verify();
    assert!(pending.take_ready(Instant::now()).is_some());
    assert!(pending.jobs.is_empty());
}
