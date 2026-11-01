//! Completion documentation is deferred only for a supporting native client.

use std::sync::atomic::Ordering;

use tower_lsp::lsp_types::ClientCapabilities;

use super::ServerState;

impl ServerState {
    /// Record the exact properties advertised during this session's initialize.
    /// The non-native handler cannot resolve items, so it keeps eager docs.
    pub(crate) fn record_client_capabilities(&self, capabilities: &ClientCapabilities) {
        #[cfg(feature = "native")]
        self.retain_project_client_capabilities(capabilities);
        #[cfg(feature = "native")]
        self.versioned_lint_edits.store(
            capabilities
                .workspace
                .as_ref()
                .and_then(|workspace| workspace.workspace_edit.as_ref())
                .and_then(|edit| edit.document_changes)
                == Some(true),
            Ordering::Relaxed,
        );
        crate::server::workspace_files::record_watcher_support(self, capabilities);
        let supported = cfg!(feature = "native")
            && capabilities
                .text_document
                .as_ref()
                .and_then(|document| document.completion.as_ref())
                .and_then(|completion| completion.completion_item.as_ref())
                .and_then(|item| item.resolve_support.as_ref())
                .is_some_and(|support| support.properties.iter().any(|p| p == "documentation"));
        self.completion_documentation_resolve
            .store(supported, Ordering::Relaxed);
    }

    /// This flag is independent of the type checker and workspace configuration.
    #[inline]
    pub(crate) fn supports_completion_documentation_resolve(&self) -> bool {
        self.completion_documentation_resolve
            .load(Ordering::Relaxed)
    }

    /// Only the exact client advertisement selects versioned lint edits.
    #[cfg(feature = "native")]
    pub(crate) fn supports_versioned_lint_edits(&self) -> bool {
        self.versioned_lint_edits.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientCapabilities, ServerState};
    use tower_lsp::lsp_types::{
        CompletionClientCapabilities, CompletionItemCapability,
        CompletionItemCapabilityResolveSupport, TextDocumentClientCapabilities,
    };

    #[cfg(feature = "native")]
    #[test]
    fn versioned_lint_edits_require_the_exact_workspace_advertisement() {
        use tower_lsp::lsp_types::{WorkspaceClientCapabilities, WorkspaceEditClientCapabilities};
        let state = ServerState::new();
        assert!(!state.supports_versioned_lint_edits());
        for advertised in [None, Some(false), Some(true), None] {
            let capabilities = ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_edit: Some(WorkspaceEditClientCapabilities {
                        document_changes: advertised,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            state.record_client_capabilities(&capabilities);
            assert_eq!(
                state.supports_versioned_lint_edits(),
                advertised == Some(true)
            );
            assert!(!state.supports_completion_documentation_resolve());
        }
        state.record_client_capabilities(&ClientCapabilities::default());
        assert!(!state.supports_versioned_lint_edits());
    }

    fn capabilities(properties: Option<&[&str]>) -> ClientCapabilities {
        ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                completion: Some(CompletionClientCapabilities {
                    completion_item: Some(CompletionItemCapability {
                        resolve_support: properties.map(|properties| {
                            CompletionItemCapabilityResolveSupport {
                                properties: properties
                                    .iter()
                                    .map(|property| (*property).into())
                                    .collect(),
                            }
                        }),
                        ..CompletionItemCapability::default()
                    }),
                    ..CompletionClientCapabilities::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        }
    }

    #[test]
    fn completion_documentation_resolve_defaults_to_eager_documentation() {
        let state = ServerState::new();
        assert!(!state.supports_completion_documentation_resolve());
        for capabilities in [
            ClientCapabilities::default(),
            ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities::default()),
                ..ClientCapabilities::default()
            },
            ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    completion: Some(CompletionClientCapabilities::default()),
                    ..TextDocumentClientCapabilities::default()
                }),
                ..ClientCapabilities::default()
            },
            capabilities(None),
            capabilities(Some(&[])),
            capabilities(Some(&["detail"])),
            capabilities(Some(&["Documentation"])),
        ] {
            state.record_client_capabilities(&capabilities);
            assert!(!state.supports_completion_documentation_resolve());
        }
    }

    #[test]
    fn completion_documentation_resolve_requires_the_exact_property_and_native_handler() {
        let state = ServerState::new();
        for properties in [&["documentation"][..], &["detail", "documentation"][..]] {
            state.record_client_capabilities(&capabilities(Some(properties)));
            assert_eq!(
                state.supports_completion_documentation_resolve(),
                cfg!(feature = "native")
            );
        }
        state.record_client_capabilities(&ClientCapabilities::default());
        assert!(!state.supports_completion_documentation_resolve());
    }

    #[test]
    fn completion_documentation_resolve_does_not_enable_or_require_typechecking() {
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "typecheck": false })));
        assert!(!state.is_lsp_typecheck_enabled());
        state.record_client_capabilities(&capabilities(Some(&["documentation"])));
        assert_eq!(
            state.supports_completion_documentation_resolve(),
            cfg!(feature = "native")
        );
        assert!(!state.is_lsp_typecheck_enabled());
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "typecheck": true })));
        assert!(state.is_lsp_typecheck_enabled());
        assert_eq!(
            state.supports_completion_documentation_resolve(),
            cfg!(feature = "native")
        );
    }
}
