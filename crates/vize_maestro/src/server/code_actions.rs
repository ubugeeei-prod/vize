//! Code-action request dispatch and kind filtering.

use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
};

use super::MaestroServer;
use crate::ide::{CodeActionService, IdeContext, position_to_offset};

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
    let (revision, content) = {
        let document = server.state.documents.get(uri)?;
        (document.revision(), document.text())
    };

    let actions = if !features.lint {
        vec![]
    } else if crate::utils::is_jsx_path(uri.path()) {
        // `.jsx`/`.tsx`: surface the fixable Patina/JSX-compiler diagnostics.
        // Lint-based (parse-only), so not gated on `typeChecker.jsxTypecheck`.
        crate::ide::JsxCodeActionService::code_actions(&content, uri, range)
    } else {
        let offset = position_to_offset(&content, range.start.line, range.start.character)?;
        let ctx = IdeContext::new(&server.state, uri, offset)?;
        CodeActionService::code_actions_for_diagnostics(&ctx, range, &params.context.diagnostics)
    };

    #[cfg(feature = "native")]
    let actions =
        {
            let mut actions = actions;
            if features.typecheck
                && params.context.only.as_deref().is_none_or(|only| {
                    code_action_kind_is_requested(&CodeActionKind::QUICKFIX, only)
                })
            {
                let offset = position_to_offset(&content, range.start.line, range.start.character)?;
                let ctx = IdeContext::new(&server.state, uri, offset)?;
                actions.extend(CodeActionService::native_actions(&ctx, range).await);
            }
            // Source actions rewrite the whole script, so they are returned only
            // when the client asks for them (`only`), as on save.
            let source_kinds = native_source_kinds(params.context.only.as_deref());
            if features.typecheck && !source_kinds.is_empty() {
                let offset = position_to_offset(&content, range.start.line, range.start.character)?;
                let ctx = IdeContext::new(&server.state, uri, offset)?;
                actions.extend(CodeActionService::native_source_actions(&ctx, &source_kinds).await);
            }
            actions
        };

    if server.state.documents.get(uri)?.revision() != revision {
        return None;
    }

    filter_requested_code_actions(actions, params.context.only.as_deref())
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
mod tests {
    use super::*;

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
}
