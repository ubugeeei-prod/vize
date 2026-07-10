#![allow(clippy::disallowed_macros, clippy::disallowed_methods)]

use super::manual::{collect_import_rename_edits, rename_open_documents};
use crate::server::ServerState;
use insta::assert_snapshot;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{FileRename, Url, WorkspaceEdit};

fn test_dir() -> tempfile::TempDir {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests");
    fs::create_dir_all(&base).unwrap();
    tempfile::tempdir_in(base).unwrap()
}

fn file_uri(path: &Path) -> std::string::String {
    Url::from_file_path(path).unwrap().to_string()
}

fn normalize_edit(root: &Path, edit: &WorkspaceEdit) -> serde_json::Value {
    let mut files = BTreeMap::<std::string::String, Vec<serde_json::Value>>::new();

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
fn rewrites_vue_imports_for_component_rename() {
    let dir = test_dir();
    let root = dir.path();
    let src_dir = root.join("src");
    let components_dir = src_dir.join("components");
    fs::create_dir_all(&components_dir).unwrap();

    let app_path = src_dir.join("App.vue");
    let old_component = components_dir.join("Foo.vue");
    let new_component = components_dir.join("Bar.vue");

    fs::write(
        &app_path,
        r#"<script setup lang="ts">
import Foo from "./components/Foo.vue";
const Lazy = () => import("./components/Foo.vue");
type FooModule = typeof import("./components/Foo.vue");
</script>
"#,
    )
    .unwrap();
    fs::write(&old_component, "<template><div>foo</div></template>").unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&old_component),
            new_uri: file_uri(&new_component),
        }],
        true,
    )
    .unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
    {
      "src/App.vue": [
        {
          "newText": "./components/Bar.vue",
          "range": {
            "end": {
              "character": 37,
              "line": 1
            },
            "start": {
              "character": 17,
              "line": 1
            }
          }
        },
        {
          "newText": "./components/Bar.vue",
          "range": {
            "end": {
              "character": 47,
              "line": 2
            },
            "start": {
              "character": 27,
              "line": 2
            }
          }
        },
        {
          "newText": "./components/Bar.vue",
          "range": {
            "end": {
              "character": 52,
              "line": 3
            },
            "start": {
              "character": 32,
              "line": 3
            }
          }
        }
      ]
    }
    "###);
}

#[test]
fn rewrites_extensionless_ts_imports_without_corsa() {
    let dir = test_dir();
    let root = dir.path();
    let src_dir = root.join("src");
    let util_dir = src_dir.join("util");
    fs::create_dir_all(&util_dir).unwrap();

    let entry_path = src_dir.join("entry.ts");
    let old_module = util_dir.join("foo.ts");
    let new_module = util_dir.join("bar.ts");

    fs::write(
        &entry_path,
        "import { value } from \"./util/foo\";\nconst lazy = require(\"./util/foo\");\n",
    )
    .unwrap();
    fs::write(&old_module, "export const value = 1;\n").unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&old_module),
            new_uri: file_uri(&new_module),
        }],
        false,
    )
    .unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
    {
      "src/entry.ts": [
        {
          "newText": "./util/bar",
          "range": {
            "end": {
              "character": 33,
              "line": 0
            },
            "start": {
              "character": 23,
              "line": 0
            }
          }
        },
        {
          "newText": "./util/bar",
          "range": {
            "end": {
              "character": 32,
              "line": 1
            },
            "start": {
              "character": 22,
              "line": 1
            }
          }
        }
      ]
    }
    "###);
}

#[test]
fn renames_open_documents_inside_renamed_folder() {
    let dir = test_dir();
    let root = dir.path();
    let old_dir = root.join("src/pages");
    let new_dir = root.join("src/views");
    let file_path = old_dir.join("Home.vue");
    fs::create_dir_all(&old_dir).unwrap();
    fs::write(&file_path, "<template><div>home</div></template>").unwrap();

    let state = ServerState::new();
    state.documents.open(
        Url::from_file_path(&file_path).unwrap(),
        "<template><div>home</div></template>".to_string(),
        1,
        "vue".to_string(),
    );
    state.update_virtual_docs(
        &Url::from_file_path(&file_path).unwrap(),
        "<template><div>home</div></template>",
    );

    let renamed = rename_open_documents(
        &state,
        &[FileRename {
            old_uri: file_uri(&old_dir),
            new_uri: file_uri(&new_dir),
        }],
    );

    assert_eq!(renamed.len(), 1);
    let new_uri = Url::from_file_path(new_dir.join("Home.vue")).unwrap();
    assert!(state.documents.contains(&new_uri));
    assert!(state.get_virtual_docs(&new_uri).is_some());
}
