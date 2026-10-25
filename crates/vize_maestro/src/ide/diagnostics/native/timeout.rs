//! A diagnostic progress threshold never cancels the warm native request.

use std::{future::Future, time::Duration};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};

/// Poll the same future before and after the soft threshold. The bridge keeps
/// its separate hard IPC bound and recovery policy. In particular, do not drop
/// an initial API read whose native project is still warming (#10604).
pub(crate) async fn with_native_diagnostic_progress<F, P>(
    bound: Duration,
    future: F,
    pending: impl FnOnce() -> P,
) -> F::Output
where
    F: Future,
    P: Future<Output = ()>,
{
    futures::pin_mut!(future);
    match crate::runtime::timeout(bound, future.as_mut()).await {
        Ok(result) => result,
        Err(_) => {
            tracing::warn!(
                "native diagnostics are still running after {}ms; retaining the shared project warm-up",
                bound.as_millis()
            );
            pending().await;
            future.await
        }
    }
}

pub(crate) fn typecheck_pending_hint() -> Diagnostic {
    Diagnostic {
        range: Range::default(),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: Some(NumberOrString::String("typecheck-pending".into())),
        source: Some(super::super::sources::TYPE_CHECKER.into()),
        message: "Type checking is still running while the shared project warms up. \
            Type diagnostics will appear when it finishes. Change `vize.typecheck.timeoutMs` \
            (initialization option `typecheckTimeoutMs`) or \
            `VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS` to adjust this notification threshold."
            .into(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests;
