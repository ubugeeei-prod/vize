//! Final server termination waits for the actual session owner to close.
use super::{CorsaBridge, CorsaBridgeError};

impl CorsaBridge {
    /// Close the worker-owned client before acknowledging server termination.
    /// Unlike ordinary shutdown, an unfinished startup or abandoned operation
    /// must drain before this cleanup enters. The configured deadline still
    /// bounds the caller; a timeout is an error, never a successful close.
    pub async fn shutdown_for_exit(&self) -> Result<(), CorsaBridgeError> {
        let batch = self.close_retirement_batch().await?;
        if !batch.outcome.custody_complete || !batch.active_errors.is_empty() {
            let error = vize_l0::cstr!(
                "Active native final close was not successful: {:?}; custody: {}",
                batch.active_errors,
                batch.outcome.receipt
            );
            // The returned error owns every raw row even without a subscriber.
            self.startup_control
                .acknowledge_publication(&batch.completed);
            return Err(CorsaBridgeError::CommunicationError(error));
        }
        tracing::info!(receipt = %batch.outcome.receipt, "native active owner shutdown custody");
        self.startup_control
            .acknowledge_publication(&batch.completed);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
