#![expect(
    clippy::disallowed_methods,
    reason = "test expectations are built as std Strings with to_string"
)]

use super::manual::collect_import_rename_edits;
use super::manual_tests::{file_uri, test_dir};
use crate::server::ServerState;
use std::fs;
use tower_lsp::lsp_types::FileRename;

/// The scan routes only importers whose specifiers reach a renamed path. A
/// directory-index import never spells the renamed file's name, through a
/// relative path, `./`, or an alias, and must still be rewritten.
#[test]
fn rewrites_directory_index_imports_that_never_spell_the_renamed_file() {
    let dir = test_dir();
    let root = dir.path();
    let components = root.join("src/components");
    let views = root.join("src/views");
    fs::create_dir_all(&components).unwrap();
    fs::create_dir_all(&views).unwrap();
    fs::write(root.join("package.json"), "{}").unwrap();
    let old_index = components.join("index.ts");
    let new_main = components.join("main.ts");
    fs::write(&old_index, "export const a = 1;\n").unwrap();
    fs::write(
        root.join("src/App.vue"),
        "<script setup lang=\"ts\">\nimport { a } from './components';\nimport { a as b } from '@/components';\n</script>\n",
    )
    .unwrap();
    fs::write(components.join("Widget.ts"), "export * from './';\n").unwrap();
    fs::write(
        views.join("Page.ts"),
        "import { a } from '../components';\nexport const page = a;\n",
    )
    .unwrap();
    fs::write(views.join("Other.ts"), "export const other = 1;\n").unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());
    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&old_index),
            new_uri: file_uri(&new_main),
        }],
        false,
    )
    .unwrap();

    let mut rewritten = edit
        .changes
        .unwrap()
        .into_iter()
        .flat_map(|(uri, edits)| {
            let path = uri.to_file_path().unwrap();
            let file = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            edits
                .into_iter()
                .map(move |edit| (file.clone(), edit.new_text))
        })
        .collect::<Vec<_>>();
    rewritten.sort();
    let expected = [
        ("src/App.vue", "./components/main"),
        ("src/App.vue", "@/components/main"),
        ("src/components/Widget.ts", "./main"),
        ("src/views/Page.ts", "../components/main"),
    ]
    .map(|(file, text)| (file.to_string(), text.to_string()));
    assert_eq!(rewritten, expected);
}
