use super::*;

#[cfg(feature = "native")]
#[test]
fn component_prop_colon_completion_does_not_wait_for_a_type_project() {
    crate::runtime::block_on(async {
        let root = tempfile::tempdir().unwrap();
        let dialog = root.path().join("dialog");
        let text_field = root.path().join("text-field");
        std::fs::create_dir_all(&dialog).unwrap();
        std::fs::create_dir_all(&text_field).unwrap();
        std::fs::write(
            text_field.join("afs-text-field.vue"),
            r#"<script setup lang="ts">
defineProps<{
  modelValue: string
  placeholder?: string
  type?: string
  appendIcon?: string
  rules?: unknown[]
}>()
</script>
"#,
        )
        .unwrap();
        let source = r#"<script setup lang="ts">
import AfsTextField from "../text-field/afs-text-field.vue";
</script>
<template>
  <label>Password <span>*</span></label
  >
  <AfsTextField
    : />
</template>
"#;
        let parent = dialog.join("reported.vue");
        std::fs::write(&parent, source).unwrap();
        let uri = Url::from_file_path(parent).unwrap();
        let service = service_with_options(serde_json::json!({
            "completion": true,
            "typecheck": true,
            "crossFile": true,
            "lint": false,
            "ecosystem": false,
        }));
        let server = service.inner();
        server.state.set_workspace_root(root.path().to_path_buf());
        open_vue(server, &uri, source);

        let offset = source.find("    : />").unwrap() + "    :".len();
        let (line, character) = crate::ide::offset_to_position(source, offset);
        let mut params = completion_params(&uri);
        params.text_document_position.position = Position::new(line, character);
        params.context = Some(tower_lsp::lsp_types::CompletionContext {
            trigger_kind: tower_lsp::lsp_types::CompletionTriggerKind::TRIGGER_CHARACTER,
            trigger_character: Some(":".into()),
        });
        let _native = server.state.corsa_request_scope().await;
        let response =
            crate::runtime::timeout(std::time::Duration::from_secs(3), server.completion(params))
                .await
                .expect("component prop completion must bypass a held native transaction")
                .unwrap()
                .expect("component prop completion after colon");
        let labels: Vec<_> = match response {
            CompletionResponse::Array(items) => items.into_iter().map(|item| item.label).collect(),
            CompletionResponse::List(tower_lsp::lsp_types::CompletionList { items, .. }) => {
                items.into_iter().map(|item| item.label).collect()
            }
        };

        assert_eq!(
            labels,
            ["modelValue", "placeholder", "type", "appendIcon", "rules"]
        );
        assert_eq!(server.state.canonical_bridge_request_count(), 0);
        assert!(!server.state.has_corsa_bridge());
    });
}

#[test]
fn script_symbol_hover_survives_the_reported_multiline_template_shape() {
    let source = r#"<script setup lang="ts">
import { ref } from "vue";
const inputPassword = ref("");
</script>
<template>
  <label>Password <span>*</span></label
  >
  <AfsTextField v-model="inputPassword" />
</template>
"#;
    let service = service_with_options(options(&[("hover", true)]));
    let server = service.inner();
    let uri = uri("ReportedHover.vue");
    open_vue(server, &uri, source);
    let offset = source.find("inputPassword").unwrap() + 2;
    let (line, character) = crate::ide::offset_to_position(source, offset);
    let mut params = hover_params(&uri);
    params.text_document_position_params.position = Position::new(line, character);

    let hover = futures::executor::block_on(server.hover(params))
        .unwrap()
        .expect("script binding hover");
    assert_eq!(hover.range, None);
    let tower_lsp::lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markdown script binding hover");
    };
    assert_eq!(markup.kind, tower_lsp::lsp_types::MarkupKind::Markdown);
    assert_eq!(
        markup.value,
        "**inputPassword**\n\n_Script binding_\n\n```typescript\ninputPassword: Ref<string>\n```\n\nReactive reference created with `ref()`. Access `.value` in script, auto-unwrapped in template.\n\n**Tip**\n\nUse `inputPassword.value` to read or write this ref in script."
    );
}
