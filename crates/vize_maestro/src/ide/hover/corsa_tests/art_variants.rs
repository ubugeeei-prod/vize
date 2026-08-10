use super::*;

#[test]
fn hover_with_corsa_resolves_art_variant_script_callable() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
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
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();
        fs::write(
            src.join("format.ts"),
            "export function imported(value: number, radix: number): string { return value.toString(radix) }\n",
        )
        .unwrap();

        let source = r#"<script setup lang="ts">
import { imported } from './format'
function format(value: string, precision: number): string {
  return value.repeat(precision)
}
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary">
    <p>{{ format('art', 2) }} {{ imported(10, 16) }}</p>
  </variant>
</art>
"#;
        let source_path = src.join("Button.art.vue");
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();

        for (marker, expected) in [
            (
                "format('art'",
                ["format", "value: string", "precision: number"],
            ),
            (
                "imported(10",
                ["imported", "value: number", "radix: number"],
            ),
        ] {
            let offset = source.rfind(marker).unwrap() + 2;
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let template = ctx
                .virtual_docs
                .as_ref()
                .and_then(|documents| documents.art_template(0))
                .expect("typed art template");
            assert!(
                template
                    .content
                    .contains("import { imported } from './format'"),
                "typed art template lost workspace import:\n{}",
                template.content
            );
            let generated_offset = template
                .source_map
                .to_generated(offset as u32)
                .expect("authored art cursor mapping") as usize;
            let authored_word = marker.split(['(', '\'']).next().unwrap();
            assert_eq!(
                &template.content[generated_offset - 2..generated_offset - 2 + authored_word.len()],
                authored_word,
                "art cursor mapped to the wrong generated token:\n{}",
                template.content
            );
            let hover = HoverService::hover_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .expect("art callable hover from tsgo");
            let range = hover.range.expect("art hover range");
            let range_start =
                crate::ide::position_to_offset(source, range.start.line, range.start.character)
                    .expect("art hover range start");
            let range_end =
                crate::ide::position_to_offset(source, range.end.line, range.end.character)
                    .expect("art hover range end");
            assert_eq!(
                &source[range_start..range_end],
                authored_word,
                "art hover range did not map back to the authored token"
            );
            let value = hover_markdown(hover);
            for part in expected {
                assert!(value.contains(part), "missing {part:?}: {value}");
            }
        }

        let changed_source = source.replace("precision: number", "precision: bigint");
        fs::write(&source_path, &changed_source).unwrap();
        assert!(state.documents.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: changed_source.clone(),
            }],
            2,
        ));
        state.update_virtual_docs(&uri, &changed_source);
        let changed_offset = changed_source.rfind("format('art'").unwrap() + 2;
        let changed_ctx = IdeContext::new(&state, &uri, changed_offset).unwrap();
        let changed_hover = HoverService::hover_with_corsa(&changed_ctx, Some(bridge.clone()))
            .await
            .expect("edited art callable hover from tsgo");
        let changed_value = hover_markdown(changed_hover);
        assert!(
            changed_value.contains("precision: bigint"),
            "art cache did not refresh after an edit: {changed_value}"
        );

        let inline_source = r#"<script setup lang="ts">
function inlineFormat(value: boolean, count: number): string {
  return String(value).repeat(count)
}
</script>

<template><main /></template>

<art>
  <variant name="Inline">
    <p>{{ inlineFormat(true, 2) }}</p>
  </variant>
</art>
"#;
        let inline_path = src.join("InlineArt.vue");
        fs::write(&inline_path, inline_source).unwrap();
        let inline_uri = Url::from_file_path(&inline_path).unwrap();
        state.documents.open(
            inline_uri.clone(),
            inline_source.to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(&inline_uri, inline_source);
        let inline_offset = inline_source.rfind("inlineFormat(true").unwrap() + 2;
        let inline_ctx = IdeContext::new(&state, &inline_uri, inline_offset).unwrap();
        let inline_hover = HoverService::hover_with_corsa(&inline_ctx, Some(bridge.clone()))
            .await
            .expect("inline art callable hover from tsgo");
        let inline_value = hover_markdown(inline_hover);
        for part in ["inlineFormat", "value: boolean", "count: number"] {
            assert!(
                inline_value.contains(part),
                "missing {part:?}: {inline_value}"
            );
        }

        for (name, language_id, source, marker, expected) in [
            (
                "SharedArt.art.vue",
                "art-vue",
                r#"<script setup lang="ts" isolate="false">
function sharedFormat(value: boolean, count: number): string {
  return String(value).repeat(count)
}
</script>
<art title="Shared">
  <variant name="Shared">
    <p>{{ sharedFormat(true, 2) }}</p>
  </variant>
</art>
"#,
                "sharedFormat(true",
                ["sharedFormat", "value: boolean", "count: number"],
            ),
            (
                "RegularArt.art.vue",
                "art-vue",
                r#"<script lang="ts">
export function regularFormat(value: Date, locale: string): string {
  return value.toLocaleDateString(locale)
}
</script>
<art title="Regular">
  <variant name="Regular">
    <p>{{ regularFormat(new Date(), 'en') }}</p>
  </variant>
</art>
"#,
                "regularFormat(new",
                ["regularFormat", "value: Date", "locale: string"],
            ),
            (
                "MacroArt.art.vue",
                "art-vue",
                r#"<script setup lang="ts">
const props = defineProps<{
  formatter: (value: number) => string
}>()
</script>
<art title="Macro">
  <variant name="Macro">
    <p>{{ props.formatter(1) }}</p>
  </variant>
</art>
"#,
                "formatter(1",
                ["formatter", "value: number", "string"],
            ),
            (
                "GenericArt.art.vue",
                "art-vue",
                r#"<script setup lang="ts" generic="T extends string">
const selected = await Promise.resolve('ready' as T)
function genericFormat(value: T): T { return value }
</script>
<art title="Generic">
  <variant name="Generic">
    <p>{{ genericFormat(selected) }}</p>
  </variant>
</art>
"#,
                "genericFormat(selected",
                ["genericFormat", "value: T", "): T"],
            ),
        ] {
            let path = src.join(name);
            fs::write(&path, source).unwrap();
            let uri = Url::from_file_path(&path).unwrap();
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, language_id.to_string());
            state.update_virtual_docs(&uri, source);
            let offset = source.rfind(marker).unwrap() + 2;
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let template = ctx
                .virtual_docs
                .as_ref()
                .and_then(|documents| documents.art_template(0))
                .expect("typed art template");
            let hover = HoverService::hover_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .unwrap_or_else(|| {
                    panic!(
                        "art script callable hover from tsgo: {name}\n{}",
                        template.content
                    )
                });
            let value = hover_markdown(hover);
            for part in expected {
                assert!(value.contains(part), "missing {part:?}: {value}");
            }
        }

        let completion_source = r#"<script setup lang="ts">
const props = defineProps<{
  formatter: (value: number) => string
  label: string
}>()
</script>
<art title="Completion">
  <variant name="Completion">
    <p>{{ props. }}</p>
  </variant>
</art>
"#;
        let completion_path = src.join("CompletionArt.art.vue");
        fs::write(&completion_path, completion_source).unwrap();
        let completion_uri = Url::from_file_path(&completion_path).unwrap();
        state.documents.open(
            completion_uri.clone(),
            completion_source.to_string(),
            1,
            "art-vue".to_string(),
        );
        state.update_virtual_docs(&completion_uri, completion_source);
        let completion_offset = completion_source.rfind("props.").unwrap() + "props.".len();
        let completion_ctx = IdeContext::new(&state, &completion_uri, completion_offset).unwrap();
        let info = match completion_ctx.block_type {
            Some(crate::virtual_code::BlockType::Art(
                crate::virtual_code::ArtCursorPosition::VariantTemplate(info),
            )) => info,
            other => panic!("expected art variant completion context, got {other:?}"),
        };
        let completion_template = completion_ctx
            .virtual_docs
            .as_ref()
            .and_then(|documents| documents.art_template(info.variant_index))
            .expect("typed completion art template");
        assert!(
            crate::ide::corsa_support::completion_source_offset_to_generated(
                completion_template,
                completion_offset as u32,
            )
            .is_some(),
            "art completion cursor did not map at the expression boundary:\n{}",
            completion_template.content
        );
        let items = CompletionService::complete_art_variant_with_corsa(
            &completion_ctx,
            &info,
            bridge.as_ref(),
        )
        .await;
        for expected in ["formatter", "label"] {
            assert!(
                items.iter().any(|item| item.label == expected),
                "missing typed art completion {expected:?}: {items:#?}"
            );
        }

        bridge.shutdown().await.unwrap();
    });
}
