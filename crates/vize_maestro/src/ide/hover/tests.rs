use std::fs;

use super::{HoverBuilder, HoverService};
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::{HoverContents, Url};
use vize_relief::BindingType;

#[test]
fn test_get_word_at_offset() {
    let content = "const message = 'hello'";

    assert_eq!(HoverService::get_word_at_offset(content, 0), "const");
    assert_eq!(HoverService::get_word_at_offset(content, 6), "message");
    assert_eq!(HoverService::get_word_at_offset(content, 5), "const");
    assert_eq!(HoverService::get_word_at_offset(content, 14), "");
}

#[test]
fn test_hover_directive() {
    let hover = HoverService::hover_directive("v-if");
    assert!(hover.is_some());

    let hover = HoverService::hover_directive("unknown");
    assert!(hover.is_none());
}

#[test]
fn test_hover_vue_api() {
    let hover = HoverService::hover_vue_api("ref");
    assert!(hover.is_some());

    let hover = HoverService::hover_vue_api("unknown");
    assert!(hover.is_none());
}

#[test]
fn test_hover_template_returns_none_for_plain_text_node() {
    let source = r#"<template>
  <div>Hello world</div>
</template>
"#;
    let (state, uri) = state_with_document("PlainTextHover.vue", source);

    let offset = source.find("Hello").unwrap() + "Hello".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(HoverService::hover(&ctx).is_none());
}

#[test]
fn test_hover_template_returns_none_for_static_attribute_value() {
    let source = r#"<script setup lang="ts">
const message = 'hello'
</script>
<template>
  <div title="message" />
</template>
"#;
    let (state, uri) = state_with_document("StaticAttributeHover.vue", source);

    let offset = source.rfind("message\"").unwrap() + "message".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(HoverService::hover(&ctx).is_none());
}

#[test]
fn test_hover_template_keeps_binding_hover_in_interpolation() {
    let source = r#"<script setup lang="ts">
const message = ref('hello')
</script>
<template>
  {{ message }}
</template>
"#;
    let (state, uri) = state_with_document("InterpolationHover.vue", source);

    let offset = source.rfind("message").unwrap() + "message".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).unwrap();
    let value = hover_markdown(hover);

    // The binding still resolves at the identifier boundary; its *type* is
    // the backend's to answer now, so the heuristic reports provenance only.
    assert_eq!(
        value,
        "**message**\n\n_Template binding_\n\nBinding from `<script setup>`.\n\n**Behavior**\n- Available directly in the template scope.\n- Vue automatically unwraps refs when rendering templates."
    );
}

#[test]
fn test_hover_template_keeps_binding_hover_in_dynamic_attribute() {
    let source = r#"<script setup lang="ts">
const message = ref('hello')
</script>
<template>
  <div :title = "message" />
</template>
"#;
    let (state, uri) = state_with_document("DynamicAttributeHover.vue", source);

    let offset = source.rfind("message").unwrap() + "message".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).unwrap();
    let value = hover_markdown(hover);

    assert_eq!(
        value,
        "**message**\n\n_Template binding_\n\nBinding from `<script setup>`.\n\n**Behavior**\n- Available directly in the template scope.\n- Vue automatically unwraps refs when rendering templates."
    );
}

#[test]
fn test_hover_infers_computed_getter_return_type() {
    // Exercises the inference directly: on a typecheck session the hover
    // surface hands type text to the backend (see `hover::backend`).
    let script = "const count = ref(0)\nconst double = computed(() => count.value * 2)\n";
    let inferred = HoverService::infer_type_from_script(script, "double", BindingType::SetupRef);

    assert_eq!(inferred.as_deref(), Some("ComputedRef<number>"));
}

#[test]
fn test_hover_builder() {
    let hover = HoverBuilder::new()
        .title("ref")
        .code("typescript", "function ref<T>(value: T): Ref<T>")
        .description("Creates a reactive reference.")
        .link("Documentation", "https://vuejs.org")
        .build();

    if let HoverContents::Markup(content) = hover.contents {
        insta::assert_snapshot!(content.value.as_str());
    } else {
        panic!("Expected Markup content");
    }
}

#[cfg(feature = "native")]
#[test]
fn test_corsa_hover_is_decorated_for_editor_clients() {
    let hover = HoverService::convert_lsp_hover(vize_canon::LspHover {
        contents: vize_canon::LspHoverContents::String("(property) count: number".to_string()),
        range: None,
    });
    let value = hover_markdown(hover);

    // The signature is the whole body: no preamble above the fence (#3894).
    assert_eq!(value, "```typescript\n(property) count: number\n```");
}

#[cfg(feature = "native")]
#[test]
fn script_module_specifier_hover_uses_authored_text_and_range() {
    crate::runtime::block_on(async {
        let source = r#"<script setup lang="ts">
import { formatCount } from "./util";
</script>
"#;
        let (state, uri) = state_with_document("App.vue", source);
        let offset = source.find("./util").unwrap() + 3;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();

        let hover = HoverService::hover_with_corsa(&ctx, None).await.unwrap();
        let range = hover.range.unwrap();
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.character, 29);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.character, 35);

        let value = hover_markdown(hover);
        assert_eq!(value, "```typescript\nmodule \"./util\"\n```");
    });
}

#[test]
fn test_binding_type_to_ts_display() {
    assert_eq!(
        HoverService::binding_type_to_ts_display(BindingType::SetupRef),
        "Ref<unknown>"
    );
    assert_eq!(
        HoverService::binding_type_to_ts_display(BindingType::SetupReactiveConst),
        "Reactive<unknown>"
    );
    assert_eq!(
        HoverService::binding_type_to_ts_display(BindingType::Props),
        "Props"
    );
    assert_eq!(
        HoverService::binding_type_to_ts_display(BindingType::SetupConst),
        "const"
    );
}

#[test]
fn test_binding_type_to_description() {
    let desc = HoverService::binding_type_to_description(BindingType::SetupRef);
    insta::assert_snapshot!(desc);

    let desc = HoverService::binding_type_to_description(BindingType::Props);
    insta::assert_snapshot!(desc);
}

mod boundaries;
mod directives;
mod template_bindings;

fn hover_markdown(hover: tower_lsp::lsp_types::Hover) -> String {
    match hover.contents {
        HoverContents::Markup(content) => content.value,
        HoverContents::Scalar(marked) => match marked {
            tower_lsp::lsp_types::MarkedString::String(value) => value,
            tower_lsp::lsp_types::MarkedString::LanguageString(value) => value.value,
        },
        HoverContents::Array(items) => items
            .into_iter()
            .map(|item| match item {
                tower_lsp::lsp_types::MarkedString::String(value) => value,
                tower_lsp::lsp_types::MarkedString::LanguageString(value) => value.value,
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    }
}

fn state_with_document(name: &str, source: &str) -> (ServerState, Url) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join(name);
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    (state, uri)
}
