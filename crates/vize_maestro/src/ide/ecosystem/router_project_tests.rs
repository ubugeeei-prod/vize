//! Route-name completion read off the Vue Router provider (P4-10a).

use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Url};

use crate::ide::IdeContext;
use crate::ide::completion::CompletionService;
use crate::server::ServerState;

fn completion_items(response: CompletionResponse) -> Vec<tower_lsp::lsp_types::CompletionItem> {
    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
}

#[test]
fn test_route_name_completion_reads_the_vue_router_provider_groups() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("package.json"), "{}").unwrap();
    fs::create_dir_all(dir.path().join("src/router")).unwrap();
    fs::create_dir_all(dir.path().join("src/components")).unwrap();
    fs::write(
        dir.path().join("src/router/index.ts"),
        r#"import { createRouter, createWebHistory } from "vue-router";
export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "home" },
    { path: "/users/:userId", children: [{ path: "posts/:postId", name: "user-post" }] },
  ],
});
"#,
    )
    .unwrap();
    let source_path = dir.path().join("src/components/Nav.vue");
    let source = r#"<script setup lang="ts">
definePage({ name: "home" })
router.push({ name: "" })
</script>
<route lang="json">
{ "name": "legacy" }
</route>
"#;
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({ "ecosystem": true })));
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let offset = source.find("name: \"\"").unwrap() + "name: \"".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let items = completion_items(CompletionService::complete(&ctx).unwrap());
    let rows: Vec<_> = items
        .iter()
        .map(|item| (item.label.as_str(), item.detail.as_deref().unwrap_or("")))
        .collect();

    assert_eq!(
        rows,
        vec![
            ("home", "Vue Router route `/`"),
            (
                "user-post",
                "Vue Router route `/users/:userId/posts/:postId`"
            ),
            ("legacy", "Vue Router route name"),
        ]
    );
}
