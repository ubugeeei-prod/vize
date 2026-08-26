//! One-retry transport policy for the reusable editor LSP session.

use crate::lsp_client::lsp_transport_error_is_transient;
use vize_s0::{String, cstr};

/// Apply the editor transport's one-retry policy without coupling its state
/// machine to a concrete process. Keeping this policy generic makes every
/// branch deterministic under unit test.
pub(super) fn retry_transient_editor_request<State, Output>(
    state: &mut State,
    first: Result<Output, String>,
    recover: impl FnOnce(&mut State) -> Result<(), String>,
    retry: impl FnOnce(&mut State) -> Result<Output, String>,
) -> Result<Output, String> {
    let first_error = match first {
        Ok(output) => return Ok(output),
        Err(error) if !lsp_transport_error_is_transient(&error) => return Err(error),
        Err(error) => error,
    };

    // `retire_editor_lsp` clears the old session even when graceful shutdown
    // reports an error, so a fresh spawn is still safe and useful.
    let recovery_error = recover(state).err();
    match retry(state) {
        Ok(output) => Ok(output),
        Err(retry_error) => Err(match recovery_error {
            Some(recovery_error) => cstr!(
                "{first_error}; editor LSP session retirement also failed: {recovery_error}; one recovery retry failed: {retry_error}"
            ),
            None => cstr!(
                "{first_error}; one editor LSP transport recovery retry failed: {retry_error}"
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::retry_transient_editor_request;
    use vize_s0::{String, cstr};

    #[derive(Default)]
    struct FakeSession {
        recoveries: usize,
        retries: usize,
    }

    #[test]
    fn successful_first_request_does_not_recover_or_retry() {
        let mut session = FakeSession::default();
        let result = retry_transient_editor_request(
            &mut session,
            Ok::<_, String>(41),
            |_| panic!("successful request must not recover"),
            |_| panic!("successful request must not retry"),
        );

        assert_eq!(result.unwrap(), 41);
        assert_eq!(session.recoveries, 0);
        assert_eq!(session.retries, 0);
    }

    #[test]
    fn transient_failure_recovers_and_retries_exactly_once() {
        let mut session = FakeSession::default();
        let result = retry_transient_editor_request(
            &mut session,
            Err(cstr!("protocol error: EOF while parsing a string")),
            |session| {
                session.recoveries += 1;
                Ok(())
            },
            |session| {
                session.retries += 1;
                Ok(42)
            },
        );

        assert_eq!(result.unwrap(), 42);
        assert_eq!(session.recoveries, 1);
        assert_eq!(session.retries, 1);
    }

    #[test]
    fn non_transient_failure_propagates_without_recovery() {
        let mut session = FakeSession::default();
        let error = retry_transient_editor_request::<_, ()>(
            &mut session,
            Err(cstr!("method not found: textDocument/signatureHelp")),
            |_| panic!("semantic failure must not recover"),
            |_| panic!("semantic failure must not retry"),
        )
        .unwrap_err();

        assert_eq!(error, "method not found: textDocument/signatureHelp");
    }

    #[test]
    fn failed_retry_preserves_both_transport_errors() {
        let mut session = FakeSession::default();
        let error = retry_transient_editor_request::<_, ()>(
            &mut session,
            Err(cstr!("protocol error: EOF while parsing first response")),
            |session| {
                session.recoveries += 1;
                Ok(())
            },
            |session| {
                session.retries += 1;
                Err(cstr!("process is closed: jsonrpc reader"))
            },
        )
        .unwrap_err();

        assert_eq!(
            error,
            "protocol error: EOF while parsing first response; \
             one editor LSP transport recovery retry failed: process is closed: jsonrpc reader"
        );
        assert_eq!(session.recoveries, 1);
        assert_eq!(session.retries, 1);
    }

    #[test]
    fn recovery_error_does_not_prevent_a_fresh_retry_and_is_retained_on_failure() {
        let mut successful = FakeSession::default();
        let result = retry_transient_editor_request(
            &mut successful,
            Err(cstr!("Broken pipe")),
            |session| {
                session.recoveries += 1;
                Err(cstr!("shutdown acknowledgement timed out"))
            },
            |session| {
                session.retries += 1;
                Ok(7)
            },
        );
        assert_eq!(result.unwrap(), 7);

        let mut failed = FakeSession::default();
        let error = retry_transient_editor_request::<_, ()>(
            &mut failed,
            Err(cstr!("Broken pipe")),
            |_| Err(cstr!("shutdown acknowledgement timed out")),
            |_| Err(cstr!("replacement process failed to initialize")),
        )
        .unwrap_err();
        assert_eq!(
            error,
            "Broken pipe; \
             editor LSP session retirement also failed: shutdown acknowledgement timed out; \
             one recovery retry failed: replacement process failed to initialize"
        );
    }
}
