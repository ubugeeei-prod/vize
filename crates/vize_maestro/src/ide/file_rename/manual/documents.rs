//! Move synchronized buffers once after an authored file rename.
use super::{ServerState, apply_all_uri_renames, rename_targets};
use tower_lsp::lsp_types::{FileRename, Url};

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

