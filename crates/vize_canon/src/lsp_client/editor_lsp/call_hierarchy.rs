use corsa::runtime::block_on;
use serde_json::Value;
use vize_s0::{String, cstr};

use super::{
    EditorLspSession,
    requests::{
        RawCallHierarchyIncomingCallsRequest, RawCallHierarchyOutgoingCallsRequest,
        RawPrepareCallHierarchyRequest,
    },
};

impl EditorLspSession {
    pub(super) fn prepare_call_hierarchy(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(
            self.client
                .request::<RawPrepareCallHierarchyRequest>(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character },
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP call hierarchy: {error}"))
    }

    pub(super) fn call_hierarchy_incoming_calls(
        &mut self,
        item: Value,
    ) -> Result<Option<Value>, String> {
        self.ready_workspace_request()?;
        block_on(
            self.client
                .request::<RawCallHierarchyIncomingCallsRequest>(serde_json::json!({
                    "item": item,
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP incoming call hierarchy: {error}"))
    }

    pub(super) fn call_hierarchy_outgoing_calls(
        &mut self,
        item: Value,
    ) -> Result<Option<Value>, String> {
        self.ready_workspace_request()?;
        block_on(
            self.client
                .request::<RawCallHierarchyOutgoingCallsRequest>(serde_json::json!({
                    "item": item,
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP outgoing call hierarchy: {error}"))
    }
}
