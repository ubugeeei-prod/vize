use std::fs;
use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::{RenameService, authored_text, prepare_range, resolve_tsgo_binary};
use crate::ide::IdeContext;
use crate::server::ServerState;

const APP_SOURCE: &str = r#"<template>
  <Child :title="greeting" />
  <Other :title="greeting" />
</template>
<script setup lang="ts">
import Child from "./Child.vue";
import Other from "./Other.vue";
const greeting = "same spelling, distinct declarations";
</script>
"#;

const CHILD_SOURCE: &str = r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template><h1>{{ title }}</h1></template>
"#;

const OTHER_SOURCE: &str = r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template><aside>{{ title }}</aside></template>
"#;

#[test]
fn canonical_prop_rename_reaches_only_the_matching_parent_usage() {
    crate::runtime::block_on(async {
        let Some(tsgo_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let root = project.path().canonicalize().expect("canonical root");
        let src = root.join("src");
        let repeated_other = "  <Other :title=\"greeting\" />\n".repeat(64);
        let app_source =
            APP_SOURCE.replace("  <Other :title=\"greeting\" />", repeated_other.as_str());
        fs::create_dir_all(&src).expect("src");
        fs::write(
            root.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .expect("tsconfig");
        fs::write(
            src.join("vue.d.ts"),
            r#"declare module "vue" { export interface ComponentPublicInstance {} }
"#,
        )
        .expect("Vue declarations");
        fs::write(src.join("App.vue"), &app_source).expect("App fixture");
        fs::write(src.join("Child.vue"), CHILD_SOURCE).expect("Child fixture");
        fs::write(src.join("Other.vue"), OTHER_SOURCE).expect("Other fixture");
        let app_uri = Url::from_file_path(src.join("App.vue")).expect("app URI");
        let child_uri = Url::from_file_path(src.join("Child.vue")).expect("child URI");
        let other_uri = Url::from_file_path(src.join("Other.vue")).expect("other URI");
        let state = ServerState::new();
        state.set_workspace_root(root.clone());
        for (uri, source) in [
            (&app_uri, app_source.as_str()),
            (&child_uri, CHILD_SOURCE),
            (&other_uri, OTHER_SOURCE),
        ] {
            state
                .documents
                .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
            state.update_virtual_docs(uri, source);
        }

        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(root),
            timeout_ms: 30_000,
            enable_profiling: true,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");
        let query_offset = CHILD_SOURCE.find("title: string").expect("title") + 1;
        let ctx = IdeContext::new(&state, &child_uri, query_offset).expect("query context");
        let prepared = RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
            .await
            .expect("prepare prop rename");
        assert_eq!(
            authored_text(CHILD_SOURCE, prepare_range(prepared)),
            "title"
        );
        let edit =
            RenameService::rename_with_corsa(&ctx, "renamedTitle", Some(Arc::clone(&bridge)))
                .await
                .expect("prop rename");
        assert_eq!(
            bridge
                .profiler()
                .get("corsa_rename_batch")
                .expect("rename batch metric")
                .count,
            1,
            "all linked rename positions must share one aggregate bridge deadline",
        );
        bridge.shutdown().await.expect("shutdown");

        let changes = edit.changes.expect("plain workspace changes");
        assert_eq!(
            changes.keys().collect::<std::collections::HashSet<_>>(),
            std::collections::HashSet::from([&app_uri, &child_uri]),
            "the unrelated Other.title must remain untouched: {changes:#?}",
        );
        let app_edits = changes.get(&app_uri).expect("parent edit");
        assert_eq!(app_edits.len(), 1, "one authored Child prop usage");
        assert_eq!(authored_text(&app_source, app_edits[0].range), "title");
        assert_eq!(app_edits[0].new_text, "renamedTitle");
        let child_edits = changes.get(&child_uri).expect("child edits");
        assert_eq!(child_edits.len(), 2, "declaration and local template use");
        assert!(child_edits.iter().all(|edit| {
            authored_text(CHILD_SOURCE, edit.range) == "title" && edit.new_text == "renamedTitle"
        }));
    });
}
