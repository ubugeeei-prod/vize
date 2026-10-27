//! Complete stamped native publication and pending-job retirement.

use super::CollectedDiagnostics;
use crate::server::MaestroServer;
#[cfg(feature = "native")]
use std::future::Future;
use tower_lsp::lsp_types::Url;

impl MaestroServer {
    pub(in crate::server) async fn publish_collected_diagnostics(
        &self,
        uri: &Url,
        collected: CollectedDiagnostics,
    ) {
        self.publish_collected_diagnostics_with_cause(uri, collected, false)
            .await;
    }

    pub(in crate::server) async fn publish_collected_diagnostics_with_cause(
        &self,
        uri: &Url,
        collected: CollectedDiagnostics,
        explicit_refresh: bool,
    ) {
        // Without native work there is no asynchronous stamped coalescing;
        // both causes retain the ordinary complete publication below.
        #[cfg(not(feature = "native"))]
        let _ = explicit_refresh;
        let CollectedDiagnostics {
            version,
            diagnostics,
            #[cfg(feature = "native")]
            stamp,
        } = collected;
        #[cfg(feature = "native")]
        if self.state.project_context_retired()
            || stamp.is_some_and(|stamp| !stamp.is_current(&self.state))
        {
            self.retry_current_diagnostics(uri);
            return;
        }
        tracing::info!(
            "publishing collected diagnostics for {} version {}",
            uri,
            version
        );
        if self.state.documents.version(uri) != Some(version) {
            #[cfg(feature = "native")]
            self.retry_current_diagnostics(uri);
            tracing::debug!(
                "skipping superseded diagnostics for {}: collected version {}, current {:?}",
                uri,
                version,
                self.state.documents.version(uri)
            );
            return;
        }
        // A taken initial/retry job may outlive a foreground publish. Claim
        // the complete result for this source/environment, not just the root
        // version: dependency/configuration changes must still republish it.
        #[cfg(feature = "native")]
        let retained = super::super::initial_diagnostics::RetainedDiagnostics::new(self, uri);
        #[cfg(feature = "native")]
        let mut publication = if let Some(stamp) = stamp {
            let document = self.state.diagnostic_lock(uri);
            let claim = document.claim(
                &self.state,
                uri,
                version,
                stamp,
                &diagnostics,
                explicit_refresh,
            );
            if !stamp.is_current(&self.state) || self.state.documents.version(uri) != Some(version)
            {
                self.retry_current_diagnostics(uri);
                return;
            }
            let publication = match claim {
                Ok(Some(publication)) => publication,
                Ok(None) => {
                    if let Some(scheduler) = &self.initial_diagnostics {
                        scheduler.complete_if_current(uri, version, stamp, &self.state);
                    }
                    retained.finish();
                    self.publish_typecheck_unavailable_notice().await;
                    return;
                }
                Err(_) => {
                    self.retry_current_diagnostics(uri);
                    return;
                }
            };
            Some(publication)
        } else {
            None
        };
        self.state
            .cache_lint_hover_diagnostics(uri, version, &diagnostics);
        tracing::info!(
            "cached collected diagnostics for {} version {}",
            uri,
            version
        );

        // An importer refresh, save, or edit may satisfy a queued initial pass.
        // Retire that job only after collecting the complete current result;
        // parser-only initial feedback deliberately bypasses this method.
        #[cfg(feature = "native")]
        if let Some(scheduler) = &self.initial_diagnostics
            && let Some(stamp) = stamp
        {
            scheduler.complete_if_current(uri, version, stamp, &self.state);
        }
        tracing::info!(
            "sending collected diagnostics for {} version {}",
            uri,
            version
        );

        #[cfg(feature = "native")]
        {
            let notification =
                self.client
                    .publish_diagnostics(uri.clone(), diagnostics, Some(version));
            futures::pin_mut!(notification);
            futures::future::poll_fn(|cx| {
                let result = notification.as_mut().poll(cx);
                // tower-lsp 0.20 clones the futures-channel 0.3 sender for
                // each notification. Its first poll enqueues into that
                // sender's reserved slot; Pending can then mean only flush
                // backpressure. Dropping that flush cannot retract the
                // queued notification, so retain its claim from this poll.
                if let Some(publication) = publication.take() {
                    publication.finish();
                }
                result
            })
            .await;
            retained.finish();
        }
        #[cfg(not(feature = "native"))]
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;
        tracing::info!("sent collected diagnostics for {} version {}", uri, version);

        #[cfg(feature = "native")]
        self.publish_typecheck_unavailable_notice().await;
    }
}
