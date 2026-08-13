use super::*;

#[test]
fn update_art_virtual_docs_tracks_non_default_variants_separately() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Button.art.vue").unwrap();
    let source = r#"<script setup lang="ts">
const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button :label="primaryLabel" />
  </variant>
  <variant name="Secondary">
    <Button :label="secondaryLabel" />
  </variant>
</art>
"#;

    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
    state.update_virtual_docs(&uri, source);

    let virtual_docs = state.get_virtual_docs(&uri).unwrap();
    assert_eq!(virtual_docs.art_templates.len(), 2);

    let default_template = virtual_docs.template.as_ref().unwrap();
    let secondary_template = virtual_docs.art_template(1).unwrap();

    assert!(default_template.content.contains("primaryLabel"));
    assert!(secondary_template.content.contains("secondaryLabel"));
    assert!(!secondary_template.uri.ends_with(".__template.ts"));
    assert!(
        secondary_template
            .uri
            .contains(".art_variant_1.template.ts")
    );

    let offset = source.rfind("secondaryLabel").unwrap() + 1;
    assert!(
        matches!(
            find_art_block_at_offset(source, offset),
            Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
        ) && secondary_template
            .source_map
            .to_generated(offset as u32)
            .is_some()
    );
}

#[test]
fn typed_art_template_docs_strip_loose_context_prefixes_and_keep_mappings() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Button.art.vue").unwrap();
    let source = r#"<script setup lang="ts">
const simpleLabel: string = "typed"
const props = defineProps<{ title: string }>()
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <p>{{ simpleLabel }} {{ props.title }}</p>
  </variant>
</art>
"#;

    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
    state.update_virtual_docs(&uri, source);

    let virtual_docs = state.get_virtual_docs(&uri).unwrap();
    let template = virtual_docs.art_template(0).unwrap();
    assert!(
        !template.content.contains("__VIZE_ctx."),
        "typed art template must use setup bindings directly:\n{}",
        template.content,
    );
    assert!(template.content.contains("= simpleLabel;"));
    assert!(template.content.contains("= props.title;"));

    for marker in ["simpleLabel", "props.title"] {
        let source_offset = source.rfind(marker).unwrap();
        let generated_offset = template
            .source_map
            .to_generated(source_offset as u32)
            .expect("typed art template mapping") as usize;
        assert_eq!(
            &template.content[generated_offset..generated_offset + marker.len()],
            marker,
            "mapping for {marker:?} should land on the authored expression:\n{}",
            template.content,
        );
    }
}

#[test]
fn typed_art_template_docs_keep_mappings_across_unicode_and_crlf_sources() {
    let lf_source = r#"<script setup lang="ts">
const simpleLabel: string = "型付き"
const props = defineProps<{ title: string }>()
</script>

<art title="ボタン" component="./Button.vue">
  <variant name="Primary" default>
    <p>ラベル → {{ simpleLabel }} ・ 見出し ✅ {{ props.title }}</p>
  </variant>
</art>
"#;
    let crlf_source = lf_source.replace('\n', "\r\n");

    for (endings, source) in [("lf", lf_source), ("crlf", crlf_source.as_str())] {
        let state = ServerState::new();
        let uri = Url::parse("file:///Unicode.art.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let virtual_docs = state.get_virtual_docs(&uri).unwrap();
        let template = virtual_docs.art_template(0).unwrap();
        assert!(
            !template.content.contains("__VIZE_ctx."),
            "typed art template must use setup bindings directly ({endings}):\n{}",
            template.content,
        );

        for marker in ["simpleLabel", "props.title"] {
            let source_offset = source.rfind(marker).unwrap();
            let generated_offset = template
                .source_map
                .to_generated(source_offset as u32)
                .unwrap_or_else(|| {
                    panic!(
                        "missing {endings} mapping for {marker:?}:\n{}",
                        template.content
                    )
                }) as usize;
            assert_eq!(
                &template.content[generated_offset..generated_offset + marker.len()],
                marker,
                "{endings} mapping for {marker:?} should land on the authored expression:\n{}",
                template.content,
            );
        }
    }
}

#[test]
fn update_art_virtual_docs_isolates_script_setup_per_variant() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Counter.art.vue").unwrap();
    let source = r#"<script setup lang="ts">
import { computed, ref } from "vue";
defineArt("./Counter.vue", { title: "Counter" });
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>

<art>
  <variant name="First">
    <Counter :count="doubled" />
  </variant>
  <variant name="Second">
    <Counter :count="count" />
  </variant>
</art>
"#;

    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
    state.update_virtual_docs(&uri, source);

    let virtual_docs = state.get_virtual_docs(&uri).unwrap();
    let script_setup = virtual_docs.script_setup.as_ref().unwrap();
    assert!(
        script_setup
            .content
            .contains("function __VIZE_art_variant_0_setup()")
    );
    assert!(
        script_setup
            .content
            .contains("function __VIZE_art_variant_1_setup()")
    );
    assert!(!script_setup.content.contains("defineArt"));

    let state_offset = source.find("doubled = computed").unwrap();
    assert!(
        script_setup
            .source_map
            .to_generated(state_offset as u32)
            .is_some()
    );
}

#[test]
fn update_art_virtual_docs_keeps_script_setup_shared_when_isolate_false() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Counter.art.vue").unwrap();
    let source = r#"<script setup lang="ts" isolate="false">
const count = ref(0)
</script>

<art title="Counter">
  <variant name="First">
    <Counter :count="count" />
  </variant>
  <variant name="Second">
    <Counter :count="count" />
  </variant>
</art>
"#;

    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
    state.update_virtual_docs(&uri, source);

    let virtual_docs = state.get_virtual_docs(&uri).unwrap();
    let script_setup = virtual_docs.script_setup.as_ref().unwrap();

    assert!(script_setup.content.contains("isolate=\"false\""));
    assert!(!script_setup.content.contains("__VIZE_art_variant_0_setup"));
    assert!(!script_setup.content.contains("__VIZE_art_variant_1_setup"));
}

#[test]
fn update_virtual_docs_generates_standalone_html_template_doc() {
    let state = ServerState::new();
    let uri = Url::parse("file:///index.html").unwrap();
    let source = r#"<div v-scope="{ count: 0 }">{{ count }}</div>"#;

    state.update_virtual_docs(&uri, source);

    let virtual_docs = state.get_virtual_docs(&uri).unwrap();
    let template = virtual_docs.template.as_ref().unwrap();
    assert!(template.uri.ends_with("index.html.__template.ts"));
    assert!(template.content.contains("count"));
}

#[test]
fn update_virtual_docs_removes_cache_after_sfc_parse_failure() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Broken.vue").unwrap();
    let valid_source = r#"<script setup lang="ts">
const message = 'ok'
</script>

<template>
  <div>{{ message }}</div>
</template>
"#;

    state.update_virtual_docs(&uri, valid_source);
    assert!(state.get_virtual_docs(&uri).is_some());

    state.update_virtual_docs(&uri, "<template><div></div>");
    assert!(state.get_virtual_docs(&uri).is_none());
}

#[test]
fn update_virtual_docs_removes_cache_after_art_parse_failure() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Broken.art.vue").unwrap();
    let valid_source = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button />
  </variant>
</art>
"#;

    state.update_virtual_docs(&uri, valid_source);
    assert!(state.get_virtual_docs(&uri).is_some());

    state.update_virtual_docs(&uri, "<template><div>not an art file</div></template>");
    assert!(state.get_virtual_docs(&uri).is_none());
}

#[test]
#[ignore = "requires pkl runtime installed"]
fn load_lsp_config_from_pkl() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.pkl"),
        "lsp {\n    lint = true\n    typecheck = true\n}\n",
    )
    .unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    assert!(state.is_lsp_lint_enabled());
    assert!(state.is_lsp_typecheck_enabled());
}
