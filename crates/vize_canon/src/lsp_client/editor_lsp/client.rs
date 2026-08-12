//! `CorsaProjectClient` entry points that route editor requests through the
//! lazily spawned editor LSP session.

use serde_json::Value;
use vize_carton::{String, cstr};

use super::{CorsaProjectClient, EditorLspSession};
use crate::lsp_client::lsp_transport_error_is_transient;

impl CorsaProjectClient {
    /// Answer a hover through the editor LSP transport, spawning the session on
    /// first use.
    pub(in crate::lsp_client) fn hover_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.hover(uri, line, character))
    }

    pub(in crate::lsp_client) fn completion_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.completion(uri, line, character))
    }

    pub(in crate::lsp_client) fn definition_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.definition(uri, line, character))
    }

    pub(in crate::lsp_client) fn references_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.references(uri, line, character, include_declaration)
        })
    }

    pub(in crate::lsp_client) fn prepare_rename_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.prepare_rename(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn rename_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.rename(uri, line, character, new_name)
        })
    }

    pub(in crate::lsp_client) fn signature_help_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        context: Option<Value>,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.signature_help(uri, line, character, context.clone())
        })
    }

    /// Execute an idempotent editor query and rebuild the reusable LSP session
    /// once when its transport has become unusable.
    ///
    /// Recreating the session also marks every current virtual document dirty;
    /// [`editor_lsp_session`](Self::editor_lsp_session) synchronizes that full
    /// project before the retry. Semantic and protocol-shape failures are never
    /// retried.
    fn request_with_editor_lsp_recovery<T>(
        &mut self,
        mut request: impl FnMut(&mut EditorLspSession) -> Result<T, String>,
    ) -> Result<T, String> {
        let first = self.editor_lsp_session().and_then(&mut request);
        retry_transient_editor_request(
            self,
            first,
            CorsaProjectClient::retire_editor_lsp,
            |client| client.editor_lsp_session().and_then(request),
        )
    }

    fn editor_lsp_session(&mut self) -> Result<&mut EditorLspSession, String> {
        if self.editor_lsp.is_none() {
            self.editor_lsp = Some(EditorLspSession::spawn(
                self.executable.as_str(),
                &self.cwd,
                &self.project_root,
            )?);
            self.editor_lsp_documents_dirty = true;
        }
        let session = self
            .editor_lsp
            .as_mut()
            .ok_or_else(|| cstr!("Corsa editor LSP session did not initialize"))?;
        if self.editor_lsp_documents_dirty {
            session.synchronize(&self.document_texts)?;
            self.editor_lsp_documents_dirty = false;
        }
        Ok(session)
    }

    /// Drop the editor session so the next request respawns it after a project
    /// session transition.
    pub(in crate::lsp_client) fn retire_editor_lsp(&mut self) -> Result<(), String> {
        let result = match self.editor_lsp.as_mut() {
            Some(session) => session.shutdown(),
            None => Ok(()),
        };
        self.editor_lsp = None;
        self.editor_lsp_documents_dirty = true;
        result
    }
}

/// Apply the editor transport's one-retry policy without coupling its state
/// machine to a concrete process. Keeping this policy generic makes every
/// branch deterministic under unit test.
fn retry_transient_editor_request<State, Output>(
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
    use vize_carton::{String, cstr};

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

        assert!(error.contains("EOF while parsing first response"));
        assert!(error.contains("process is closed: jsonrpc reader"));
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
        assert!(error.contains("Broken pipe"));
        assert!(error.contains("shutdown acknowledgement timed out"));
        assert!(error.contains("replacement process failed to initialize"));
    }
}
