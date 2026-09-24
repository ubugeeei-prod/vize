//! LSP signature-help payloads returned by the Corsa bridge.
//!
//! Uses `std::string::String` for serde deserialization compatibility.
#![expect(clippy::disallowed_types, reason = "dependency API uses std String")]

use serde::Deserialize;

use super::LspDocumentation;

/// LSP signature-help response.
#[derive(Debug, Clone, Deserialize)]
pub struct LspSignatureHelp {
    /// Candidate call signatures.
    pub signatures: Vec<LspSignatureInformation>,
    /// Selected signature, when the server identifies one.
    #[serde(rename = "activeSignature")]
    pub active_signature: Option<u32>,
    /// Selected parameter, when the server identifies one.
    #[serde(rename = "activeParameter")]
    pub active_parameter: Option<u32>,
}

/// One candidate call signature.
#[derive(Debug, Clone, Deserialize)]
#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub struct LspSignatureInformation {
    /// Display label for the complete signature.
    pub label: std::string::String,
    /// Documentation attached to the signature.
    pub documentation: Option<LspDocumentation>,
    /// Parameters in declaration order.
    pub parameters: Option<Vec<LspParameterInformation>>,
    /// Signature-local active parameter.
    #[serde(rename = "activeParameter")]
    pub active_parameter: Option<u32>,
}

/// One parameter in a signature-help response.
#[derive(Debug, Clone, Deserialize)]
pub struct LspParameterInformation {
    /// Parameter label as text or UTF-16 offsets into the signature label.
    pub label: LspParameterLabel,
    /// Documentation attached to the parameter.
    pub documentation: Option<LspDocumentation>,
}

/// LSP permits parameter labels as text or a two-offset tuple.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub enum LspParameterLabel {
    String(std::string::String),
    Offsets([u32; 2]),
}
