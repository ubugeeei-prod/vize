//! Whole native transactions remain the oracle for private rename admission.

use std::{path::Path, sync::Arc};
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url, WorkspaceEdit};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_l0::cstr;

use super::{
    ArtCursorPosition, BlockType, IdeContext, NativeRename, RenameService, canonical, corsa,
};
mod fixtures;
use fixtures::{LOCAL, opened, project};

#[test]
fn private_setup_rename_requires_the_selected_declaration_and_complete_plain_edits() {
    for source in [LOCAL.to_string(), LOCAL.replace('\n', "\r\n")] {
        let (directory, uri) = project(&source, "App.vue");
        let state = opened(directory.path(), &uri, &source);
        for (index, _) in source.match_indices("message") {
            let ctx = IdeContext::new(&state, &uri, index + 1).expect("query context");
            let edit = RenameService::private_setup_rename(&ctx, "nextMessage")
                .expect("private ref local must be admitted");
            let changes = edit.changes.as_ref().expect("whole plain packet");
            assert_eq!(changes.len(), 1);
            let edits = changes.get(&uri).expect("current source edits");
            assert_eq!(edits.len(), 3);
            assert!(edit.document_changes.is_none() && edit.change_annotations.is_none());
            for edit in edits {
                let start = crate::ide::position_to_offset(
                    &source,
                    edit.range.start.line,
                    edit.range.start.character,
                )
                .expect("start");
                let end = crate::ide::position_to_offset(
                    &source,
                    edit.range.end.line,
                    edit.range.end.character,
                )
                .expect("end");
                assert_eq!(source.get(start..end), Some("message"));
                assert_eq!(edit.new_text, "nextMessage");
            }
            assert!(
                !state.has_corsa_bridge(),
                "admission must not initialize native"
            );
        }
    }
}

#[test]
fn uncertain_or_public_rename_inputs_keep_the_original_route() {
    for (script, template, suffix, query, new_name) in [
        (
            "const props = defineProps<{ message: string }>();",
            "{{ props.message }}",
            "",
            "message:",
            "next",
        ),
        (
            "const message = 1; defineEmits<{ save: [] }>();",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1; defineSlots<{ default(): unknown }>();",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1; withDefaults(defineProps<{ x?: number }>(), { x: 1 });",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1; defineOptions({ name: 'Public' });",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "export const message = 1;",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1; export { message };",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const { message } = store;",
            "{{ message }}",
            "",
            "message }",
            "next",
        ),
        (
            "const { value: message } = store;",
            "{{ message }}",
            "",
            "message }",
            "next",
        ),
        (
            "const [message] = store;",
            "{{ message }}",
            "",
            "message]",
            "next",
        ),
        (
            "function message() {}",
            "{{ message }}",
            "",
            "message()",
            "next",
        ),
        (
            "const message = 1; const f = (message: number) => message;",
            "{{ message }}",
            "",
            "message:",
            "next",
        ),
        (
            "const message = 1;",
            "<i v-for=\"message in [1]\">{{ message }}</i>",
            "",
            "message in",
            "next",
        ),
        (
            "const message = 1;",
            "<Child :message=\"message\" />",
            "",
            "message=",
            "next",
        ),
        (
            "const message = 1;",
            "<Child :message />",
            "",
            "message />",
            "next",
        ),
        (
            "const message = 1; const value = { message };",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1; const value = { message: 2 }; void value.message;",
            "{{ message }}",
            "",
            "message;",
            "next",
        ),
        (
            "const message = 1; const value = { ['message']: 2 };",
            "{{ message }}",
            "",
            "message']",
            "next",
        ),
        (
            "const message = 1; const next = 2;",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1; const f = (next: number) => message;",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1; void next;",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "import { ref } from 'vue'; const message = ref(1);",
            "{{ message }}",
            "",
            "message =",
            "ref",
        ),
        (
            "const message = ;",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "declare const message: number;",
            "{{ message }}",
            "",
            "message:",
            "next",
        ),
        (
            "const message = 1; const message = 2;",
            "{{ message }}",
            "",
            "message =",
            "next",
        ),
        (
            "const message = 1;",
            "{{ message }}",
            "<style>.x{color:v-bind(message)}</style>",
            "message =",
            "next",
        ),
        (
            "const message = 1;",
            "{{ message }}",
            "<script>export default {}</script>",
            "message =",
            "next",
        ),
        (
            "const message = 1;",
            "{{ message }}",
            "<art title=\"A\" component=\"./Child.vue\"><variant name=\"V\">{{ message }}</variant></art>",
            "message =",
            "next",
        ),
        (
            "const message = 1;",
            "{{ message }}",
            "",
            "message =",
            "class",
        ),
        (
            "const message = 1;",
            "{{ message }}",
            "",
            "message =",
            "not-valid",
        ),
        (
            "const message = 1;",
            "{{ message }}",
            "",
            "message =",
            "message",
        ),
    ] {
        let source = cstr!(
            "<script setup lang=\"ts\">{script}</script><template>{template}</template>{suffix}"
        );
        let (directory, uri) = project(&source, "App.vue");
        let state = opened(directory.path(), &uri, &source);
        let ctx = IdeContext::new(&state, &uri, source.find(query).expect("query") + 1)
            .expect("negative context");
        assert_eq!(
            RenameService::private_setup_rename(&ctx, new_name),
            None,
            "{source}; {query}"
        );
        assert!(!state.has_corsa_bridge());
    }
}

#[test]
fn private_rename_refuses_stale_equal_version_reopens_and_foreign_owners() {
    let (directory, uri) = project(LOCAL, "App.vue");
    let state = opened(directory.path(), &uri, LOCAL);
    let ctx = IdeContext::new(&state, &uri, LOCAL.find("message =").expect("declaration"))
        .expect("old context");
    assert!(RenameService::private_setup_rename(&ctx, "next").is_some());
    state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: LOCAL.replace("hello", "dirty"),
        }],
        2,
    );
    assert_eq!(RenameService::private_setup_rename(&ctx, "next"), None);
    state.documents.close(&uri);
    state
        .documents
        .open(uri.clone(), LOCAL.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, LOCAL);
    assert_eq!(RenameService::private_setup_rename(&ctx, "next"), None);
    let current = IdeContext::new(&state, &uri, ctx.offset).expect("reopened context");
    assert!(RenameService::private_setup_rename(&current, "next").is_some());
    let foreign = crate::test_support::tempdir().expect("foreign root");
    state.set_workspace_root(foreign.path().to_path_buf());
    assert_eq!(RenameService::private_setup_rename(&current, "next"), None);
    state.set_workspace_root(directory.path().to_path_buf());
    let art_uri = Url::from_file_path(directory.path().join("App.art.vue")).expect("ART URI");
    state
        .documents
        .open(art_uri.clone(), LOCAL.into(), 1, "vue".into());
    state.update_virtual_docs(&art_uri, LOCAL);
    let art = IdeContext::new(&state, &art_uri, ctx.offset).expect("ART context");
    assert_eq!(RenameService::private_setup_rename(&art, "next"), None);
}

async fn native_packet(
    root: &Path,
    uri: &Url,
    source: &str,
    query: &str,
    new_name: &str,
    provider: &Path,
    original: bool,
) -> Option<WorkspaceEdit> {
    let state = opened(root, uri, source);
    let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(provider.to_path_buf()),
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    }));
    bridge.spawn().await.expect("actual pinned native session");
    let ctx = IdeContext::new(&state, uri, source.find(query).expect("native query") + 1)
        .expect("native context");
    let packet = if original {
        RenameService::original_native_dispatch(&ctx, new_name, Some(Arc::clone(&bridge))).await
    } else {
        RenameService::rename_with_corsa(&ctx, new_name, Some(Arc::clone(&bridge))).await
    };
    bridge.shutdown().await.expect("native shutdown");
    packet
}

#[test]
fn private_setup_rename_matches_frozen_whole_native_workspace_edits() {
    let Some(provider) = fixtures::provider() else {
        return;
    };
    crate::runtime::block_on(async {
        for (source, query, new_name, private) in fixtures::native_cases() {
            let (directory, uri) = project(&source, "App.vue");
            let state = opened(directory.path(), &uri, &source);
            let ctx = IdeContext::new(&state, &uri, source.find(query).expect("query") + 1)
                .expect("gate context");
            assert_eq!(
                RenameService::private_setup_rename(&ctx, new_name).is_some(),
                private
            );
            let old = native_packet(
                directory.path(),
                &uri,
                &source,
                query,
                new_name,
                &provider,
                true,
            )
            .await;
            let actual = native_packet(
                directory.path(),
                &uri,
                &source,
                query,
                new_name,
                &provider,
                false,
            )
            .await;
            // Preserve all containers, ranges, order, versions and annotations
            // before assertions; no counts-only or normalized packet oracle.
            eprintln!(
                "VIZE_PRIVATE_RENAME_NATIVE {}",
                serde_json::json!({
                    "source": source, "query": query, "newName": new_name,
                    "private": private, "old": old, "actual": actual,
                })
            );
            if private {
                assert!(old.is_some(), "positive native oracle must answer");
            }
            assert_eq!(actual, old, "whole native transaction: {source}; {query}");
            if private {
                let edit = actual.expect("private packet");
                let edits = edit
                    .changes
                    .expect("plain")
                    .remove(&uri)
                    .expect("current source");
                let mut applied = source.clone();
                for edit in edits.iter().rev() {
                    let start = crate::ide::position_to_offset(
                        &source,
                        edit.range.start.line,
                        edit.range.start.character,
                    )
                    .expect("start");
                    let end = crate::ide::position_to_offset(
                        &source,
                        edit.range.end.line,
                        edit.range.end.character,
                    )
                    .expect("end");
                    applied.replace_range(start..end, &edit.new_text);
                }
                assert_eq!(
                    applied,
                    fixtures::expected_private_source(&source, new_name)
                );
            }
        }
    });
}

impl RenameService {
    // Frozen original B dispatcher. Only this test function's name/header and
    // module imports differ; the complete production body is an independent oracle.
    async fn original_native_dispatch(
        ctx: &IdeContext<'_>,
        new_name: &str,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<WorkspaceEdit> {
        let (canonical_result, mut scope) =
            canonical::rename(ctx, new_name, corsa_bridge.as_deref()).await;
        match canonical_result {
            canonical::CanonicalRename::Refused => return None,
            canonical::CanonicalRename::Owned(edit) => {
                // A component argument owns the public property identity. Its
                // shorthand token also maps to a separate local value, which
                // the structural provider must not add to this transaction.
                if corsa::is_component_attribute_query(ctx) {
                    return corsa::merge_missing_authored_rename(ctx, Some(edit), None)
                        .filter(|edit| scope.admits_authored(edit));
                }
                return corsa::merge_missing_authored_rename(
                    ctx,
                    Some(edit),
                    Self::rename(ctx, new_name),
                )
                .filter(|edit| scope.admits_authored(edit));
            }
            canonical::CanonicalRename::Unavailable if corsa::is_component_attribute_query(ctx) => {
                return None;
            }
            canonical::CanonicalRename::Unavailable => {}
        }
        let corsa_result = match ctx.block_type? {
            BlockType::Template => {
                Self::rename_template_with_corsa(ctx, new_name, corsa_bridge.as_deref()).await
            }
            BlockType::Script | BlockType::ScriptSetup => {
                Self::rename_script_with_corsa(
                    ctx,
                    new_name,
                    matches!(ctx.block_type, Some(BlockType::ScriptSetup)),
                    corsa_bridge.as_deref(),
                )
                .await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::rename_art_variant_with_corsa(ctx, info, new_name, corsa_bridge.as_deref())
                    .await
            }
            BlockType::Style(_) | BlockType::Art(_) => None,
        };

        let corsa_result = match corsa_result {
            Some(NativeRename::Refused) => return None,
            Some(NativeRename::Edit(edit)) => Some(edit),
            None => None,
        };

        // Corsa only renames the virtual document the request opened, so the
        // authored edits carry the other blocks of this SFC.
        corsa::merge_authored_rename(ctx, corsa_result, Self::rename(ctx, new_name))
            .filter(|edit| corsa::RenameScope::native(ctx).admits_authored(edit))
    }
}
