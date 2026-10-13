use corsa_core::fast::CompactString;
use lsp_types::request::Request;
use serde::{Deserialize, Serialize};

/// Parameters for Corsa's custom `initializeAPISession` request.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeApiSessionParams {
    /// Optional path to an already-created pipe/socket that Corsa should use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipe: Option<CompactString>,
}

/// Result returned by Corsa's custom `initializeAPISession` request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeApiSessionResult {
    /// Server-generated session identifier.
    pub session_id: CompactString,
    /// Pipe or socket path exposed for the API session.
    pub pipe: CompactString,
}

/// Marker type for the custom `initializeAPISession` request.
///
/// The method name is `custom/initializeAPISession`, matching Corsa's current
/// protocol extension.
pub enum InitializeApiSessionRequest {}

impl Request for InitializeApiSessionRequest {
    type Params = InitializeApiSessionParams;
    type Result = InitializeApiSessionResult;
    const METHOD: &'static str = "custom/initializeAPISession";
}
