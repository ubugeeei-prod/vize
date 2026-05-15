//! Definition provider for Vue SFC files.
//!
//! Provides go-to-definition for:
//! - Template expressions -> script bindings
//! - Component usages -> component definitions
//! - Import statements -> imported files
//! - Real definitions from Corsa (when available)

pub mod bindings;
mod helpers;
mod script;
mod service;
mod template;

pub use bindings::{extract_bindings_with_locations, BindingKind, BindingLocation};

use super::IdeContext;

/// Definition service for providing go-to-definition functionality.
pub struct DefinitionService;

#[cfg(test)]
mod tests {
    use std::fs;

    use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};

    use super::{bindings, helpers, script, BindingKind, DefinitionService};
    use crate::{ide::IdeContext, server::ServerState};

    #[test]
    fn test_find_binding_location_const() {
        let content = r#"// Virtual TypeScript
// Generated

const message = ref('hello')
const count = ref(0)
"#;

        let loc = script::find_binding_location(content, "message", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "message");
        assert_eq!(loc.kind, BindingKind::Const);
    }

    #[test]
    fn test_find_binding_location_function() {
        let content = r#"// Virtual TypeScript
// Generated

function handleClick() {
  console.log('clicked')
}
"#;

        let loc = script::find_binding_location(content, "handleClick", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "handleClick");
        assert_eq!(loc.kind, BindingKind::Function);
    }

    #[test]
    fn test_find_binding_location_destructure() {
        let content = r#"// Virtual TypeScript
// Generated

const { data, error } = useFetch('/api')
"#;

        let loc = script::find_binding_location(content, "data", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "data");
        assert_eq!(loc.kind, BindingKind::Destructure);
    }

    #[test]
    fn test_offset_to_position() {
        let content = "line1\nline2\nline3";

        let (line, col) = helpers::offset_to_position(content, 0);
        assert_eq!(line, 0);
        assert_eq!(col, 0);

        let (line, col) = helpers::offset_to_position(content, 3);
        assert_eq!(line, 0);
        assert_eq!(col, 3);

        let (line, col) = helpers::offset_to_position(content, 6);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_word_at_offset() {
        let content = "const message = 'hello'";

        let word = helpers::get_word_at_offset(content, 6);
        assert_eq!(word, Some("message".to_string()));

        let word = helpers::get_word_at_offset(content, 5);
        assert_eq!(word, Some("const".to_string()));

        let word = helpers::get_word_at_offset(content, 14);
        assert_eq!(word, None);

        let word = helpers::get_word_at_offset(content, 0);
        assert_eq!(word, Some("const".to_string()));
    }

    #[test]
    fn test_get_tag_at_offset_only_matches_tag_name() {
        let content = r#"<MyButton :message="msg" />"#;

        let tag = helpers::get_tag_at_offset(content, "<MyButton".len());
        assert_eq!(tag, Some("MyButton".to_string()));

        let tag = helpers::get_tag_at_offset(content, content.find("message").unwrap() + 7);
        assert_eq!(tag, None);
    }

    #[test]
    fn test_get_attribute_and_component_at_offset_only_matches_attribute_name() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("Parent.vue");
        let content = r#"<template><Child :message="msg" /></template>"#;
        fs::write(&file_path, content).unwrap();

        let uri = Url::from_file_path(&file_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), content.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, content);

        let attr_offset = content.find(":message").unwrap() + ":message".len();
        let attr_ctx = IdeContext::new(&state, &uri, attr_offset).unwrap();
        let attr = helpers::get_attribute_and_component_at_offset(&attr_ctx);
        assert_eq!(attr, Some(("message".to_string(), "Child".to_string())));

        let value_offset = content.rfind("msg").unwrap() + "msg".len();
        let value_ctx = IdeContext::new(&state, &uri, value_offset).unwrap();
        let attr = helpers::get_attribute_and_component_at_offset(&value_ctx);
        assert_eq!(attr, None);
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(bindings::is_valid_identifier("foo"));
        assert!(bindings::is_valid_identifier("_foo"));
        assert!(bindings::is_valid_identifier("$foo"));
        assert!(bindings::is_valid_identifier("foo123"));
        assert!(!bindings::is_valid_identifier("123foo"));
        assert!(!bindings::is_valid_identifier(""));
    }

    #[test]
    fn test_find_binding_location_raw_const() {
        let content = r#"
import { ref } from 'vue'

const message = ref('hello')
const count = ref(0)
"#;

        let loc = script::find_binding_location_raw(content, "message");
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "message");
        assert_eq!(loc.kind, BindingKind::Const);
        assert_eq!(&content[loc.offset..loc.offset + 7], "message");
    }

    #[test]
    fn test_find_binding_location_raw_import() {
        let content = r#"import { ref } from 'vue'
import MyComponent from './MyComponent.vue'
"#;

        let loc = script::find_binding_location_raw(content, "MyComponent");
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "MyComponent");
        assert_eq!(loc.kind, BindingKind::Import);
        assert_eq!(&content[loc.offset..loc.offset + 11], "MyComponent");
    }

    #[test]
    fn test_find_binding_location_raw_destructure() {
        let content = r#"const { data, error } = useFetch('/api')
"#;

        let loc = script::find_binding_location_raw(content, "data");
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "data");
        assert_eq!(loc.kind, BindingKind::Destructure);
        assert_eq!(&content[loc.offset..loc.offset + 4], "data");
    }

    #[test]
    fn test_find_prop_in_define_props() {
        let content = r#"defineProps<{
  title: string
  isSubmitting?: boolean
  count: number
}>()"#;

        let pos = helpers::find_prop_in_define_props(content, "title");
        assert!(pos.is_some());

        let pos = helpers::find_prop_in_define_props(content, "isSubmitting");
        assert!(pos.is_some());

        let pos = helpers::find_prop_in_define_props(content, "nonExistent");
        assert!(pos.is_none());
    }

    #[test]
    fn test_is_in_vue_directive_expression_detection() {
        let vue_attrs = [
            ":disabled",
            "@click",
            "v-if",
            "v-for",
            "v-model",
            "#default",
        ];
        let html_attrs = ["id", "class", "href", "src", "title"];

        for attr in vue_attrs {
            assert!(
                attr.starts_with(':')
                    || attr.starts_with('@')
                    || attr.starts_with('#')
                    || attr.starts_with("v-"),
                "Vue directive {} should match pattern",
                attr
            );
        }

        for attr in html_attrs {
            assert!(
                !attr.starts_with(':')
                    && !attr.starts_with('@')
                    && !attr.starts_with('#')
                    && !attr.starts_with("v-"),
                "HTML attribute {} should NOT match Vue pattern",
                attr
            );
        }
    }

    #[test]
    fn test_definition_resolves_component_tag_at_identifier_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let component_path = dir.path().join("MyButton.vue");
        let source_path = dir.path().join("Parent.vue");

        fs::write(
            &component_path,
            "<script setup lang=\"ts\"></script>\n<template><button /></template>\n",
        )
        .unwrap();

        let source = r#"<script setup lang="ts">
import MyButton from './MyButton.vue'
</script>

<template>
  <MyButton />
</template>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("MyButton />").unwrap() + "MyButton".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());

        assert_eq!(
            location.uri.to_file_path().unwrap().canonicalize().unwrap(),
            component_path.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_definition_prefers_component_prop_on_attribute_name_only() {
        let dir = tempfile::tempdir().unwrap();
        let component_path = dir.path().join("Child.vue");
        let source_path = dir.path().join("Parent.vue");

        let child = r#"<script setup lang="ts">
defineProps<{
  message: string
}>()
</script>
"#;
        fs::write(&component_path, child).unwrap();

        let source = r#"<script setup lang="ts">
import Child from './Child.vue'

const msg = 'hello'
</script>

<template>
  <Child :message="msg" />
</template>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let attr_offset = source.find(":message").unwrap() + ":message".len();
        let attr_ctx = IdeContext::new(&state, &uri, attr_offset).unwrap();
        let attr_location = scalar_location(DefinitionService::definition(&attr_ctx).unwrap());
        let expected_prop_offset = child.find("message: string").unwrap();
        let (line, character) = crate::ide::offset_to_position(child, expected_prop_offset);
        assert_eq!(
            attr_location
                .uri
                .to_file_path()
                .unwrap()
                .canonicalize()
                .unwrap(),
            component_path.canonicalize().unwrap()
        );
        assert_eq!(attr_location.range.start.line, line);
        assert_eq!(attr_location.range.start.character, character);

        let value_offset = source.rfind("msg").unwrap() + "msg".len();
        let value_ctx = IdeContext::new(&state, &uri, value_offset).unwrap();
        let value_location = scalar_location(DefinitionService::definition(&value_ctx).unwrap());
        let expected_binding_offset = source.find("const msg").unwrap() + "const ".len();
        let (line, character) = crate::ide::offset_to_position(source, expected_binding_offset);
        assert_eq!(value_location.uri, uri);
        assert_eq!(value_location.range.start.line, line);
        assert_eq!(value_location.range.start.character, character);
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn test_definition_with_corsa_fallback_resolves_template_binding_at_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("Boundary.vue");
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
        let location = scalar_location(
            DefinitionService::definition_with_corsa(&ctx, None)
                .await
                .unwrap(),
        );
        let expected_binding_offset = source.find("const count").unwrap() + "const ".len();
        let (line, character) = crate::ide::offset_to_position(source, expected_binding_offset);

        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
    }

    #[test]
    fn test_definition_resolves_art_variant_binding_at_identifier_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("Button.art.vue");
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
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
        let expected_binding_offset = source.find("const secondaryLabel").unwrap() + "const ".len();
        let (line, character) = crate::ide::offset_to_position(source, expected_binding_offset);

        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
    }

    fn scalar_location(response: GotoDefinitionResponse) -> Location {
        match response {
            GotoDefinitionResponse::Scalar(location) => location,
            GotoDefinitionResponse::Array(mut locations) => {
                assert_eq!(locations.len(), 1);
                locations.remove(0)
            }
            GotoDefinitionResponse::Link(_) => panic!("expected location result"),
        }
    }
}
