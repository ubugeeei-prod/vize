//! Raw JSON-RPC request shapes for the editor LSP transport.
//!
//! The pinned runtime answers editor requests over `--lsp --stdio` with
//! untyped payloads, so every request keeps `Value` params and results.

use lsp_types::Uri;
use serde_json::Value;

pub(super) struct RawHoverRequest;

impl lsp_types::request::Request for RawHoverRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/hover";
}

pub(super) struct RawCompletionRequest;

impl lsp_types::request::Request for RawCompletionRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/completion";
}

pub(super) struct RawDefinitionRequest;

impl lsp_types::request::Request for RawDefinitionRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/definition";
}

pub(super) struct RawTypeDefinitionRequest;

impl lsp_types::request::Request for RawTypeDefinitionRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/typeDefinition";
}

pub(super) struct RawDeclarationRequest;

impl lsp_types::request::Request for RawDeclarationRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/declaration";
}

pub(super) struct RawImplementationRequest;

impl lsp_types::request::Request for RawImplementationRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/implementation";
}

pub(super) struct RawPrepareCallHierarchyRequest;

impl lsp_types::request::Request for RawPrepareCallHierarchyRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/prepareCallHierarchy";
}

pub(super) struct RawCallHierarchyIncomingCallsRequest;

impl lsp_types::request::Request for RawCallHierarchyIncomingCallsRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "callHierarchy/incomingCalls";
}

pub(super) struct RawCallHierarchyOutgoingCallsRequest;

impl lsp_types::request::Request for RawCallHierarchyOutgoingCallsRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "callHierarchy/outgoingCalls";
}

pub(super) struct RawReferencesRequest;

impl lsp_types::request::Request for RawReferencesRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/references";
}

pub(super) struct RawPrepareRenameRequest;

impl lsp_types::request::Request for RawPrepareRenameRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/prepareRename";
}

pub(super) struct RawRenameRequest;

impl lsp_types::request::Request for RawRenameRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/rename";
}

pub(super) struct RawSignatureHelpRequest;

impl lsp_types::request::Request for RawSignatureHelpRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/signatureHelp";
}

pub(super) struct RawWillRenameFilesRequest;

impl lsp_types::request::Request for RawWillRenameFilesRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "workspace/willRenameFiles";
}

pub(super) fn signature_help_request_params(
    uri: &Uri,
    line: u32,
    character: u32,
    context: Option<Value>,
) -> Value {
    let context = context.unwrap_or_else(|| {
        serde_json::json!({
            "triggerKind": 1,
            "isRetrigger": false
        })
    });
    serde_json::json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character },
        "context": context,
    })
}

pub(super) fn will_rename_files_request_params(renames: &[(&str, &str)]) -> Value {
    let files = renames
        .iter()
        .map(|(old_uri, new_uri)| {
            serde_json::json!({
                "oldUri": old_uri,
                "newUri": new_uri,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({ "files": files })
}
