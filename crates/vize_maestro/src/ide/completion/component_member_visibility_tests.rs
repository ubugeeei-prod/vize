#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]

use std::fs;

use tower_lsp::lsp_types::Url;

use super::{CompletionService, component_props_tests::resolve_tsgo_binary};
use crate::{ide::IdeContext, server::ServerState};

/// Verify the script member route against a generated Vue 3 component that
/// also contributes an authored value member.
#[test]
fn imported_component_member_completion_hides_generated_contract_fields() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
const AuthoredChild = Object.assign(Child, { authoredMember: true as const })
AuthoredChild.
</script>
"#;
        let (raw_labels, labels) = component_member_completion_labels(
            corsa_path,
            r#"<script setup lang="ts">
defineProps<{ someMessage: string }>()
</script>
"#,
            source,
            "AuthoredChild.",
            MemberCompletionRoute::Script,
            false,
        )
        .await;

        assert_eq!(
            raw_labels
                .iter()
                .filter(|label| label.as_str() == "__vizeComponentMarker")
                .count(),
            1,
            "the checker response must prove that this is a generated component: {raw_labels:?}"
        );
        assert_eq!(
            labels
                .iter()
                .filter(|label| label.as_str() == "authoredMember")
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["authoredMember"],
            "the authored component value member must survive: {labels:?}"
        );
        assert_eq!(
            generated_member_labels(&labels),
            Vec::<&str>::new(),
            "generated component contract members leaked: {labels:?}"
        );
    });
}

/// Verify that template member access passes through the same canonical
/// visibility filter as script member access.
#[test]
fn template_component_member_completion_hides_generated_contract_fields() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template>{{ Child. }}</template>
"#;
        let (raw_labels, labels) = component_member_completion_labels(
            corsa_path,
            r#"<script setup lang="ts">
defineProps<{ someMessage: string }>()
</script>
"#,
            source,
            "Child.",
            MemberCompletionRoute::Template,
            false,
        )
        .await;

        assert_eq!(
            raw_labels
                .iter()
                .filter(|label| label.as_str() == "__vizeComponentMarker")
                .count(),
            1,
            "the raw template response must contain the generated marker: {raw_labels:?}"
        );
        assert_eq!(
            generated_member_labels(&labels),
            Vec::<&str>::new(),
            "generated component contract members leaked in a template: {labels:?}"
        );
    });
}

/// Verify that a Vue 2 emit-only contract is filtered even though it carries
/// no Vue 3 component marker and declares no slots.
#[test]
fn legacy_component_member_completion_hides_unmarked_generated_contract_fields() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
Child.
</script>
"#;
        let (raw_labels, labels) = component_member_completion_labels(
            corsa_path,
            r#"<script setup lang="ts">
defineEmits<{ save: [id: number] }>()
</script>
"#,
            source,
            "Child.",
            MemberCompletionRoute::Script,
            true,
        )
        .await;

        assert_eq!(
            raw_labels
                .iter()
                .filter(|label| label.as_str() == "__vizeComponentMarker")
                .count(),
            0,
            "the legacy control must remain unmarked: {raw_labels:?}"
        );
        assert_eq!(
            raw_labels
                .iter()
                .filter(|label| { label.strip_suffix('?').unwrap_or(label) == "__vizeEmitProps" })
                .count(),
            1,
            "the legacy checker response must carry generated emit props: {raw_labels:?}"
        );
        assert_eq!(
            generated_member_labels(&labels),
            Vec::<&str>::new(),
            "generated legacy component contract members leaked: {labels:?}"
        );
    });
}

#[derive(Clone, Copy)]
enum MemberCompletionRoute {
    Script,
    Template,
}

/// Run one checker-backed component member request and return the raw checker
/// labels beside the labels produced by the selected Maestro route.
async fn component_member_completion_labels(
    corsa_path: std::path::PathBuf,
    child_source: &str,
    parent_source: &str,
    cursor_needle: &str,
    route: MemberCompletionRoute,
    legacy_vue2: bool,
) -> (Vec<String>, Vec<String>) {
    let dir = crate::test_support::tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        dir.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    fs::write(
        src.join("vue.d.ts"),
        r#"declare module "vue" {
  export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
  export interface Ref<T = unknown> { value: T }
  export interface ShallowRef<T = unknown> extends Ref<T> {}
}
"#,
    )
    .unwrap();
    fs::write(src.join("Child.vue"), child_source).unwrap();
    let parent_path = src.join("Parent.vue");
    fs::write(&parent_path, parent_source).unwrap();

    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(dir.path().to_path_buf());
    state.set_legacy_vue2_for_test(legacy_vue2);
    assert_eq!(
        state.legacy_vue2_enabled(),
        legacy_vue2,
        "the checker-backed control must use its requested Vue dialect"
    );
    state
        .documents
        .open(uri.clone(), parent_source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, parent_source);

    let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
        vize_canon::CorsaBridgeConfig {
            corsa_path: Some(corsa_path),
            working_dir: Some(dir.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        },
    ));
    bridge.spawn().await.unwrap();

    let offset = parent_source.rfind(cursor_needle).unwrap() + cursor_needle.len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let document = crate::ide::corsa_support::open_canonical_virtual_document(&ctx, &bridge)
        .await
        .unwrap();
    let (line, character) =
        crate::ide::corsa_support::canonical_source_offset_to_position(&document, ctx.offset)
            .unwrap();
    let raw_labels = CompletionService::request_resolvable(
        &ctx,
        &bridge,
        &document.request_uri,
        line,
        character,
    )
    .await
    .into_iter()
    .map(|item| item.label)
    .collect::<Vec<_>>();
    let labels = match route {
        MemberCompletionRoute::Script => CompletionService::request_canonical(&ctx, &bridge)
            .await
            .unwrap(),
        MemberCompletionRoute::Template => {
            super::service_corsa_template::complete(&ctx, &bridge).await
        }
    }
    .into_iter()
    .map(|item| item.label)
    .collect::<Vec<_>>();
    let _ = bridge.shutdown().await;

    (raw_labels, labels)
}

/// Select generated private labels outside assertion macros so Davinci's
/// assertions remain exact full-vector comparisons.
fn generated_member_labels(labels: &[String]) -> Vec<&str> {
    labels
        .iter()
        .map(String::as_str)
        .filter(|label| label.starts_with("__vize"))
        .collect()
}
