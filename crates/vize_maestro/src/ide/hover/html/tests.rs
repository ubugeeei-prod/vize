use std::fs;

use super::HoverService;
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::{HoverContents, Url};

#[test]
fn test_hover_template_describes_native_html_element() {
    let source = r#"<template>
  <button type="button">Save</button>
</template>
"#;
    let (state, uri) = state_with_document("NativeElementHover.vue", source);

    let offset = source.find("button").unwrap() + "but".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).unwrap();
    let value = hover_markdown(hover);

    assert!(value.contains("<button>"), "got {value:?}");
    assert!(value.contains("HTML element"), "got {value:?}");
    assert!(
        value.contains("HTMLElementTagNameMap[\"button\"]"),
        "got {value:?}"
    );
    assert!(value.contains("MDN reference"), "got {value:?}");
    assert!(value.contains("**Example**"), "got {value:?}");
    assert!(value.contains("```vue"), "got {value:?}");
}

#[test]
fn test_hover_template_describes_native_html_attribute() {
    let source = r#"<template>
  <button disabled>Save</button>
</template>
"#;
    let (state, uri) = state_with_document("NativeAttributeHover.vue", source);

    let offset = source.find("disabled").unwrap() + "disabled".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).unwrap();
    let value = hover_markdown(hover);

    assert!(value.contains("disabled on <button>"), "got {value:?}");
    assert!(value.contains("HTML attribute"), "got {value:?}");
    assert!(
        value.contains("HTMLElementTagNameMap[\"button\"][\"disabled\"]"),
        "got {value:?}"
    );
    assert!(value.contains("Boolean HTML attribute"), "got {value:?}");
    assert!(value.contains("**Example**"), "got {value:?}");
    assert!(value.contains("```vue"), "got {value:?}");
}

#[test]
fn test_hover_template_describes_native_html_bound_attribute_name() {
    let source = r#"<script setup>
const disabled = true
</script>
<template>
  <button :disabled="disabled">Save</button>
</template>
"#;
    let (state, uri) = state_with_document("NativeBoundAttributeHover.vue", source);

    let offset = source.find(":disabled").unwrap() + ":disabled".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).unwrap();
    let value = hover_markdown(hover);

    assert!(value.contains("disabled on <button>"), "got {value:?}");
    assert!(
        value.contains("HTMLElementTagNameMap[\"button\"][\"disabled\"]"),
        "got {value:?}"
    );
}

#[test]
fn test_hover_template_describes_multiline_native_html_bound_attribute_name() {
    let source = r#"<script setup>
const disabled = true
</script>
<template>
  <button
    :disabled="disabled"
  >
    Save
  </button>
</template>
"#;
    let (state, uri) = state_with_document("MultilineNativeBoundAttributeHover.vue", source);

    let offset = source.find(":disabled").unwrap() + ":disabled".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let hover = HoverService::hover(&ctx).unwrap();
    let value = hover_markdown(hover);

    assert!(value.contains("disabled on <button>"), "got {value:?}");
    assert!(
        value.contains("HTMLElementTagNameMap[\"button\"][\"disabled\"]"),
        "got {value:?}"
    );
}

#[test]
fn test_hover_template_does_not_describe_custom_element_as_native_dom() {
    let source = r#"<template>
  <my-widget />
</template>
"#;
    let (state, uri) = state_with_document("CustomElementHover.vue", source);

    let offset = source.find("my-widget").unwrap() + "my".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(HoverService::hover(&ctx).is_none());
}

#[test]
fn test_hover_template_does_not_describe_unknown_native_attribute() {
    let source = r#"<template>
  <button not-real>Save</button>
</template>
"#;
    let (state, uri) = state_with_document("UnknownNativeAttributeHover.vue", source);

    let offset = source.find("not-real").unwrap() + "not".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(HoverService::hover(&ctx).is_none());
}

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
