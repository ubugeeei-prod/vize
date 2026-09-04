#![allow(clippy::disallowed_macros, clippy::disallowed_methods)]

//! Rename edits for `paths`-aliased specifiers (#3917).

use super::manual::collect_import_rename_edits;
use super::manual_tests::{file_uri, normalize_edit, test_dir};
use crate::server::ServerState;
use insta::assert_snapshot;
use std::fs;
use tower_lsp::lsp_types::FileRename;

#[test]
fn aliased_specifiers_follow_the_move_and_leave_the_subtree_relative() {
    let dir = test_dir();
    let root = dir.path();
    fs::create_dir_all(root.join("src/widgets")).unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    // Solution-style shell: paths live in the referenced app config (#3915).
    fs::write(
        root.join("tsconfig.json"),
        r#"{ "files": [], "references": [{ "path": "./tsconfig.app.json" }] }"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.app.json"),
        r#"{
  // aliases govern src only
  "compilerOptions": { "paths": { "@/*": ["./src/*"] } }
}"#,
    )
    .unwrap();
    fs::write(
        root.join("src/App.vue"),
        r#"<script setup lang="ts">
import Child from "./Child.vue";
import AliasChild from "@/Child.vue";
import { helper } from "@/util";
</script>
"#,
    )
    .unwrap();
    fs::write(root.join("src/Child.vue"), "<template><i /></template>").unwrap();
    fs::write(root.join("src/util.ts"), "export const helper = 1;").unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    // Move within the alias subtree: both spellings survive, style preserved.
    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&root.join("src/Child.vue")),
            new_uri: file_uri(&root.join("src/widgets/Kid.vue")),
        }],
        true,
    )
    .unwrap();
    assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
    {
      "src/App.vue": [
        {
          "newText": "./widgets/Kid.vue",
          "range": {
            "end": {
              "character": 30,
              "line": 1
            },
            "start": {
              "character": 19,
              "line": 1
            }
          }
        },
        {
          "newText": "@/widgets/Kid.vue",
          "range": {
            "end": {
              "character": 35,
              "line": 2
            },
            "start": {
              "character": 24,
              "line": 2
            }
          }
        }
      ]
    }
    "###);

    // Move an extensionless script target out of the alias subtree: the alias
    // has no spelling there, so the specifier falls back to a relative path.
    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&root.join("src/util.ts")),
            new_uri: file_uri(&root.join("lib/util.ts")),
        }],
        true,
    )
    .unwrap();
    assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
    {
      "src/App.vue": [
        {
          "newText": "../lib/util",
          "range": {
            "end": {
              "character": 30,
              "line": 3
            },
            "start": {
              "character": 24,
              "line": 3
            }
          }
        }
      ]
    }
    "###);
}

#[test]
fn nuxt_alias_rename_edits_use_the_open_importer_buffer() {
    let dir = test_dir();
    let root = dir.path();
    let nuxt = root.join(".nuxt");
    let pages = root.join("app/pages/[[server]]/list/[list]/index");
    let components = root.join("app/components/list");
    fs::create_dir_all(&nuxt).unwrap();
    fs::create_dir_all(&pages).unwrap();
    fs::create_dir_all(&components).unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"references":[{"path":"./.nuxt/tsconfig.app.json"}],"files":[]}"#,
    )
    .unwrap();
    fs::write(
        nuxt.join("tsconfig.app.json"),
        r#"{"compilerOptions":{"paths":{"~/*":["../app/*"]}}}"#,
    )
    .unwrap();

    let importer = pages.join("accounts.vue");
    fs::write(
        &importer,
        "<script setup lang=\"ts\">\nimport Result from '~/components/list/Original.vue'\n</script>\n",
    )
    .unwrap();
    let copied = components.join("__VizeOracleResult.vue");
    let renamed = components.join("__VizeOracleRenamedResult.vue");
    fs::write(&copied, "<template />\n").unwrap();

    let open_source = "<script setup lang=\"ts\">\nimport Result from '~/components/list/__VizeOracleResult.vue'\n</script>\n";
    let canonical_uri = tower_lsp::lsp_types::Url::from_file_path(&importer).unwrap();
    let importer_uri = tower_lsp::lsp_types::Url::parse(
        &canonical_uri
            .as_str()
            .replace('[', "%5B")
            .replace(']', "%5D"),
    )
    .unwrap();
    assert_ne!(importer_uri, canonical_uri);
    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());
    state
        .documents
        .open(importer_uri, open_source.to_owned(), 2, "vue".to_owned());

    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&copied),
            new_uri: file_uri(&renamed),
        }],
        true,
    )
    .expect("rename edit for the open authored importer");

    assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
    {
      "app/pages/[[server]]/list/[list]/index/accounts.vue": [
        {
          "newText": "~/components/list/__VizeOracleRenamedResult.vue",
          "range": {
            "end": {
              "character": 60,
              "line": 1
            },
            "start": {
              "character": 20,
              "line": 1
            }
          }
        }
      ]
    }
    "###);
}

#[test]
fn nuxt_alias_rename_edits_work_before_generated_tsconfig_exists() {
    let dir = test_dir();
    let root = dir.path();
    let pages = root.join("app/pages/[[server]]/list/[list]/index");
    let components = root.join("app/components/list");
    fs::create_dir_all(&pages).unwrap();
    fs::create_dir_all(&components).unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"references":[{"path":"./.nuxt/tsconfig.app.json"}],"files":[]}"#,
    )
    .unwrap();
    fs::write(
        root.join("nuxt.config.ts"),
        "export default defineNuxtConfig({})\n",
    )
    .unwrap();

    let importer = pages.join("accounts.vue");
    fs::write(
        &importer,
        "<script setup lang=\"ts\">\nimport Result from '~/components/list/__VizeOracleResult.vue'\n</script>\n",
    )
    .unwrap();
    let copied = components.join("__VizeOracleResult.vue");
    let renamed = components.join("__VizeOracleRenamedResult.vue");
    fs::write(&copied, "<template />\n").unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&copied),
            new_uri: file_uri(&renamed),
        }],
        true,
    )
    .expect("rename edit for Nuxt alias without generated tsconfig");

    assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
    {
      "app/pages/[[server]]/list/[list]/index/accounts.vue": [
        {
          "newText": "~/components/list/__VizeOracleRenamedResult.vue",
          "range": {
            "end": {
              "character": 60,
              "line": 1
            },
            "start": {
              "character": 20,
              "line": 1
            }
          }
        }
      ]
    }
    "###);
}

#[test]
fn project_source_alias_rename_edits_work_without_tsconfig_paths() {
    let dir = test_dir();
    let root = dir.path();
    let views = root.join("src/views");
    let components = root.join("src/components");
    fs::create_dir_all(&views).unwrap();
    fs::create_dir_all(&components).unwrap();
    fs::write(root.join("package.json"), r#"{"type":"module"}"#).unwrap();

    let importer = views.join("docs.vue");
    fs::write(
        &importer,
        "<script setup>\nimport HighlightMessage from '@/components/__VizeOracleHighlightMessage.vue'\n</script>\n",
    )
    .unwrap();
    let copied = components.join("__VizeOracleHighlightMessage.vue");
    let renamed = components.join("__VizeOracleRenamedHighlightMessage.vue");
    fs::write(&copied, "<template />\n").unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&copied),
            new_uri: file_uri(&renamed),
        }],
        true,
    )
    .expect("rename edit for project source alias without tsconfig paths");
    let normalized = normalize_edit(root, &edit);

    assert_eq!(
        normalized
            .pointer("/src~1views~1docs.vue/0/newText")
            .and_then(serde_json::Value::as_str),
        Some("@/components/__VizeOracleRenamedHighlightMessage.vue")
    );
}

#[test]
fn aliased_directory_index_specifiers_keep_the_barrel_spelling() {
    let dir = test_dir();
    let root = dir.path();
    fs::create_dir_all(root.join("src/components/Button")).unwrap();
    fs::write(
        root.join("tsconfig.json"),
        // `baseUrl` anchors the targets, so `@/*` maps to `<root>/src/*`.
        r#"{ "compilerOptions": { "baseUrl": "./src", "paths": { "@/*": ["*"] } } }"#,
    )
    .unwrap();
    fs::write(
        root.join("src/components/Button/index.ts"),
        "export const Button = 1;",
    )
    .unwrap();
    fs::write(
        root.join("src/App.vue"),
        r#"<script setup lang="ts">
import { Button } from "@/components/Button";
</script>
"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    // Moving the barrel directory: the target is still an `index.*`, so the
    // directory-index spelling survives rather than gaining an `/index` tail.
    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&root.join("src/components/Button")),
            new_uri: file_uri(&root.join("src/widgets/Button")),
        }],
        true,
    )
    .unwrap();
    assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
    {
      "src/App.vue": [
        {
          "newText": "@/widgets/Button",
          "range": {
            "end": {
              "character": 43,
              "line": 1
            },
            "start": {
              "character": 24,
              "line": 1
            }
          }
        }
      ]
    }
    "###);
}
