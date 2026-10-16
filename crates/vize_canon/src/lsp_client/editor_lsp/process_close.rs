//! Closing and reaping the native editor process.

use super::{EditorLspSession, configured_project, process_receipt};
use corsa::runtime::block_on;
use std::sync::atomic::Ordering;
use vize_l0::{String, cstr};

impl EditorLspSession {
    /// Complete the selected LSP lifecycle before closing and reaping the
    /// owned process. The responder remains alive until `shutdown` returns so
    /// server-initiated requests cannot deadlock the final response.
    pub(super) fn shutdown(&mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }

        let mut first_error = None;
        // Native 7.0.2's Exit dispatch races stdin EOF into context.Canceled.
        // Its authenticated initialize identity selects acknowledged Shutdown
        // then transport EOF. Other servers retain the SDK standard lifecycle.
        let close_result = if self.shutdown_to_eof {
            block_on(async {
                let protocol = self
                    .client
                    .request_without_params::<lsp_types::request::Shutdown>()
                    .await;
                let closed = self.client.close().await;
                protocol.and(closed)
            })
        } else {
            block_on(self.client.graceful_close())
        };
        let graceful_acknowledged = close_result.is_ok();
        if let Err(error) = close_result {
            first_error = Some(cstr!(
                "Failed to gracefully close editor LSP process: {error}"
            ));
        }
        let result = self.finish_close(first_error);
        let receipt_result =
            process_receipt::report(&self.process_observer, graceful_acknowledged, true);
        match (result, receipt_result) {
            (Err(error), Err(receipt_error)) => Err(cstr!("{error}; {receipt_error}")),
            (result, receipt_result) => result.and(receipt_result),
        }
    }

    /// Tear down the editor process without waiting for a protocol shutdown
    /// response. Workspace file events use this path when the reusable editor
    /// project view has to be discarded before the next semantic request; a
    /// stuck old server must not block the foreground LSP notification queue.
    pub(super) fn discard(&mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }

        let mut first_error = None;
        if let Err(error) = block_on(self.client.close()) {
            first_error = Some(cstr!("Failed to close editor LSP process: {error}"));
        }
        let result = self.finish_close(first_error);
        let receipt_result = process_receipt::report(&self.process_observer, false, false);
        result.and(receipt_result)
    }

    fn finish_close(&mut self, mut first_error: Option<String>) -> Result<(), String> {
        // The owning process is already reaped: its socket peer must close
        // before the SDK joins the attached full-duplex reader.
        if let Some(api) = self.configured_api.take()
            && let Err(error) = api.close_attachment()
            && first_error.is_none()
        {
            first_error = Some(configured_project::configuration_error(error));
        }
        self.stop.store(true, Ordering::Relaxed);
        if let Some(responder) = self.responder.take()
            && responder.join().is_err()
            && first_error.is_none()
        {
            first_error = Some(cstr!("Editor LSP responder panicked during shutdown"));
        }
        self.closed = true;
        self.issued_completions.invalidate();

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl Drop for EditorLspSession {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            eprintln!("{error}");
        }
    }
}
