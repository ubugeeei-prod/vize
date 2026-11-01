//! Code-action request dispatch and kind filtering.

use std::sync::OnceLock;
use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
};

use super::MaestroServer;
use crate::ide::{CodeActionService, IdeContext, position_to_offset};

struct ActionSnapshot<T> {
    revision: u64,
    content: T,
    #[cfg(feature = "native")]
    version: i32,
}

pub(super) async fn code_actions(
    server: &MaestroServer,
    params: &CodeActionParams,
) -> Option<CodeActionResponse> {
    let features = server.state.lsp_features();
    if !features.code_actions {
        return None;
    }

    let uri = &params.text_document.uri;
    let range = params.range;
    let snapshot = {
        let document = server.state.documents.get(uri)?;
        ActionSnapshot {
            revision: document.revision(),
            content: document.text_snapshot(),
            #[cfg(feature = "native")]
            version: document.version,
        }
    };
    let content = &snapshot.content;
    let context = OnceLock::new();

    let actions = if !features.lint {
        vec![]
    } else if crate::utils::is_jsx_path(uri.path()) {
        // `.jsx`/`.tsx`: surface the fixable Patina/JSX-compiler diagnostics.
        // Lint-based (parse-only), so not gated on `typeChecker.jsxTypecheck`.
        crate::ide::JsxCodeActionService::code_actions(content, uri, range)
    } else {
        let ctx = context
            .get_or_init(|| {
                let offset = position_to_offset(content, range.start.line, range.start.character)?;
                Some(IdeContext::with_content(
                    &server.state,
                    uri,
                    offset,
                    content.clone(),
                ))
            })
            .as_ref()?;
        #[cfg(feature = "native")]
        {
            if server.state.supports_versioned_lint_edits() {
                CodeActionService::code_actions_for_snapshot(
                    ctx,
                    range,
                    &params.context.diagnostics,
                    snapshot.revision,
                    snapshot.version,
                )
            } else {
                CodeActionService::code_actions_for_diagnostics(
                    ctx,
                    range,
                    &params.context.diagnostics,
                )
            }
        }
        #[cfg(not(feature = "native"))]
        CodeActionService::code_actions_for_diagnostics(ctx, range, &params.context.diagnostics)
    };

    #[cfg(feature = "native")]
    let actions = {
        let mut actions = actions;
        let range_kinds = native_range_kinds(params.context.only.as_deref());
        if features.typecheck && !range_kinds.is_empty() {
            let ctx = context
                .get_or_init(|| {
                    let offset =
                        position_to_offset(content, range.start.line, range.start.character)?;
                    Some(IdeContext::with_content(
                        &server.state,
                        uri,
                        offset,
                        content.clone(),
                    ))
                })
                .as_ref()?;
            actions.extend(CodeActionService::native_actions(ctx, range, &range_kinds).await);
        }
        // Source actions rewrite the whole script, so they are returned only
        // when the client asks for them (`only`), as on save.
        let source_kinds = native_source_kinds(params.context.only.as_deref());
        if features.typecheck && !source_kinds.is_empty() {
            let ctx = context
                .get_or_init(|| {
                    let offset =
                        position_to_offset(content, range.start.line, range.start.character)?;
                    Some(IdeContext::with_content(
                        &server.state,
                        uri,
                        offset,
                        content.clone(),
                    ))
                })
                .as_ref()?;
            actions.extend(CodeActionService::native_source_actions(ctx, &source_kinds).await);
        }
        actions
    };

    if server.state.documents.get(uri)?.revision() != snapshot.revision {
        return None;
    }

    filter_requested_code_actions(actions, params.context.only.as_deref())
}

/// Native range actions requested from tsgo. Refactors remain unadvertised
/// until a concrete kind has a real-tsgo whole-SFC oracle, but explicit client
/// requests can already use the same safe authored-edit path as quick fixes.
#[cfg(feature = "native")]
fn native_range_kinds(only: Option<&[CodeActionKind]>) -> Vec<CodeActionKind> {
    let Some(only) = only else {
        return vec![CodeActionKind::QUICKFIX];
    };
    let mut kinds = Vec::new();
    if code_action_kind_is_requested(&CodeActionKind::QUICKFIX, only) {
        kinds.push(CodeActionKind::QUICKFIX);
    }
    for kind in only {
        if (kind == &CodeActionKind::REFACTOR || kind.as_str().starts_with("refactor."))
            && !kinds.contains(kind)
        {
            kinds.push(kind.clone());
        }
    }
    kinds
}

/// The native source kinds an explicit `only` request covers, by the LSP
/// kind hierarchy (`source` covers `source.organizeImports`).
#[cfg(feature = "native")]
fn native_source_kinds(only: Option<&[CodeActionKind]>) -> Vec<CodeActionKind> {
    let Some(only) = only else {
        return vec![];
    };
    crate::ide::NATIVE_SOURCE_KINDS
        .into_iter()
        .map(|kind| CodeActionKind::from(kind.to_owned()))
        .filter(|kind| {
            only.iter().any(|requested| !requested.as_str().is_empty())
                && code_action_kind_is_requested(kind, only)
        })
        .collect()
}

/// Apply the LSP code-action kind hierarchy to one handler response.
///
/// A requested parent kind contains its descendants (`refactor` contains
/// `refactor.extract`), while requesting a descendant must not include its
/// parent. Commands and untyped actions cannot satisfy an `only` request
/// because the client cannot establish which kind they belong to.
fn filter_requested_code_actions(
    mut actions: CodeActionResponse,
    only: Option<&[CodeActionKind]>,
) -> Option<CodeActionResponse> {
    if let Some(requested_kinds) = only {
        actions.retain(|action| match action {
            CodeActionOrCommand::CodeAction(action) => action
                .kind
                .as_ref()
                .is_some_and(|kind| code_action_kind_is_requested(kind, requested_kinds)),
            CodeActionOrCommand::Command(_) => false,
        });
    }
    (!actions.is_empty()).then_some(actions)
}

fn code_action_kind_is_requested(kind: &CodeActionKind, requested: &[CodeActionKind]) -> bool {
    requested.iter().any(|requested_kind| {
        let requested = requested_kind.as_str();
        requested.is_empty()
            || kind == requested_kind
            || kind
                .as_str()
                .strip_prefix(requested)
                .is_some_and(|suffix| suffix.starts_with('.'))
    })
}

#[cfg(test)]
#[cfg(feature = "native")]
mod versioned_auto_import_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "native")]
    #[test]
    fn advertised_client_gets_the_actual_document_version_only_for_the_lint_fix() {
        use tower_lsp::{
            LspService,
            lsp_types::{
                ClientCapabilities, CodeActionContext, Diagnostic, PartialResultParams,
                TextDocumentContentChangeEvent, TextDocumentIdentifier, Url,
                WorkDoneProgressParams, WorkspaceClientCapabilities,
                WorkspaceEditClientCapabilities,
            },
        };
        const SOURCE: &str = include_str!(
            "../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/App.vue.txt"
        );
        const CONFIG: &str = include_str!(
            "../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/vize.config.json"
        );
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/references.json"
        ))
        .unwrap();
        let diagnostic: Diagnostic =
            serde_json::from_value(reference["diagnostics"][1].clone()).unwrap();
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("vite.config.mjs"),
            vize_l0::cstr!("export default {{ vue: {} }};", CONFIG).as_bytes(),
        )
        .unwrap();
        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        server.state.load_lsp_config(root.path());
        server
            .state
            .apply_lsp_initialization_options(Some(&serde_json::json!({ "typecheck": false })));
        let uri = Url::from_file_path(root.path().join("App.vue")).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), SOURCE.into(), 7, "vue".into());
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: diagnostic.range,
            context: CodeActionContext {
                diagnostics: vec![diagnostic.clone()],
                ..Default::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let edits = serde_json::json!([{
            "range": {"start":{"line":1,"character":2},"end":{"line":1,"character":23}},
            "newText": "<img src=\"/logo.png\" />"
        }]);
        let expected = |versioned, version| {
            let fix = if versioned {
                serde_json::json!({"documentChanges":[{
                    "textDocument":{"uri":uri,"version":version}, "edits":edits
                }]})
            } else {
                serde_json::json!({"changes":{(uri.as_str()):edits}})
            };
            serde_json::json!([
                {"title":"Fix: Use self-closing syntax","kind":"quickfix","diagnostics":[diagnostic],"edit":fix,"isPreferred":true},
                {"title":"Suppress with @vize:forget (vue/html-self-closing)","kind":"quickfix","diagnostics":[diagnostic],"edit":{"changes":{(uri.as_str()):[{
                    "range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}},
                    "newText":"  <!-- @vize:forget vue/html-self-closing -->\n"
                }]}},"isPreferred":false}
            ])
        };
        for advertised in [None, Some(false), Some(true)] {
            server
                .state
                .record_client_capabilities(&ClientCapabilities {
                    workspace: Some(WorkspaceClientCapabilities {
                        workspace_edit: Some(WorkspaceEditClientCapabilities {
                            document_changes: advertised,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            let response = futures::executor::block_on(code_actions(server, &params)).unwrap();
            assert_eq!(
                serde_json::to_value(response).unwrap(),
                expected(advertised == Some(true), 7)
            );
            assert_eq!(server.state.documents.text(&uri).unwrap(), SOURCE);
        }
        assert!(server.state.documents.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: SOURCE.into()
            }],
            8
        ));
        let response = futures::executor::block_on(code_actions(server, &params)).unwrap();
        assert_eq!(serde_json::to_value(response).unwrap(), expected(true, 8));
    }

    #[cfg(feature = "native")]
    #[test]
    fn source_actions_are_served_only_when_explicitly_requested() {
        let kinds = |only: Option<&[&str]>| {
            let only = only.map(|only| {
                only.iter()
                    .map(|kind| CodeActionKind::from((*kind).to_owned()))
                    .collect::<Vec<_>>()
            });
            native_source_kinds(only.as_deref())
                .into_iter()
                .map(|kind| kind.as_str().to_owned())
                .collect::<Vec<_>>()
        };
        assert!(kinds(None).is_empty());
        assert!(kinds(Some(&["quickfix"])).is_empty());
        assert!(kinds(Some(&["refactor"])).is_empty());
        assert_eq!(
            kinds(Some(&["source.organizeImports"])),
            ["source.organizeImports"]
        );
        assert_eq!(
            kinds(Some(&["source"])),
            [
                "source.organizeImports",
                "source.sortImports",
                "source.removeUnusedImports",
                "source.fixAll"
            ]
        );
        assert_eq!(kinds(Some(&["source.fixAll"])), ["source.fixAll"]);
        assert!(kinds(Some(&["source.fixAll.ts"])).is_empty());
    }

    #[test]
    fn requested_kinds_follow_the_lsp_hierarchy() {
        assert!(code_action_kind_is_requested(
            &CodeActionKind::QUICKFIX,
            &[CodeActionKind::EMPTY]
        ));
        assert!(code_action_kind_is_requested(
            &CodeActionKind::REFACTOR_EXTRACT,
            &[CodeActionKind::REFACTOR]
        ));
        assert!(!code_action_kind_is_requested(
            &CodeActionKind::REFACTOR,
            &[CodeActionKind::REFACTOR_EXTRACT]
        ));
        assert!(!code_action_kind_is_requested(
            &CodeActionKind::QUICKFIX,
            &[CodeActionKind::SOURCE]
        ));
        assert!(!code_action_kind_is_requested(
            &CodeActionKind::QUICKFIX,
            &[]
        ));
    }

    #[cfg(feature = "native")]
    #[test]
    fn explicit_refactor_kinds_reach_only_the_native_range_path() {
        let kinds = |only: Option<&[&str]>| {
            let only = only.map(|only| {
                only.iter()
                    .map(|kind| CodeActionKind::from((*kind).to_owned()))
                    .collect::<Vec<_>>()
            });
            native_range_kinds(only.as_deref())
                .into_iter()
                .map(|kind| kind.as_str().to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(kinds(None), ["quickfix"]);
        assert_eq!(kinds(Some(&["quickfix"])), ["quickfix"]);
        assert_eq!(kinds(Some(&["refactor"])), ["refactor"]);
        assert_eq!(
            kinds(Some(&["refactor.extract.function"])),
            ["refactor.extract.function"]
        );
        assert_eq!(
            kinds(Some(&["quickfix", "refactor.extract.function"])),
            ["quickfix", "refactor.extract.function"]
        );
        assert!(kinds(Some(&["source"])).is_empty());
    }
}
