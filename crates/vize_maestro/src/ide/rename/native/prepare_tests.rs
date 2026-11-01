//! Exact current tokens and whole native enum packets for private prepare.

use std::{fs, path::Path, sync::Arc};
use tower_lsp::lsp_types::{PrepareRenameResponse, Range, TextDocumentContentChangeEvent, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::{ArtCursorPosition, BlockType, IdeContext, RenameService, canonical, corsa};
use crate::server::ServerState;

const LOCAL: &str = "<script setup lang=\"ts\">import { ref } from 'vue';\nconst message = ref('hello');\nconst broken: string = 42; void broken;</script>\n<template>🚀 {{ message }} {{ message }}</template>";
const PUBLIC: &str = "<script setup lang=\"ts\">import Child from './Child.vue'; const message = 'local';</script><template><Child :message=\"message\" /></template>";
const CHILD: &str = "<script setup lang=\"ts\">defineProps<{ message: string }>();</script><template>{{ message }}</template>";

fn project(source: &str) -> (tempfile::TempDir, Url) {
    let directory = crate::test_support::tempdir().expect("physical project");
    for (name, contents) in [
        ("App.vue", source),
        ("Child.vue", CHILD),
        ("shared.ts", "export const message = 1;"),
        (
            "tsconfig.json",
            r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["./**/*"]}"#,
        ),
    ] {
        fs::write(directory.path().join(name), contents).expect("authored input");
    }
    let uri = Url::from_file_path(directory.path().join("App.vue")).expect("source URI");
    (directory, uri)
}

fn opened(root: &Path, uri: &Url, source: &str) -> ServerState {
    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());
    let child = Url::from_file_path(root.join("Child.vue")).expect("child URI");
    for (uri, source) in [(uri, source), (&child, CHILD)] {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        state.update_virtual_docs(uri, source);
    }
    state
}

fn expected(source: &str, start: usize) -> PrepareRenameResponse {
    let (line, character) = crate::ide::offset_to_position(source, start);
    let (end_line, end_character) = crate::ide::offset_to_position(source, start + "message".len());
    PrepareRenameResponse::Range(Range::new(
        tower_lsp::lsp_types::Position::new(line, character),
        tower_lsp::lsp_types::Position::new(end_line, end_character),
    ))
}

#[test]
fn private_prepare_proves_the_unique_half_open_current_token() {
    for source in [
        LOCAL.to_string(),
        LOCAL.replace('\n', "\r\n"),
        PUBLIC.into(),
    ] {
        let (directory, uri) = project(&source);
        let state = opened(directory.path(), &uri, &source);
        for (start, _) in source.match_indices("message") {
            let public_key = source
                .get(start..)
                .is_some_and(|tail| tail.starts_with("message="));
            for delta in [0, 1, "message".len() - 1, "message".len()] {
                let ctx = IdeContext::new(&state, &uri, start + delta).expect("token context");
                let wanted =
                    (!public_key && delta < "message".len()).then(|| expected(&source, start));
                assert_eq!(
                    RenameService::private_setup_prepare(&ctx),
                    wanted,
                    "{source}; {start}+{delta}"
                );
            }
        }
        let ctx = IdeContext::new(&state, &uri, source.find("🚀").unwrap_or(0))
            .expect("non-token context");
        assert_eq!(RenameService::private_setup_prepare(&ctx), None);
        assert!(!state.has_corsa_bridge());
    }
}

fn negatives() -> Vec<(std::string::String, &'static str)> {
    [
        ("import { message } from './shared';", "{{ message }}", "", "message }"),
        ("const { message } = defineProps<{ message: string }>();", "{{ message }}", "", "message }"),
        ("const message = defineModel<string>();", "{{ message }}", "", "message ="),
        ("const message = 1; defineExpose({ message });", "{{ message }}", "", "message ="),
        ("const message = 1; defineEmits<{ save: [] }>();", "{{ message }}", "", "message ="),
        ("const message = 1; defineSlots<{ default(): unknown }>();", "{{ message }}", "", "message ="),
        ("const message = 1; defineOptions({ name: 'Public' });", "{{ message }}", "", "message ="),
        ("const message = 1; withDefaults(defineProps<{ x?: number }>(), { x: 1 });", "{{ message }}", "", "message ="),
        ("export const message = 1;", "{{ message }}", "", "message ="),
        ("const message = 1; export { message };", "{{ message }}", "", "message ="),
        ("const { message } = store;", "{{ message }}", "", "message }"),
        ("const message = 1; const value = { message };", "{{ message }}", "", "message ="),
        ("const message = 1; const value = { message: 2 }; void value.message;", "{{ message }}", "", "message;"),
        ("const message = 1; const value = { ['message']: 2 };", "{{ message }}", "", "message']"),
        ("const message = 1; const f = () => message;", "{{ message }}", "", "message ="),
        ("const message = 1; const f = (message: number) => message;", "{{ message }}", "", "message:"),
        ("let message = 1;", "{{ message }}", "", "message ="),
        ("const message = 1; message = 2;", "{{ message }}", "", "message ="),
        ("const message = 1; message++;", "{{ message }}", "", "message ="),
        ("const message = 1;", "<Child :message />", "", "message />"),
        ("const message = 1;", "<Child :message=\"message\" />", "", "message="),
        ("const message = 1;", "<i v-for=\"message in [1]\">{{ message }}</i>", "", "message in"),
        ("const message = ;", "{{ message }}", "", "message ="),
        ("const message = 1;", "{{ message }}", "<script>export default {}</script>", "message ="),
        ("const message = 1;", "{{ message }}", "<style>.x{color:v-bind(message)}</style>", "message ="),
        ("const message = 1;", "{{ message }}", "<art title=\"A\" component=\"./Child.vue\"><variant name=\"V\">{{ message }}</variant></art>", "message ="),
    ].into_iter().map(|(script, template, suffix, query)| {
        (format!("<script setup lang=\"ts\">{script}</script><template>{template}</template>{suffix}"), query)
    }).collect()
}

#[test]
fn private_prepare_preserves_public_uncertain_and_enum_fallbacks() {
    for (source, query) in negatives() {
        let (directory, uri) = project(&source);
        let state = opened(directory.path(), &uri, &source);
        let ctx =
            IdeContext::new(&state, &uri, source.find(query).expect("query") + 1).expect("context");
        assert_eq!(
            RenameService::private_setup_prepare(&ctx),
            None,
            "{source}; {query}"
        );
    }
    // Exercise the actual unchanged mapper on a public prop fallback.
    let source = "<script setup lang=\"ts\">const { message } = defineProps<{ message: string }>();</script><template>{{ message }}</template>";
    let (directory, uri) = project(source);
    let state = opened(directory.path(), &uri, source);
    let offset = source.rfind("message").expect("public template value");
    let ctx = IdeContext::new(&state, &uri, offset).expect("public context");
    assert_eq!(RenameService::private_setup_prepare(&ctx), None);
    let template = ctx
        .virtual_docs
        .as_ref()
        .expect("documents")
        .template
        .as_ref()
        .expect("template");
    let generated = template
        .source_map
        .to_generated(offset)
        .expect("generated token");
    let (line, character) = crate::ide::offset_to_position(&template.content, generated);
    let range = Range::new(
        tower_lsp::lsp_types::Position::new(line, character),
        tower_lsp::lsp_types::Position::new(line, character + 7),
    );
    let request = corsa::template_request_path(&uri);
    for response in [
        PrepareRenameResponse::Range(range),
        PrepareRenameResponse::RangeWithPlaceholder {
            range,
            placeholder: "generated placeholder".into(),
        },
    ] {
        assert_eq!(
            corsa::map_corsa_prepare_rename(&ctx, &request, response),
            Some(expected(source, offset))
        );
    }
    for default_behavior in [true, false] {
        let response = PrepareRenameResponse::DefaultBehavior { default_behavior };
        assert_eq!(
            corsa::map_corsa_prepare_rename(&ctx, &request, response.clone()),
            Some(response)
        );
    }
    let invalid = Range::new(
        tower_lsp::lsp_types::Position::new(u32::MAX, 0),
        tower_lsp::lsp_types::Position::new(u32::MAX, 7),
    );
    assert_eq!(
        corsa::map_corsa_prepare_rename(&ctx, &request, PrepareRenameResponse::Range(invalid)),
        None
    );
}

#[test]
fn private_prepare_rechecks_edits_reopens_close_language_and_workspace_owner() {
    let (directory, uri) = project(LOCAL);
    let state = opened(directory.path(), &uri, LOCAL);
    let offset = LOCAL.find("message =").expect("declaration");
    let ctx = IdeContext::new(&state, &uri, offset).expect("old context");
    assert!(RenameService::private_setup_prepare(&ctx).is_some());
    state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: LOCAL.replace("hello", "dirty"),
        }],
        2,
    );
    assert_eq!(RenameService::private_setup_prepare(&ctx), None);
    state.documents.close(&uri);
    assert_eq!(RenameService::private_setup_prepare(&ctx), None);
    state
        .documents
        .open(uri.clone(), LOCAL.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, LOCAL);
    assert_eq!(RenameService::private_setup_prepare(&ctx), None);
    let current = IdeContext::new(&state, &uri, offset).expect("new owner");
    assert!(RenameService::private_setup_prepare(&current).is_some());
    let foreign = crate::test_support::tempdir().expect("foreign project");
    state.set_workspace_root(foreign.path().to_path_buf());
    assert_eq!(RenameService::private_setup_prepare(&current), None);
    state.set_workspace_root(directory.path().to_path_buf());
    state.documents.close(&uri);
    state
        .documents
        .open(uri.clone(), LOCAL.into(), 1, "typescript".into());
    let other_language = IdeContext::new(&state, &uri, offset).expect("other language");
    assert_eq!(RenameService::private_setup_prepare(&other_language), None);
}

async fn native_packet(
    root: &Path,
    uri: &Url,
    source: &str,
    offset: usize,
    provider: &Path,
    original: bool,
) -> Option<PrepareRenameResponse> {
    let state = opened(root, uri, source);
    let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(provider.to_path_buf()),
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    }));
    bridge.spawn().await.expect("actual pinned native owner");
    let ctx = IdeContext::new(&state, uri, offset).expect("native context");
    let packet = if original {
        RenameService::original_prepare_dispatch(&ctx, Some(Arc::clone(&bridge))).await
    } else {
        RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge))).await
    };
    bridge.shutdown().await.expect("native shutdown");
    packet
}

#[test]
fn private_prepare_matches_frozen_whole_native_response_variants() {
    let Some(provider) = crate::test_support::tsgo() else {
        assert!(
            std::env::var_os("VIZE_TEST_DISABLE_TSGO")
                .is_some_and(|value| !value.is_empty() && value != "false"),
            "native provider mandatory outside explicit PR-disabled envelope"
        );
        return;
    };
    let mut cases = vec![
        (LOCAL.to_string(), "message =", true),
        (LOCAL.to_string(), "message }}", true),
        (LOCAL.replace('\n', "\r\n"), "message =", true),
        (PUBLIC.to_string(), "message =", true),
        (PUBLIC.to_string(), "message\"", true),
        ("<script setup lang=\"ts\">const message = '日本語🌸';</script><template>{{ message }}</template>".into(), "message =", true),
    ];
    cases.extend(
        negatives()
            .into_iter()
            .map(|(source, query)| (source, query, false)),
    );
    crate::runtime::block_on(async {
        for (index, (source, query, private)) in cases.into_iter().enumerate() {
            let (directory, uri) = project(&source);
            let offset = source.find(query).expect("native query") + 1;
            let state = opened(directory.path(), &uri, &source);
            let ctx = IdeContext::new(&state, &uri, offset).expect("gate context");
            assert_eq!(
                RenameService::private_setup_prepare(&ctx).is_some(),
                private
            );
            let old = native_packet(directory.path(), &uri, &source, offset, &provider, true).await;
            let actual =
                native_packet(directory.path(), &uri, &source, offset, &provider, false).await;
            eprintln!(
                "VIZE_PRIVATE_PREPARE_NATIVE {}",
                serde_json::json!({
                    "case": index, "source": source, "query": query, "private": private,
                    "old": old, "actual": actual,
                })
            );
            if private {
                assert_eq!(
                    actual,
                    Some(expected(&source, offset - 1)),
                    "positive full Range"
                );
            }
            assert_eq!(actual, old, "whole Option<PrepareRenameResponse> {index}");
        }
    });
}

impl RenameService {
    // Complete actual-B dispatcher, with only the function name changed.
    async fn original_prepare_dispatch(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<PrepareRenameResponse> {
        match canonical::prepare(ctx, corsa_bridge.as_deref()).await {
            canonical::Answer::Available(response) => return response,
            canonical::Answer::Unavailable if corsa::is_component_attribute_query(ctx) => {
                return None;
            }
            canonical::Answer::Unavailable => {}
        }
        let corsa_result = match ctx.block_type? {
            BlockType::Template => {
                Self::prepare_template_rename_with_corsa(ctx, corsa_bridge.as_deref()).await
            }
            BlockType::Script | BlockType::ScriptSetup => {
                Self::prepare_script_rename_with_corsa(
                    ctx,
                    matches!(ctx.block_type, Some(BlockType::ScriptSetup)),
                    corsa_bridge.as_deref(),
                )
                .await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::prepare_art_variant_rename_with_corsa(ctx, info, corsa_bridge.as_deref())
                    .await
            }
            BlockType::Style(_) | BlockType::Art(_) => None,
        };

        corsa_result.or_else(|| Self::prepare_rename(ctx))
    }
}
