//! The identical conservation law runs against the original and repaired scheduler.

use super::{PendingInitialDiagnostics, finish_sync};
use std::time::Instant;
use tower_lsp::lsp_types::Url;

#[expect(
    clippy::disallowed_macros,
    reason = "test-only URI generation stays local and explicit"
)]
pub(super) fn capacity_uri(index: usize) -> Url {
    Url::parse(&format!("file:///workspace/capacity-{index}.vue")).unwrap()
}

#[test]
fn every_open_uri_retains_its_latest_job_at_65_129_and_2500_documents() {
    for count in [65, 129, 2500] {
        let now = Instant::now();
        let mut pending = PendingInitialDiagnostics::default();
        let uris: Vec<_> = (0..count).map(capacity_uri).collect();
        for uri in &uris {
            pending.insert(uri.clone(), 1, now);
            pending.insert(uri.clone(), 3, now);
            pending.insert(uri.clone(), 2, now);
        }
        assert_eq!(
            pending.jobs.len(),
            count,
            "accepted distinct URIs cannot be evicted"
        );
        finish_sync(&mut pending);
        let mut received = Vec::new();
        while let Some((uri, job)) = pending.take_ready(now) {
            assert_eq!(job.version, 3);
            received.push(uri);
        }
        let mut expected = uris;
        received.sort();
        expected.sort();
        assert_eq!(
            received, expected,
            "each latest URI must be taken exactly once"
        );
        assert!(pending.jobs.is_empty());
    }
}
