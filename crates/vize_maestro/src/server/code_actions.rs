//! Code-action request dispatch and kind filtering.

use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
};

use super::MaestroServer;
use crate::ide::{CodeActionService, IdeContext, position_to_offset};

pub(super) fn code_actions(
    server: &MaestroServer,
    params: &CodeActionParams,
) -> Option<CodeActionResponse> {
    let features = server.state.lsp_features();
    if !features.lint || !features.code_actions {
        return None;
    }

    let uri = &params.text_document.uri;
    let range = params.range;
    let content = server.state.documents.text(uri)?;

    let actions = if crate::utils::is_jsx_path(uri.path()) {
        // `.jsx`/`.tsx`: surface the fixable Patina/JSX-compiler diagnostics.
        // Lint-based (parse-only), so not gated on `typeChecker.jsxTypecheck`.
        crate::ide::JsxCodeActionService::code_actions(&content, uri, range)
    } else {
        let offset = position_to_offset(&content, range.start.line, range.start.character)?;
        let ctx = IdeContext::new(&server.state, uri, offset)?;
        CodeActionService::code_actions(&ctx, range)
    };

    filter_requested_code_actions(actions, params.context.only.as_deref())
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
