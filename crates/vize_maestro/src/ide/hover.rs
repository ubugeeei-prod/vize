//! Hover information provider.
//!
//! Provides contextual hover information for:
//! - Template expressions and bindings
//! - Vue directives
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod backend;
mod builder;
#[cfg(feature = "native")]
mod component_import;
mod component_prop;
mod component_tag;
#[cfg(feature = "native")]
mod corsa;
#[cfg(all(test, feature = "native"))]
mod corsa_tests;
mod declaration_keyword;
#[cfg(feature = "native")]
mod html;
mod petite_vue;
mod script;
mod script_type_infer;
mod template;
#[cfg(feature = "native")]
mod v_model;

pub use builder::HoverBuilder;
#[cfg(feature = "native")]
use std::sync::Arc;
use tower_lsp::lsp_types::Hover;

use super::IdeContext;
use crate::virtual_code::{ArtCursorPosition, BlockType};
#[cfg(feature = "native")]
use vize_canon::CorsaBridge;
/// Hover service for providing contextual information.
pub struct HoverService;

impl HoverService {
    /// Get hover information for the given context.
    pub fn hover(ctx: &IdeContext) -> Option<Hover> {
        match ctx.block_type? {
            BlockType::Template => Self::hover_template(ctx),
            BlockType::Script => Self::hover_script(ctx, false),
            BlockType::ScriptSetup => Self::hover_script(ctx, true),
            BlockType::Style(index) => Self::hover_style(ctx, index),
            BlockType::Art(ArtCursorPosition::VariantTemplate(_)) => Self::hover_template(ctx),
            BlockType::Art(_) => None,
        }
    }

    /// Get hover information with Corsa support (async version).
    ///
    /// This method first tries to get type information from Corsa,
    /// then falls back to the synchronous analysis.
    #[cfg(feature = "native")]
    pub async fn hover_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        match ctx.block_type? {
            BlockType::Template => Self::hover_template_with_corsa(ctx, corsa_bridge).await,
            BlockType::Script => Self::hover_script_with_corsa(ctx, false, corsa_bridge).await,
            BlockType::ScriptSetup => Self::hover_script_with_corsa(ctx, true, corsa_bridge).await,
            BlockType::Style(index) => Self::hover_style(ctx, index),
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::hover_art_variant_with_corsa(ctx, info, corsa_bridge).await
            }
            BlockType::Art(_) => None,
        }
    }

    /// Get hover for style context.
    fn hover_style(ctx: &IdeContext, _index: usize) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue-specific CSS features
        if let Some(hover) = Self::hover_vue_css(&word) {
            return Some(hover);
        }

        None
    }

    /// Get hover for Vue CSS features.
    fn hover_vue_css(word: &str) -> Option<Hover> {
        let (title, description) = match word {
            "v-bind" => (
                "v-bind() in CSS",
                "Link CSS values to dynamic component state. The value will be compiled into a hashed CSS custom property.",
            ),
            ":deep" => (
                ":deep()",
                "Affects child component styles in scoped CSS. The selector inside `:deep()` will be compiled with the scoped attribute.",
            ),
            ":slotted" => (
                ":slotted()",
                "Target content passed via slots in scoped CSS. Only works inside scoped `<style>` blocks.",
            ),
            ":global" => (
                ":global()",
                "Apply styles globally, escaping the scoped CSS encapsulation.",
            ),
            _ => return None,
        };

        Some(
            HoverBuilder::new()
                .title(title)
                .meta("Vue SFC CSS feature")
                .description(description)
                .bullets(
                    "Behavior",
                    &[
                        "Applies during Vue SFC scoped CSS compilation.",
                        "Keep the selector explicit so the compiled output remains predictable.",
                    ],
                )
                .link(
                    "Vue SFC CSS Features",
                    "https://vuejs.org/api/sfc-css-features.html",
                )
                .build(),
        )
    }

    // =========================================================================
    // Shared utilities
    // =========================================================================

    /// Get the word at a given offset.
    pub(super) fn get_word_at_offset(content: &str, offset: usize) -> String {
        crate::ide::token_at_offset(content, offset, Self::is_word_char).unwrap_or_default()
    }

    /// Check if a byte is a valid word character.
    #[inline]
    fn is_word_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'$' || c == b':'
    }
}

#[cfg(test)]
mod tests {
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

        assert!(value.contains("message"));
        // The binding still resolves at the identifier boundary; its *type* is
        // the backend's to answer now, so the heuristic reports provenance only.
        assert!(value.contains("_Template binding_"));
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

        assert!(value.contains("message"));
        assert!(value.contains("_Template binding_"));
    }

    #[test]
    fn test_hover_infers_computed_getter_return_type() {
        // Exercises the inference directly: on a typecheck session the hover
        // surface hands type text to the backend (see `hover::backend`).
        let script = "const count = ref(0)\nconst double = computed(() => count.value * 2)\n";
        let inferred =
            HoverService::infer_type_from_script(script, "double", BindingType::SetupRef);

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
        assert!(value.starts_with("```typescript"));
        assert!(value.contains("count: number"));
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

    #[test]
    fn test_hover_petite_vue_v_scope_binding_in_expression() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("index.html");
        let source = r#"<script src="https://unpkg.com/petite-vue" defer init></script>
<div v-scope="{ count: 0, msg: 'x' }">{{ count }}</div>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "html".to_string());
        state.update_virtual_docs(&uri, source);

        // Cursor inside the `{{ count }}` interpolation expression.
        let offset = source.find("{{ count").unwrap() + "{{ co".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover(&ctx).expect("v-scope binding should produce hover");
        let value = hover_markdown(hover);

        assert!(value.contains("count"), "got {value:?}");
        assert!(
            value.contains("petite-vue scope binding"),
            "hover should label the binding as a petite-vue scope binding; got {value:?}"
        );
    }

    #[test]
    fn test_hover_petite_vue_v_scope_binding_does_not_leak_to_sibling() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("index.html");
        let source = r#"<script src="https://unpkg.com/petite-vue" defer init></script>
<span v-scope="{ count: 0 }">{{ count }}</span><p>{{ count }}</p>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "html".to_string());
        state.update_virtual_docs(&uri, source);

        // Cursor inside the sibling `<p>` interpolation, outside the v-scope subtree.
        let p_start = source.find("<p>").unwrap();
        let offset = source[p_start..].find("{{ count").unwrap() + p_start + "{{ co".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover(&ctx);

        // Either no hover, or a generic template-expression hover — but never the
        // petite-vue scope binding hover, since the binding is out of scope here.
        if let Some(hover) = hover {
            let value = hover_markdown(hover);
            assert!(
                !value.contains("petite-vue scope binding"),
                "v-scope binding must not leak to a sibling subtree; got {value:?}"
            );
        }
    }

    #[test]
    fn test_hover_supports_art_variant_binding_at_identifier_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("HoverButton.art.vue");
        let source = r#"<script setup lang="ts">
const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>{{ primaryLabel }}</Button>
  </variant>
  <variant name="Secondary">
    <Button>{{ secondaryLabel }}</Button>
  </variant>
</art>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.rfind("secondaryLabel").unwrap() + "secondaryLabel".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover(&ctx).unwrap();
        let value = hover_markdown(hover);

        assert!(value.contains("secondaryLabel"));
        assert!(value.contains("_Template expression_"));
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_hover_with_corsa_fallback_supports_identifier_boundaries() {
        crate::runtime::block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let source_path = dir.path().join("HoverBoundary.vue");
            let source = r#"<script setup lang="ts">
const count = ref(0)
</script>

<template>
  {{ count }}
</template>
"#;
            fs::write(&source_path, source).unwrap();

            let uri = Url::from_file_path(&source_path).unwrap();
            let state = ServerState::new();
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(&uri, source);

            let offset = source.rfind("count").unwrap() + "count".len();
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let hover = HoverService::hover_with_corsa(&ctx, None).await.unwrap();
            let value = hover_markdown(hover);

            assert!(value.contains("count"));
            assert!(value.contains("_Template binding_"));
        });
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_hover_with_corsa_fallback_supports_directive_boundaries() {
        crate::runtime::block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let source_path = dir.path().join("HoverDirective.vue");
            let source = r#"<template>
  <div v-if="visible" />
</template>
"#;
            fs::write(&source_path, source).unwrap();

            let uri = Url::from_file_path(&source_path).unwrap();
            let state = ServerState::new();
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(&uri, source);

            let offset = source.find("v-if").unwrap() + "v-if".len();
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let hover = HoverService::hover_with_corsa(&ctx, None).await.unwrap();
            let value = hover_markdown(hover);

            assert!(value.contains("**v-if**"));
            assert!(value.contains("Conditionally render"));
        });
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
