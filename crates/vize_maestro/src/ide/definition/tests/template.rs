use std::fs;

use tower_lsp::lsp_types::Url;

use super::super::DefinitionService;
use super::scalar_location;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn test_definition_resolves_component_tag_at_identifier_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let component_path = dir.path().join("MyButton.vue");
    let source_path = dir.path().join("Parent.vue");

    fs::write(
        &component_path,
        "<script setup lang=\"ts\"></script>\n<template><button></button></template>\n",
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
fn test_definition_resolves_kebab_tag_for_camel_case_import() {
    // Vue resolves `<description-item>` against a `descriptionItem` script-setup
    // import through camelize; the jump must work without a PascalCase local.
    let dir = tempfile::tempdir().unwrap();
    let component_path = dir.path().join("DescriptionItem.vue");
    let source_path = dir.path().join("Parent.vue");

    fs::write(
        &component_path,
        "<script setup lang=\"ts\"></script>\n<template><div></div></template>\n",
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import descriptionItem from './DescriptionItem.vue'
</script>

<template>
  <description-item />
</template>
"#;
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let offset = source.find("<description-item />").unwrap() + "<desc".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let location = scalar_location(DefinitionService::definition(&ctx).unwrap());

    assert_eq!(
        location.uri.to_file_path().unwrap().canonicalize().unwrap(),
        component_path.canonicalize().unwrap()
    );
}

#[test]
fn test_definition_resolves_define_art_source() {
    let dir = tempfile::tempdir().unwrap();
    let component_path = dir.path().join("Button.vue");
    let source_path = dir.path().join("Button.art.vue");

    fs::write(&component_path, "<template><button></button></template>\n").unwrap();

    let source = r#"<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
});
</script>

<art>
  <variant name="Default">
    <Button />
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

    let offset = source.find("Button.vue").unwrap() + "Button".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let location = scalar_location(DefinitionService::definition(&ctx).unwrap());

    assert_eq!(
        location.uri.to_file_path().unwrap().canonicalize().unwrap(),
        component_path.canonicalize().unwrap()
    );
    assert_eq!(location.range.start.line, 0);
    assert_eq!(location.range.start.character, 0);
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

#[test]
fn test_definition_ignores_static_attribute_value() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("StaticAttribute.vue");
    let source = r#"<script setup lang="ts">
const message = 'hello'
</script>

<template>
  <div title="message" />
</template>
"#;
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let value_offset = source.rfind("message\"").unwrap() + "message".len();
    let value_ctx = IdeContext::new(&state, &uri, value_offset).unwrap();

    assert!(DefinitionService::definition(&value_ctx).is_none());
}

#[cfg(feature = "native")]
#[test]
fn test_definition_with_corsa_fallback_resolves_template_binding_at_boundary() {
    crate::runtime::block_on(async {
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
    });
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
