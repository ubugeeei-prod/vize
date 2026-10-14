//! TypeScript source actions land on authored SFC script text only.
#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_macros,
    reason = "tests assert by panicking and name fixture files with format!"
)]

use tower_lsp::lsp_types::{CodeActionKind, CodeActionOrCommand, DocumentChanges, OneOf, Url};

use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config, write_vue_test_package,
};
use crate::ide::{CodeActionService, IdeContext, position_to_offset};

const SOURCE: &str = "<script setup lang=\"ts\">
import { c, a } from \"./lib\";
import { b } from \"./lib\";
const value = a;
</script>

<template>
  <p>{{ value }} {{ c }}</p>
</template>
";

// `b` is unused; `c` is used only by the template and must stay.
const ORGANIZED: &str = "<script setup lang=\"ts\">
import { a, c } from \"./lib\";
const value = a;
</script>

<template>
  <p>{{ value }} {{ c }}</p>
</template>
";

const SORTED: &str = "<script setup lang=\"ts\">
import { a, b, c } from \"./lib\";
const value = a;
</script>

<template>
  <p>{{ value }} {{ c }}</p>
</template>
";

const UNUSED_REMOVED: &str = "<script setup lang=\"ts\">
import { c, a } from \"./lib\";
const value = a;
</script>

<template>
  <p>{{ value }} {{ c }}</p>
</template>
";

fn apply(source: &str, action: &CodeActionOrCommand) -> String {
    let CodeActionOrCommand::CodeAction(action) = action else {
        panic!("source actions are code actions, not commands");
    };
    let Some(DocumentChanges::Edits(changes)) = action
        .edit
        .as_ref()
        .and_then(|edit| edit.document_changes.clone())
    else {
        panic!("expected versioned document edits");
    };
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].text_document.version, Some(1));
    let mut text = source.to_string();
    // Edits are ordered and disjoint; apply from the end.
    for edit in changes[0].edits.iter().rev() {
        let OneOf::Left(edit) = edit else {
            panic!("annotated edits are not forwarded");
        };
        let start = position_to_offset(source, edit.range.start.line, edit.range.start.character)
            .expect("edit start in the SFC");
        let end = position_to_offset(source, edit.range.end.line, edit.range.end.character)
            .expect("edit end in the SFC");
        text.replace_range(start..end, &edit.new_text);
    }
    text
}

#[test]
fn organize_imports_keeps_template_only_imports_and_edits_authored_script_only() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required native source-action law has no configured runtime"
        );
        return;
    };
    crate::runtime::block_on(async {
        let project = tempfile::tempdir().expect("fixture root");
        let root = project.path();
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        write_vue_test_package(root);
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"bundler","target":"ES2022","noEmit":true},"include":["src/**/*"]}"#,
        )
        .expect("tsconfig");
        write_corsa_config(root, &corsa_path);
        std::fs::write(
            root.join("src/lib.ts"),
            "export const a = 1;\nexport const b = 2;\nexport const c = 3;\n",
        )
        .expect("lib");
        let app = root.join("src/App.vue");
        std::fs::write(&app, SOURCE).expect("App.vue");
        let uri = Url::from_file_path(&app).expect("App.vue URI");
        let state = state_for_fixture(root, &uri, SOURCE);
        state.load_workspace_config(root);
        let ctx = IdeContext::new(&state, &uri, 0).expect("SFC context");

        let organize = CodeActionKind::from("source.organizeImports".to_string());
        let actions =
            CodeActionService::native_source_actions(&ctx, std::slice::from_ref(&organize)).await;
        assert_eq!(actions.len(), 1, "one organize-imports action: {actions:?}");
        let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
            panic!("organize imports is a code action");
        };
        assert_eq!(action.kind.as_ref(), Some(&organize));
        assert_eq!(apply(SOURCE, &actions[0]), ORGANIZED);

        // Sorting merges and orders without removing; removal keeps order and
        // the template-only `c`. Each returns only the requested kind.
        for (kind, expected) in [
            ("source.sortImports", SORTED),
            ("source.removeUnusedImports", UNUSED_REMOVED),
        ] {
            let kind = CodeActionKind::from(kind.to_string());
            let actions =
                CodeActionService::native_source_actions(&ctx, std::slice::from_ref(&kind)).await;
            assert_eq!(actions.len(), 1, "{}: one action", kind.as_str());
            let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
                panic!("{} is a code action", kind.as_str());
            };
            assert_eq!(action.kind.as_ref(), Some(&kind));
            assert_eq!(apply(SOURCE, &actions[0]), expected, "{}", kind.as_str());
        }

        let bridge = state
            .get_corsa_bridge()
            .await
            .expect("initialized native bridge");
        bridge.shutdown().await.expect("bridge shutdown");
    });
}

// (name, authored SFC, whole SFC after organize imports)
const CASES: &[(&str, &str, &str)] = &[
    (
        "components",
        "<script setup lang=\"ts\">\nimport Kebab from \"./Kebab.vue\";\nimport Pascal from \"./Pascal.vue\";\nimport Unused from \"./Unused.vue\";\n</script>\n\n<template>\n  <Pascal />\n  <kebab />\n</template>\n",
        "<script setup lang=\"ts\">\nimport Kebab from \"./Kebab.vue\";\nimport Pascal from \"./Pascal.vue\";\n</script>\n\n<template>\n  <Pascal />\n  <kebab />\n</template>\n",
    ),
    (
        "directive-and-types",
        "<script setup lang=\"ts\">\nimport type { Shape, Unread } from \"./lib\";\nimport { vFocus, b } from \"./lib\";\nconst shape: Shape | undefined = undefined;\n</script>\n\n<template>\n  <input v-focus :title=\"String(shape)\" />\n</template>\n",
        "<script setup lang=\"ts\">\nimport type { Shape } from \"./lib\";\nimport { vFocus } from \"./lib\";\nconst shape: Shape | undefined = undefined;\n</script>\n\n<template>\n  <input v-focus :title=\"String(shape)\" />\n</template>\n",
    ),
    (
        "unicode-before-imports",
        "<script setup lang=\"ts\">\n// 日本語のコメント 🎌\nimport { c, a, b } from \"./lib\";\nconst value = a;\n</script>\n\n<template>\n  <p>{{ value }} {{ c }}</p>\n</template>\n",
        "<script setup lang=\"ts\">\n// 日本語のコメント 🎌\nimport { a, c } from \"./lib\";\nconst value = a;\n</script>\n\n<template>\n  <p>{{ value }} {{ c }}</p>\n</template>\n",
    ),
    (
        "broken-template",
        "<script setup lang=\"ts\">\nimport { c, a, b } from \"./lib\";\n</script>\n\n<template>\n  <p>{{ a </p>\n</template>\n",
        "<script setup lang=\"ts\">\nimport { a, b, c } from \"./lib\";\n</script>\n\n<template>\n  <p>{{ a </p>\n</template>\n",
    ),
];

#[test]
fn organize_imports_keeps_component_directive_and_type_reads_and_refuses_broken_templates() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required native source-action law has no configured runtime"
        );
        return;
    };
    crate::runtime::block_on(async {
        let project = tempfile::tempdir().expect("fixture root");
        let root = project.path();
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        write_vue_test_package(root);
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"bundler","target":"ES2022","noEmit":true},"include":["src/**/*"]}"#,
        )
        .expect("tsconfig");
        write_corsa_config(root, &corsa_path);
        std::fs::write(
            root.join("src/lib.ts"),
            "export const a = 1;\nexport const b = 2;\nexport const c = 3;\nexport const vFocus = {};\nexport type Shape = { x: number };\nexport type Unread = string;\n",
        )
        .expect("lib");
        for name in ["Kebab", "Pascal", "Unused"] {
            std::fs::write(
                root.join(format!("src/{name}.vue")),
                "<template><i /></template>\n",
            )
            .expect("component");
        }
        for (name, source, expected) in CASES {
            let app = root.join(format!("src/{name}.vue"));
            std::fs::write(&app, source).expect("case");
            let uri = Url::from_file_path(&app).expect("case URI");
            let state = state_for_fixture(root, &uri, source);
            state.load_workspace_config(root);
            let ctx = IdeContext::new(&state, &uri, 0).expect("SFC context");
            let organize = CodeActionKind::from("source.organizeImports".to_string());
            let actions = CodeActionService::native_source_actions(&ctx, &[organize]).await;
            assert_eq!(actions.len(), 1, "{name}: one organize-imports action");
            assert_eq!(apply(source, &actions[0]), *expected, "{name}");
            if let Some(bridge) = state.get_corsa_bridge().await {
                bridge.shutdown().await.expect("shutdown");
            }
        }
    });
}
