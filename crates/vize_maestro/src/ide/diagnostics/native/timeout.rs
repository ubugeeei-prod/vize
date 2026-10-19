//! The outer collection deadline must disclose an incomplete native result.

use std::{future::Future, time::Duration};
use tower_lsp::lsp_types::Diagnostic;

/// Production bound for one native diagnostics pass.
const DEFAULT_NATIVE_DIAGNOSTIC_TIMEOUT: Duration = Duration::from_secs(10);

/// Overrides the bound in milliseconds. Integration tests on loaded CI runners
/// raise it so a slow type-check backend is not mistaken for a wrong result
/// (#9012); editors keep the ten-second default.
const NATIVE_DIAGNOSTIC_TIMEOUT_ENV: &str = "VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS";

fn native_diagnostic_timeout() -> Duration {
    std::env::var(NATIVE_DIAGNOSTIC_TIMEOUT_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|millis| *millis > 0)
        .map_or(DEFAULT_NATIVE_DIAGNOSTIC_TIMEOUT, Duration::from_millis)
}

pub(crate) async fn with_native_diagnostic_timeout<F>(
    future: F,
) -> Result<F::Output, Vec<Diagnostic>>
where
    F: Future,
{
    let bound = native_diagnostic_timeout();
    let started = std::time::Instant::now();
    crate::runtime::timeout(bound, future).await.map_err(|_| {
        // Name the bound and the elapsed time so a slow backend can be told
        // apart from a request that never returns.
        tracing::warn!(
            "native diagnostics outran the {}ms bound after {}ms; publishing the timeout hint",
            bound.as_millis(),
            started.elapsed().as_millis()
        );
        vec![super::super::corsa::collect::typecheck_timed_out_hint()]
    })
}

#[cfg(test)]
mod tests;
