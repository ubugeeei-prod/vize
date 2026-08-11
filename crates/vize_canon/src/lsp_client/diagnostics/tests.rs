use super::{CorsaError, corsa_diagnostics_error_is_unsupported, diagnostics_api_is_unsupported};
use crate::lsp_client::lsp_transport_error_is_transient;

// Regression: a missing diagnostics scope on the corsa `ProjectSession`
// must be detected through the typed `CorsaError::Unsupported` variant
// (corsa-bind raises it directly and normalizes "unknown method" RPC
// errors into it), so the project/file-diagnostics fallback gates on the
// variant rather than sniffing the rendered message. A genuine failure on
// a different variant must propagate instead of being silently swallowed.
#[test]
fn corsa_diagnostics_unsupported_uses_typed_capability_error() {
    assert!(corsa_diagnostics_error_is_unsupported(
        &CorsaError::Unsupported("file diagnostics are not supported")
    ));
    assert!(!corsa_diagnostics_error_is_unsupported(
        &CorsaError::Protocol("diagnostics request failed: process exited".into())
    ));
}

#[test]
fn recognizes_unsupported_diagnostics_api_errors() {
    assert!(diagnostics_api_is_unsupported("unknown API method"));
    assert!(diagnostics_api_is_unsupported(
        "unsupported: project diagnostics are not supported by this runtime"
    ));
    assert!(diagnostics_api_is_unsupported(
        "project diagnostics are not supported by this runtime"
    ));
    assert!(!diagnostics_api_is_unsupported(
        "Failed to request Corsa project diagnostics: process exited"
    ));
}

#[test]
fn recognizes_transient_lsp_transport_errors() {
    assert!(lsp_transport_error_is_transient(
        "protocol error: EOF while parsing a string at line 1 column 150"
    ));
    assert!(lsp_transport_error_is_transient(
        "Failed to request LSP diagnostics for file:///src/App.vue.ts: process is closed: jsonrpc reader"
    ));
    assert!(lsp_transport_error_is_transient("Broken pipe"));
    assert!(!lsp_transport_error_is_transient(
        "TypeScript semantic diagnostics are unavailable"
    ));
}
