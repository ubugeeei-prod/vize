//! Rename for `.jsx`/`.tsx` Vue components over the Corsa bridge (#1498).
//!
//! The JSX parallel to the SFC [`RenameService`](crate::ide::RenameService):
//! lower the document to plain virtual TS, forward-map the cursor, call the
//! **same** `CorsaBridge::prepare_rename` / `CorsaBridge::rename` the SFC path
//! uses, then map every returned edit range back to the original `.jsx`/`.tsx`
//! source. Edits the backend produces against this document's own virtual TS are
//! retargeted at the real source range; edits in other real files pass through
//! unchanged.
//!
//! The rename **guard** mirrors SFC: `rename` validates the new name is a legal
//! identifier before touching the backend (the type backend's own
//! prepare-rename gates *where* a rename may start). Gated by the caller on
//! `typeChecker.jsxTypecheck`.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::collections::HashMap;
use std::sync::Arc;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf, PrepareRenameResponse,
    TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};
use vize_canon::CorsaBridge;

use super::service::JsxService;
use super::virtual_ts::JsxVirtualTs;
use crate::ide::IdeContext;

/// Rename service for `.jsx`/`.tsx` components.
pub struct JsxRenameService;

impl JsxRenameService {
    /// Check whether a rename may start at the cursor, in `.jsx`/`.tsx`
    /// coordinates. Delegates the decision to the type backend (so it only
    /// allows renaming real symbols) and maps the returned range back to source.
    pub async fn prepare_rename(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<PrepareRenameResponse> {
        let bridge = corsa_bridge?;
        let (virtual_ts, uri, line, character) = JsxService::prepare_request(ctx, &bridge).await?;

        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response: PrepareRenameResponse = serde_json::from_value(response).ok()?;
        Self::map_prepare_rename(ctx, &virtual_ts, response)
    }

    /// Rename the symbol at the cursor across the project, mapping the edits in
    /// this document's virtual TS back onto the original `.jsx`/`.tsx` source.
    pub async fn rename(
        ctx: &IdeContext<'_>,
        new_name: &str,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<WorkspaceEdit> {
        // Guard: reject illegal identifiers before touching the backend, as the
        // SFC rename service does.
        if !is_valid_identifier(new_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        let (virtual_ts, uri, line, character) = JsxService::prepare_request(ctx, &bridge).await?;

        let edit = bridge
            .rename(&uri, line, character, new_name)
            .await
            .ok()??;
        let edit: WorkspaceEdit = serde_json::from_value(edit).ok()?;
        Self::map_workspace_edit(ctx, &virtual_ts, &uri, edit)
    }

    /// Translate a prepare-rename payload from virtual-TS coordinates into the
    /// original `.jsx`/`.tsx` source.
    fn map_prepare_rename(
        ctx: &IdeContext<'_>,
        virtual_ts: &JsxVirtualTs,
        response: PrepareRenameResponse,
    ) -> Option<PrepareRenameResponse> {
        match response {
            PrepareRenameResponse::Range(range) => {
                JsxService::map_virtual_range(virtual_ts, &ctx.content, range)
                    .map(PrepareRenameResponse::Range)
            }
            PrepareRenameResponse::RangeWithPlaceholder { range, placeholder } => {
                JsxService::map_virtual_range(virtual_ts, &ctx.content, range)
                    .map(|range| PrepareRenameResponse::RangeWithPlaceholder { range, placeholder })
            }
            PrepareRenameResponse::DefaultBehavior { default_behavior } => {
                Some(PrepareRenameResponse::DefaultBehavior { default_behavior })
            }
        }
    }

    /// Rewrite a workspace edit so edits in this document's virtual TS target the
    /// original `.jsx`/`.tsx` source; edits in other real files pass through.
    fn map_workspace_edit(
        ctx: &IdeContext<'_>,
        virtual_ts: &JsxVirtualTs,
        request_uri: &str,
        mut edit: WorkspaceEdit,
    ) -> Option<WorkspaceEdit> {
        if let Some(changes) = edit.changes.take() {
            let mut mapped: HashMap<Url, Vec<TextEdit>> = HashMap::with_capacity(changes.len());
            for (uri, edits) in changes {
                if JsxService::same_uri(uri.as_str(), request_uri) {
                    let entry = mapped.entry(ctx.uri.clone()).or_default();
                    entry.extend(
                        edits
                            .into_iter()
                            .filter_map(|edit| Self::map_text_edit(ctx, virtual_ts, edit)),
                    );
                } else {
                    mapped.insert(uri, edits);
                }
            }
            if !mapped.is_empty() {
                edit.changes = Some(mapped);
            }
        }

        if let Some(document_changes) = edit.document_changes.take() {
            let mapped = match document_changes {
                DocumentChanges::Edits(edits) => {
                    let edits = edits
                        .into_iter()
                        .filter_map(|edit| {
                            Self::map_document_edit(ctx, virtual_ts, request_uri, edit)
                        })
                        .collect::<Vec<_>>();
                    (!edits.is_empty()).then_some(DocumentChanges::Edits(edits))
                }
                DocumentChanges::Operations(operations) => {
                    let operations = operations
                        .into_iter()
                        .filter_map(|operation| match operation {
                            DocumentChangeOperation::Edit(edit) => {
                                Self::map_document_edit(ctx, virtual_ts, request_uri, edit)
                                    .map(DocumentChangeOperation::Edit)
                            }
                            DocumentChangeOperation::Op(op) => {
                                Some(DocumentChangeOperation::Op(op))
                            }
                        })
                        .collect::<Vec<_>>();
                    (!operations.is_empty()).then_some(DocumentChanges::Operations(operations))
                }
            };
            if let Some(mapped) = mapped {
                edit.document_changes = Some(mapped);
            }
        }

        if workspace_edit_is_empty(&edit) {
            None
        } else {
            Some(edit)
        }
    }

    fn map_document_edit(
        ctx: &IdeContext<'_>,
        virtual_ts: &JsxVirtualTs,
        request_uri: &str,
        mut edit: TextDocumentEdit,
    ) -> Option<TextDocumentEdit> {
        if JsxService::same_uri(edit.text_document.uri.as_str(), request_uri) {
            edit.text_document.uri = ctx.uri.clone();
            edit.edits = edit
                .edits
                .into_iter()
                .filter_map(|entry| match entry {
                    OneOf::Left(text_edit) => {
                        Self::map_text_edit(ctx, virtual_ts, text_edit).map(OneOf::Left)
                    }
                    OneOf::Right(annotated) => {
                        Self::map_annotated_text_edit(ctx, virtual_ts, annotated).map(OneOf::Right)
                    }
                })
                .collect();
        }

        if edit.edits.is_empty() {
            None
        } else {
            Some(edit)
        }
    }

    fn map_annotated_text_edit(
        ctx: &IdeContext<'_>,
        virtual_ts: &JsxVirtualTs,
        mut edit: AnnotatedTextEdit,
    ) -> Option<AnnotatedTextEdit> {
        edit.text_edit = Self::map_text_edit(ctx, virtual_ts, edit.text_edit)?;
        Some(edit)
    }

    fn map_text_edit(
        ctx: &IdeContext<'_>,
        virtual_ts: &JsxVirtualTs,
        mut edit: TextEdit,
    ) -> Option<TextEdit> {
        edit.range = JsxService::map_virtual_range(virtual_ts, &ctx.content, edit.range)?;
        Some(edit)
    }
}

fn workspace_edit_is_empty(edit: &WorkspaceEdit) -> bool {
    let changes_empty = edit
        .changes
        .as_ref()
        .is_none_or(|changes| changes.values().all(Vec::is_empty));
    let document_changes_empty =
        edit.document_changes
            .as_ref()
            .is_none_or(|changes| match changes {
                DocumentChanges::Edits(edits) => edits.is_empty(),
                DocumentChanges::Operations(operations) => operations.is_empty(),
            });
    changes_empty && document_changes_empty
}

/// Whether `s` is a legal JS/TS identifier (rename target guard).
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ServerState;

    fn ctx_for<'a>(
        state: &'a ServerState,
        uri: &'a Url,
        source: &str,
        marker: &str,
    ) -> IdeContext<'a> {
        let offset = source.find(marker).expect("marker present") + marker.len();
        IdeContext::with_content(state, uri, offset, source.to_string())
    }

    #[test]
    fn rejects_invalid_new_name() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123abc"));
        assert!(!is_valid_identifier("has space"));
        assert!(is_valid_identifier("renamed"));
        assert!(is_valid_identifier("_x"));
        assert!(is_valid_identifier("$ref"));
    }

    #[test]
    fn rename_with_invalid_identifier_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.msg");
            // Even with a (would-be) bridge, an illegal name short-circuits.
            assert!(
                JsxRenameService::rename(&ctx, "not valid", None)
                    .await
                    .is_none()
            );
        });
    }

    #[test]
    fn prepare_rename_without_bridge_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.msg");
            assert!(JsxRenameService::prepare_rename(&ctx, None).await.is_none());
        });
    }
}
