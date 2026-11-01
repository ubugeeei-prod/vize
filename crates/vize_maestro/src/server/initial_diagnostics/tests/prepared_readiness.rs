//! Prepared readiness changes a deadline, never source or queue ownership.

use std::time::{Duration, Instant};
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::super::{
    DiagnosticsJobCause, PendingInitialDiagnostics, pending::InitialDiagnosticsJob,
};
use crate::server::ServerState;

fn world() -> (ServerState, Url) {
    let state = ServerState::new();
    let uri = Url::parse("file:///prepared/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), "<template />".into(), 1, "vue".into());
    (state, uri)
}

fn ready_preparation(
    pending: &mut PendingInitialDiagnostics,
    uri: &Url,
    now: Instant,
) -> InitialDiagnosticsJob {
    pending.insert(uri.clone(), 1, now + Duration::from_secs(1));
    let (_, sync) = pending.take_sync().unwrap();
    pending.complete_sync(uri, sync.sequence);
    pending.take_preparation().unwrap().1
}

#[test]
fn current_completed_preparation_advances_only_the_same_open_slot() {
    let (state, uri) = world();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    let job = ready_preparation(&mut pending, &uri, now);
    let stamp = state.corsa_request_stamp();
    assert!(pending.take_ready(now).is_none());
    assert!(pending.advance_prepared_open(&uri, &job, stamp, &state, now));
    pending.jobs.verify();
    let (selected_uri, selected) = pending.take_ready(now).unwrap();
    assert_eq!(selected_uri, uri);
    assert_eq!(
        (selected.version, selected.sequence, selected.cause),
        (1, job.sequence, DiagnosticsJobCause::Open)
    );
    assert_eq!(selected.not_before, now);
    assert!(pending.jobs.is_empty());
    pending.jobs.assert_slots(1);
}

#[test]
fn stale_source_reopen_environment_and_retirement_keep_the_original_grace() {
    for control in ["edit", "close", "reopen", "environment", "retired"] {
        let (state, uri) = world();
        let now = Instant::now();
        let mut pending = PendingInitialDiagnostics::default();
        let job = ready_preparation(&mut pending, &uri, now);
        let stamp = state.corsa_request_stamp();
        match control {
            "edit" => {
                assert!(state.documents.apply_changes(
                    &uri,
                    vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: "<template>changed</template>".into(),
                    }],
                    2,
                ));
            }
            "close" => {
                state.documents.close(&uri);
            }
            "reopen" => {
                state.documents.close(&uri);
                state.documents.open(
                    uri.clone(),
                    "<template>new owner</template>".into(),
                    1,
                    "vue".into(),
                );
            }
            "environment" => state.set_workspace_root(std::path::PathBuf::from("/another-project")),
            "retired" => state.retire_project_contexts(),
            _ => {}
        }
        assert!(!stamp.is_current(&state), "{control}");
        assert!(
            !pending.advance_prepared_open(&uri, &job, stamp, &state, now),
            "{control}"
        );
        assert_eq!(pending.jobs[&uri].not_before, now + Duration::from_secs(1));
        assert!(pending.take_ready(now).is_none());
        pending.jobs.verify();
    }
}

#[test]
fn older_completion_never_advances_a_replaced_or_reopened_job() {
    for same_version in [false, true] {
        let (state, uri) = world();
        let now = Instant::now();
        let mut pending = PendingInitialDiagnostics::default();
        let old = ready_preparation(&mut pending, &uri, now);
        let version = if same_version { 1 } else { 2 };
        pending.insert(uri.clone(), version, now + Duration::from_secs(2));
        let (_, current_sync) = pending.take_sync().unwrap();
        pending.complete_sync(&uri, current_sync.sequence);
        let (_, current) = pending.take_preparation().unwrap();
        assert_ne!(current.sequence, old.sequence);
        let stamp = state.corsa_request_stamp();
        assert!(!pending.advance_prepared_open(&uri, &old, stamp, &state, now));
        assert_eq!(pending.jobs[&uri].sequence, current.sequence);
        assert_eq!(pending.jobs[&uri].not_before, now + Duration::from_secs(2));
        assert!(pending.take_ready(now).is_none());
        pending.jobs.verify();
    }
}

#[test]
fn sync_feedback_and_actual_preparation_cannot_be_overtaken() {
    let (state, uri) = world();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    pending.insert(uri.clone(), 1, now + Duration::from_secs(1));
    let queued = pending.jobs[&uri].clone();
    let stamp = state.corsa_request_stamp();
    assert!(!pending.advance_prepared_open(&uri, &queued, stamp, &state, now));
    let (_, sync) = pending.take_sync().unwrap();
    assert!(!pending.advance_prepared_open(&uri, &queued, stamp, &state, now));
    pending.complete_sync(&uri, sync.sequence);
    assert!(!pending.advance_prepared_open(&uri, &queued, stamp, &state, now));
    let (_, prepared) = pending.take_preparation().unwrap();
    assert!(pending.advance_prepared_open(&uri, &prepared, stamp, &state, now));
    pending.jobs.verify();
}

#[test]
fn retained_retries_keep_their_original_deadline_after_preparation() {
    let (state, uri) = world();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    pending.insert_retry(uri.clone(), 1, now + Duration::from_secs(1));
    assert!(pending.take_sync().is_none());
    let (_, retry) = pending.take_preparation().unwrap();
    assert_eq!(retry.cause, DiagnosticsJobCause::Retry);
    assert!(!pending.advance_prepared_open(&uri, &retry, state.corsa_request_stamp(), &state, now));
    assert!(pending.take_ready(now).is_none());
    assert_eq!(
        pending
            .take_ready(now + Duration::from_secs(1))
            .unwrap()
            .1
            .cause,
        DiagnosticsJobCause::Retry
    );
    pending.jobs.verify();
}

#[test]
fn edit_deadlines_missing_slots_and_elapsed_grace_are_unchanged() {
    let (state, uri) = world();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    let job = ready_preparation(&mut pending, &uri, now);
    pending.jobs.remove(&uri);
    assert!(!pending.advance_prepared_open(&uri, &job, state.corsa_request_stamp(), &state, now));
    pending.insert_with_continuation(
        uri.clone(),
        2,
        now + Duration::from_millis(150),
        DiagnosticsJobCause::Edit,
        None,
        None,
    );
    let edit = pending.jobs[&uri].clone();
    assert!(!pending.advance_prepared_open(&uri, &edit, state.corsa_request_stamp(), &state, now));
    assert_eq!(
        pending.jobs[&uri].not_before,
        now + Duration::from_millis(150)
    );
    pending.jobs.remove(&uri);
    let elapsed = ready_preparation(&mut pending, &uri, now);
    let later = now + Duration::from_secs(2);
    assert!(!pending.advance_prepared_open(
        &uri,
        &elapsed,
        state.corsa_request_stamp(),
        &state,
        later
    ));
    assert_eq!(pending.jobs[&uri].not_before, now + Duration::from_secs(1));
    pending.jobs.verify();
}
