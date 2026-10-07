//! The outer collection deadline must disclose an incomplete native result.

use std::{future::Future, time::Duration};
use tower_lsp::lsp_types::Diagnostic;

pub(super) async fn with_native_diagnostic_timeout<F>(
    future: F,
) -> Result<F::Output, Vec<Diagnostic>>
where
    F: Future,
{
    crate::runtime::timeout(Duration::from_secs(10), future)
        .await
        .map_err(|_| vec![super::super::corsa::collect::typecheck_timed_out_hint()])
}

#[cfg(test)]
mod tests;
