use std::time::{Duration, Instant};

use tower_lsp::lsp_types::Url;

use super::{MAX_PENDING_DOCUMENTS, PendingInitialDiagnostics};

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
    assert_eq!(pending.take_sync(), Some((uri.clone(), 2)));
    assert_eq!(pending.take_sync(), None);
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
