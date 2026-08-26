use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tower_lsp::lsp_types::{FileRename, RenameFilesParams, Url};
use vize_maestro::ide::FileRenameService;
use vize_maestro::runtime;
use vize_maestro::server::ServerState;
use vize_s0::cstr;

fn file_uri(path: &Path) -> vize_s0::String {
    cstr!("{}", Url::from_file_path(path).unwrap())
}

fn normalize_edit(root: &Path, edit: &tower_lsp::lsp_types::WorkspaceEdit) -> serde_json::Value {
    // Inferred from the `insert` below so the `lsp_types`-side key type never
    // has to be named here.
    let mut files = BTreeMap::new();

    for (uri, edits) in edit.changes.as_ref().unwrap() {
        let path = uri.to_file_path().unwrap();
        let label = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");

        let items = edits
            .iter()
            .map(|edit| {
                serde_json::json!({
                    "range": {
                        "start": {
                            "line": edit.range.start.line,
                            "character": edit.range.start.character,
                        },
                        "end": {
                            "line": edit.range.end.line,
                            "character": edit.range.end.character,
                        }
                    },
                    "newText": edit.new_text
                })
            })
            .collect::<Vec<_>>();

        files.insert(label, items);
    }

    serde_json::json!(files)
}

#[test]
fn rewrites_extensionless_declaration_imports_for_renamed_dts() {
    assert_declaration_rename("d.ts");
}

#[test]
fn rewrites_extensionless_declaration_imports_for_renamed_dmts() {
    assert_declaration_rename("d.mts");
}

#[test]
fn rewrites_extensionless_declaration_imports_for_renamed_dcts() {
    assert_declaration_rename("d.cts");
}

fn assert_declaration_rename(extension: &str) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let src_dir = root.join("src");
    let types_dir = src_dir.join("types");
    fs::create_dir_all(&types_dir).unwrap();

    let entry_path = src_dir.join("entry.ts");
    let old_declaration = types_dir.join(cstr!("schema.{extension}"));
    let new_declaration = types_dir.join(cstr!("generated-schema.{extension}"));

    fs::write(
        &entry_path,
        "import type { Schema } from \"./types/schema\";\ntype Lazy = import(\"./types/schema\").Schema;\n",
    )
    .unwrap();
    fs::write(
        &old_declaration,
        "export interface Schema { value: string }\n",
    )
    .unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    let edit = runtime::block_on(FileRenameService::will_rename_files(
        &state,
        &RenameFilesParams {
            files: vec![FileRename {
                old_uri: file_uri(&old_declaration).into(),
                new_uri: file_uri(&new_declaration).into(),
            }],
        },
    ))
    .unwrap();

    assert_eq!(
        normalize_edit(root, &edit),
        serde_json::json!({
            "src/entry.ts": [
                {
                    "range": {
                        "start": { "line": 0, "character": 29 },
                        "end": { "line": 0, "character": 43 }
                    },
                    "newText": "./types/generated-schema"
                },
                {
                    "range": {
                        "start": { "line": 1, "character": 20 },
                        "end": { "line": 1, "character": 34 }
                    },
                    "newText": "./types/generated-schema"
                }
            ]
        })
    );
}
