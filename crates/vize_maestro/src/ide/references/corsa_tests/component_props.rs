use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::{ReferencesService, authored_text, resolve_tsgo_binary};
use crate::ide::IdeContext;
use crate::server::ServerState;

const APP_SOURCE: &str = r#"<template>
  <section>
    <p>{{ greeting }}</p>
    <Child :title="greeting" planted-bad-prop="nope" />
  </section>
</template>

<script setup lang="ts">
import Child from "./Child.vue";

const greeting = "confirm-lsp";
</script>
"#;

const CHILD_SOURCE: &str = r#"<script setup lang="ts">
defineProps<{
  title: string;
  /** Distinctive optional prop: the completion plant looks for THIS name.
   *  `title` alone would be satisfiable by HTML's global `title` attribute. */
  epilogueText?: string;
}>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
"#;

const COLLISION_APP_SOURCE: &str = r#"<template>
  <Child :title="greeting" />
  <Other :title="greeting" />
</template>
<script setup lang="ts">
import Child from "./Child.vue";
import Other from "./Other.vue";
const greeting = "same spelling, distinct declarations";
</script>
"#;

const OTHER_SOURCE: &str = r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template><aside>{{ title }}</aside></template>
"#;

fn write_project_scaffold(project: &Path) -> PathBuf {
    fs::write(
        project.join("tsconfig.json"),
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
    let src = project.join("src");
    fs::create_dir_all(&src).expect("src");
    fs::write(
        src.join("vue.d.ts"),
        r#"declare module "vue" {
  export interface ComponentPublicInstance {}
  export interface Ref<T> { value: T }
  export interface ShallowRef<T> { value: T }
}
"#,
    )
    .expect("Vue declarations");
    src
}

#[test]
fn canonical_prop_references_reach_parent_template_usage() {
    crate::runtime::block_on(async {
        let Some(tsgo_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let src = write_project_scaffold(project.path());

        let app_path = src.join("App.vue");
        let child_path = src.join("Child.vue");
        fs::write(&app_path, APP_SOURCE).expect("App.vue");
        fs::write(&child_path, CHILD_SOURCE).expect("Child.vue");
        let app_uri = Url::from_file_path(&app_path).expect("app URI");
        let child_uri = Url::from_file_path(&child_path).expect("child URI");

        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        state
            .documents
            .open(app_uri.clone(), APP_SOURCE.to_owned(), 1, "vue".to_owned());
        state.update_virtual_docs(&app_uri, APP_SOURCE);
        state.documents.open(
            child_uri.clone(),
            CHILD_SOURCE.to_owned(),
            1,
            "vue".to_owned(),
        );
        state.update_virtual_docs(&child_uri, CHILD_SOURCE);
        assert_eq!(state.open_importers(&child_uri), vec![app_uri.clone()]);

        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let query_offset = CHILD_SOURCE
            .find("title: string")
            .expect("prop declaration")
            + 1;
        let ctx = IdeContext::new(&state, &child_uri, query_offset).expect("query context");
        let references =
            ReferencesService::references_with_corsa(&ctx, true, Some(Arc::clone(&bridge)))
                .await
                .expect("prop references");
        bridge.shutdown().await.expect("shutdown");

        let child_hits = references
            .iter()
            .filter(|location| location.uri == child_uri)
            .collect::<Vec<_>>();
        assert_eq!(
            child_hits.len(),
            2,
            "the declaration and local template use must remain mapped: {references:#?}",
        );
        assert!(
            child_hits
                .iter()
                .all(|location| authored_text(CHILD_SOURCE, location) == "title"),
            "child hits must cover the authored prop name: {child_hits:#?}",
        );

        let app_hits = references
            .iter()
            .filter(|location| location.uri == app_uri)
            .collect::<Vec<_>>();
        assert_eq!(
            app_hits.len(),
            1,
            "the prop declaration must reach the parent template attribute: {references:#?}",
        );
        assert_eq!(authored_text(APP_SOURCE, app_hits[0]), "title");
        assert!(
            references
                .iter()
                .all(|location| !location.uri.path().ends_with(".vue.ts")),
            "canonical mirror URIs must never leak: {references:#?}",
        );
    });
}

#[test]
fn canonical_prop_references_reject_other_components_with_the_same_prop_name() {
    crate::runtime::block_on(async {
        let Some(tsgo_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let src = write_project_scaffold(project.path());
        let fixtures = [
            ("App.vue", COLLISION_APP_SOURCE),
            ("Child.vue", CHILD_SOURCE),
            ("Other.vue", OTHER_SOURCE),
        ];
        for (name, source) in fixtures {
            fs::write(src.join(name), source).expect("Vue fixture");
        }
        let app_uri = Url::from_file_path(src.join("App.vue")).expect("app URI");
        let child_uri = Url::from_file_path(src.join("Child.vue")).expect("child URI");
        let other_uri = Url::from_file_path(src.join("Other.vue")).expect("other URI");

        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        for (uri, source) in [
            (&app_uri, COLLISION_APP_SOURCE),
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
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");
        let query_offset = CHILD_SOURCE.find("title: string").expect("title") + 1;
        let ctx = IdeContext::new(&state, &child_uri, query_offset).expect("query context");
        let references =
            ReferencesService::references_with_corsa(&ctx, true, Some(Arc::clone(&bridge)))
                .await
                .expect("prop references");
        bridge.shutdown().await.expect("shutdown");

        assert!(
            references.iter().all(|location| location.uri != other_uri),
            "definition identity must reject the unrelated Other.title: {references:#?}",
        );
        let app_hits = references
            .iter()
            .filter(|location| location.uri == app_uri)
            .collect::<Vec<_>>();
        assert_eq!(app_hits.len(), 1, "only Child.title belongs to the query");
        assert_eq!(authored_text(COLLISION_APP_SOURCE, app_hits[0]), "title");
        let expected_offset = COLLISION_APP_SOURCE.find(":title").expect("Child title") + 1;
        let expected = crate::ide::offset_to_position(COLLISION_APP_SOURCE, expected_offset);
        assert_eq!(
            (
                app_hits[0].range.start.line,
                app_hits[0].range.start.character,
            ),
            expected,
        );
    });
}
