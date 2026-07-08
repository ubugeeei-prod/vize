#![allow(clippy::disallowed_macros)]

use std::sync::Arc;

use tower_lsp::lsp_types::Hover;
use vize_canon::CorsaBridge;

use super::{HoverBuilder, HoverService};
use crate::ide::IdeContext;

impl HoverService {
    pub(super) fn hover_native_dom_attribute(ctx: &IdeContext<'_>) -> Option<Hover> {
        let (attr_name, tag_name) =
            crate::ide::definition::helpers::get_attribute_and_component_at_offset(ctx)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let info = crate::ide::corsa_support::native_dom_attribute_info(&tag_name, &attr_name)?;
        let signature = format!("{}: {}", info.property_name, info.type_expression);
        let value_kind = if info.is_boolean {
            "Boolean HTML attribute"
        } else {
            "DOM reflected attribute"
        };
        let example = native_attribute_example(&tag_name, &attr_name, info.is_boolean);

        Some(
            HoverBuilder::new()
                .title(&format!("{attr_name} on <{tag_name}>"))
                .meta(info.category)
                .code("typescript", &signature)
                .description(
                    "Native DOM attribute recognized by the Vue template compiler. Vue patches it on the platform element rather than resolving it as a component prop.",
                )
                .section("Runtime surface", value_kind)
                .bullets(
                    "Editor behavior",
                    &[
                        "Hover and go-to-definition use the DOM library type for the reflected property when the type service is available.",
                        "Component prop lookup is skipped because the enclosing tag is a native element.",
                    ],
                )
                .example("vue", &example)
                .docs("MDN reference", &info.documentation_url)
                .build(),
        )
    }

    pub(super) fn hover_native_dom_tag(ctx: &IdeContext<'_>) -> Option<Hover> {
        let tag_name =
            crate::ide::definition::helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let info = crate::ide::corsa_support::native_dom_tag_info(&tag_name)?;
        let signature = format!("const element: {}", info.type_expression);

        Some(
            HoverBuilder::new()
                .title(&format!("<{tag_name}>"))
                .meta(info.category)
                .code("typescript", &signature)
                .description(
                    "Native DOM element recognized by the Vue template compiler. It is emitted as an element node, not resolved as a component.",
                )
                .bullets(
                    "Editor behavior",
                    &[
                        "Go-to-definition uses TypeScript DOM lib data when the type service is available.",
                        "Component resolution is skipped for this tag because it is part of the platform DOM surface.",
                    ],
                )
                .example("vue", &native_tag_example(&tag_name))
                .docs("MDN reference", &info.documentation_url)
                .build(),
        )
    }

    pub(super) async fn hover_html_attribute_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let (attr_name, tag_name) =
            crate::ide::definition::helpers::get_attribute_and_component_at_offset(ctx)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc =
            crate::ide::corsa_support::html_attribute_virtual_document(&tag_name, &attr_name)?;
        let request_path = crate::ide::corsa_support::html_attribute_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.hover_offset);
        let hover = bridge.hover(&request_uri, line, character).await.ok()??;

        Some(Self::convert_lsp_hover(hover))
    }

    pub(super) async fn hover_html_tag_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let tag_name =
            crate::ide::definition::helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc = crate::ide::corsa_support::html_tag_virtual_document(&tag_name)?;
        let request_path = crate::ide::corsa_support::html_tag_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.hover_offset);
        let hover = bridge.hover(&request_uri, line, character).await.ok()??;

        Some(Self::convert_lsp_hover(hover))
    }
}

fn native_attribute_example(tag_name: &str, attr_name: &str, is_boolean: bool) -> String {
    if is_boolean {
        format!("<template>\n  <{tag_name} {attr_name}>...</{tag_name}>\n</template>")
    } else {
        format!("<template>\n  <{tag_name} {attr_name}=\"...\">...</{tag_name}>\n</template>")
    }
}

fn native_tag_example(tag_name: &str) -> String {
    format!("<template>\n  <{tag_name}>...</{tag_name}>\n</template>")
}

#[cfg(test)]
mod tests {
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
}
