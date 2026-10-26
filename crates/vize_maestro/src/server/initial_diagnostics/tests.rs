use std::time::{Duration, Instant};

use tower_lsp::lsp_types::Url;

use super::{EDIT_DIAGNOSTIC_DEBOUNCE, MAX_PENDING_DOCUMENTS, PendingInitialDiagnostics};

fn finish_sync(pending: &mut PendingInitialDiagnostics) {
    while let Some((uri, job)) = pending.take_sync() {
        pending.complete_sync(&uri, job.sequence);
    }
}

#[test]
fn completed_diagnostics_consume_only_the_satisfied_initial_version() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    pending.insert(uri.clone(), 3, now);
    pending.complete(&uri, 2);
    assert_eq!(pending.jobs[&uri].version, 3);
    pending.complete(&uri, 3);
    assert!(pending.take_ready(now).is_none());
    pending.insert(uri.clone(), 4, now);
    pending.complete(&uri, 5);
    assert!(pending.take_ready(now).is_none());
}

#[test]
fn pending_jobs_keep_only_the_newest_version_per_uri() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    pending.insert(uri.clone(), 1, now + Duration::from_secs(1));
    pending.insert(uri.clone(), 3, now + Duration::from_secs(3));
    pending.insert(uri.clone(), 2, now + Duration::from_secs(2));

    assert_eq!(pending.jobs.len(), 1);
    assert!(pending.take_ready(now + Duration::from_secs(2)).is_none());
    finish_sync(&mut pending);
    let (ready_uri, ready) = pending.take_ready(now + Duration::from_secs(3)).unwrap();
    assert_eq!(ready_uri, uri);
    assert_eq!(ready.version, 3);
    assert!(pending.jobs.is_empty());
}

#[test]
fn sync_feedback_runs_once_for_the_latest_open_version() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    pending.insert(uri.clone(), 1, now + Duration::from_secs(1));
    pending.insert(uri.clone(), 2, now + Duration::from_secs(1));
    let (sync_uri, job) = pending.take_sync().unwrap();
    assert_eq!((sync_uri, job.version), (uri.clone(), 2));
    assert!(
        pending.take_ready(now + Duration::from_secs(2)).is_none(),
        "native must not overtake a running sync pass"
    );
    pending.complete_sync(&uri, job.sequence);
    assert!(pending.take_sync().is_none());
    assert!(pending.take_ready(now).is_none());
    assert_eq!(
        pending
            .take_ready(now + Duration::from_secs(1))
            .unwrap()
            .1
            .version,
        2
    );
}

#[test]
fn distinct_uris_keep_independent_pending_jobs() {
    let now = Instant::now();
    let first = Url::parse("file:///workspace/First.vue").unwrap();
    let second = Url::parse("file:///workspace/Second.vue").unwrap();
    let mut pending = PendingInitialDiagnostics::default();
    for (uri, version) in [(first, 4), (second, 7)] {
        pending.insert(uri, version, now);
    }

    finish_sync(&mut pending);
    let mut versions = [
        pending.take_ready(now).unwrap().1.version,
        pending.take_ready(now).unwrap().1.version,
    ];
    versions.sort_unstable();
    assert_eq!(versions, [4, 7]);
    assert!(pending.jobs.is_empty());
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "test-only URI generation stays local and explicit"
)]
fn pending_jobs_evict_the_oldest_uri_at_capacity() {
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    for index in 0..MAX_PENDING_DOCUMENTS {
        pending.insert(
            Url::parse(&format!("file:///workspace/{index}.vue")).unwrap(),
            index as i32,
            now,
        );
    }
    let oldest = Url::parse("file:///workspace/0.vue").unwrap();
    let newest = Url::parse("file:///workspace/newest.vue").unwrap();
    pending.insert(newest.clone(), 99, now);

    assert_eq!(pending.jobs.len(), MAX_PENDING_DOCUMENTS);
    assert!(!pending.jobs.contains_key(&oldest));
    assert_eq!(pending.jobs[&newest].version, 99);
}

#[test]
fn completing_old_sync_never_releases_a_newer_open_before_its_feedback() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    pending.insert(uri.clone(), 1, now);
    let (_, old) = pending.take_sync().unwrap();
    pending.insert(uri.clone(), 2, now);
    pending.complete_sync(&uri, old.sequence);
    assert!(pending.take_ready(now).is_none());
    let (_, current) = pending.take_sync().unwrap();
    pending.complete_sync(&uri, old.sequence);
    assert!(pending.take_ready(now).is_none());
    pending.complete_sync(&uri, current.sequence);
    assert_eq!(pending.take_ready(now).unwrap().1.version, 2);
}

#[test]
fn an_edit_after_a_cheap_pass_starts_without_the_quiet_period() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics {
        last_pass: Some(Duration::from_millis(30)),
        ..PendingInitialDiagnostics::default()
    };
    assert_eq!(pending.edit_not_before(&uri, now), now);
    pending.insert(uri.clone(), 1, now);
    assert_eq!(
        pending.edit_not_before(&uri, now),
        now + EDIT_DIAGNOSTIC_DEBOUNCE,
        "an edit behind a queued pass waits for typing to pause"
    );
}

#[test]
fn an_unmeasured_or_slow_pass_keeps_the_quiet_period() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    assert_eq!(
        pending.edit_not_before(&uri, now),
        now + EDIT_DIAGNOSTIC_DEBOUNCE
    );
    pending.last_pass = Some(EDIT_DIAGNOSTIC_DEBOUNCE);
    assert_eq!(
        pending.edit_not_before(&uri, now),
        now + EDIT_DIAGNOSTIC_DEBOUNCE
    );
}

#[test]
fn did_change_after_a_cheap_settled_pass_validates_without_the_quiet_period() {
    let fixture = super::open_importer_tests::queued_open();
    {
        let mut pending = fixture.pending.lock();
        finish_sync(&mut pending);
        let later = Instant::now() + Duration::from_secs(2);
        assert_eq!(pending.take_ready(later).unwrap().1.version, 1);
        pending.last_pass = Some(Duration::from_millis(30));
    }
    let change = tower_lsp::lsp_types::DidChangeTextDocumentParams {
        text_document: tower_lsp::lsp_types::VersionedTextDocumentIdentifier {
            uri: fixture.uri.clone(),
            version: 2,
        },
        content_changes: vec![tower_lsp::lsp_types::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "<template>{{ edited }}</template>".into(),
        }],
    };
    crate::runtime::block_on(tower_lsp::LanguageServer::did_change(
        fixture.service.inner(),
        change,
    ));

    let (uri, job) = fixture.pending.lock().take_ready(Instant::now()).unwrap();
    assert_eq!(
        (uri, job.version, job.cause),
        (fixture.uri.clone(), 2, super::DiagnosticsJobCause::Edit)
    );
}
