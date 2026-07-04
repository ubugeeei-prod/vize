use std::fs;
use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{FileRename, RenameFilesParams, Url};

use super::FileRenameService;
use crate::server::ServerState;

fn test_dir() -> tempfile::TempDir {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests");
    fs::create_dir_all(&base).unwrap();
    tempfile::tempdir_in(base).unwrap()
}

fn file_uri(path: &Path) -> String {
    Url::from_file_path(path).unwrap().to_string()
}

fn edit_text_for(
    root: &Path,
    edit: &tower_lsp::lsp_types::WorkspaceEdit,
    rel: &str,
) -> Vec<String> {
    let path = root.join(rel);
    let uri = Url::from_file_path(path).unwrap();
    edit.changes
        .as_ref()
        .and_then(|changes| changes.get(&uri))
        .unwrap()
        .iter()
        .map(|edit| edit.new_text.clone())
        .collect()
}

#[test]
fn renaming_module_declaration_index_directory_keeps_directory_specifier() {
    let dir = test_dir();
    let root = dir.path();
    let src = root.join("src");
    let old_types = src.join("types");
    let new_types = src.join("models");
    fs::create_dir_all(&old_types).unwrap();

    let entry = src.join("entry.ts");
    fs::write(
        &entry,
        "import type { Model } from './types'\ntype Loaded = import('./types')\n",
    )
    .unwrap();
    fs::write(
        old_types.join("index.d.cts"),
        "export interface Model { value: string }\n",
    )
    .unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    let edit = futures::executor::block_on(FileRenameService::will_rename_files(
        &state,
        &RenameFilesParams {
            files: vec![FileRename {
                old_uri: file_uri(&old_types),
                new_uri: file_uri(&new_types),
            }],
        },
    ))
    .unwrap();

    assert_eq!(
        edit_text_for(root, &edit, "src/entry.ts"),
        vec!["./models", "./models"]
    );
}
