//! Move synchronized buffers once after an authored file rename.
use super::{ServerState, apply_all_uri_renames, rename_targets};
use tower_lsp::lsp_types::{FileRename, Url};

#[cfg(feature = "native")]
pub(in crate::ide::file_rename) fn rename_project_open_documents(
    owner: &std::sync::Arc<ServerState>,
    renames: &[FileRename],
) -> Vec<(Url, Url)> {
    let targets = rename_targets(renames);
    let mut renamed = Vec::new();
    for old in owner.documents.uris() {
        let Some(new) = apply_all_uri_renames(&old, &targets).filter(|new| *new != old) else {
            continue;
        };
        if owner.rename_document(&old, new.clone()) {
            // Move the shared buffer before any destination config can await.
            // Old owners need cleanup, never a new config evaluation.
            let mut states = owner.cached_project_states();
            states.push(owner.clone());
            for state in states {
                state.forget_closed_document(&old);
                state.remove_virtual_docs(&old);
            }
            renamed.push((old, new));
        }
    }
    renamed
}

pub(in crate::ide::file_rename) fn rename_open_documents(
    state: &ServerState,
    renames: &[FileRename],
) -> Vec<(Url, Url)> {
    let rename_targets = rename_targets(renames);
    if rename_targets.is_empty() {
        return Vec::new();
    }

    let mut renamed_documents = Vec::new();
    let open_uris = state.documents.uris();

    for old_uri in open_uris {
        let Some(new_uri) = apply_all_uri_renames(&old_uri, &rename_targets) else {
            continue;
        };

        if new_uri == old_uri {
            continue;
        }

        if state.rename_document(&old_uri, new_uri.clone()) {
            state.remove_virtual_docs(&old_uri);

            if let Some(document) = state.documents.get(&new_uri) {
                let content = document.text();
                drop(document);
                state.update_virtual_docs(&new_uri, &content);
            }

            renamed_documents.push((old_uri, new_uri));
        }
    }

    renamed_documents
}
