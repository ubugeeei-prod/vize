//! Definition provider for Vue SFC files.
//!
//! Provides go-to-definition for:
//! - Template expressions -> script bindings
//! - Component usages -> component definitions
//! - Import statements -> imported files
//! - Real definitions from Corsa (when available)

pub mod bindings;
pub(crate) mod helpers;
pub(crate) mod script;
mod service;
mod template;

pub use bindings::{BindingKind, BindingLocation, extract_bindings_with_locations};

use super::IdeContext;

/// Definition service for providing go-to-definition functionality.
pub struct DefinitionService;

#[cfg(test)]
mod tests {
    use std::fs;

    use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};

    use super::{BindingKind, DefinitionService, bindings, helpers, script};
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

    #[test]
    fn test_definition_resolves_standalone_html_v_scope_property() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("index.html");
        let source = r#"<script src="https://unpkg.com/petite-vue" defer init></script>
<div v-scope="{ count: 0, inc() { count++ } }">
  {{ count }}
  <button @click="inc">inc</button>
</div>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "html".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.rfind("count").unwrap() + "count".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
        let expected_binding_offset = source.find("count: 0").unwrap();
        let (line, character) = crate::ide::offset_to_position(source, expected_binding_offset);

        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
    }

    #[test]
    fn test_definition_resolves_standalone_html_create_app_property() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("index.html");
        let source = r#"<script src="https://unpkg.com/petite-vue"></script>
<script>
PetiteVue.createApp({
  count: 0
}).mount()
</script>
<div v-scope>{{ count }}</div>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "html".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.rfind("count").unwrap() + "count".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
        let expected_binding_offset = source.find("count: 0").unwrap();
        let (line, character) = crate::ide::offset_to_position(source, expected_binding_offset);

        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
    }

    #[test]
    fn test_definition_in_style_resolves_inside_v_bind_argument() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("Styled.vue");
        let source = r#"<script setup lang="ts">
const color = 'red'
</script>
<style>
.foo { color: v-bind(color); }
</style>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("v-bind(color").unwrap() + "v-bind(".len() + "color".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
        let expected_binding_offset = source.find("const color").unwrap() + "const ".len();
        let (line, character) = crate::ide::offset_to_position(source, expected_binding_offset);

        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
    }

    #[test]
    fn test_definition_in_style_ignores_same_word_after_closed_v_bind() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("Styled.vue");
        let source = r#"<script setup lang="ts">
const color = 'red'
</script>
<style>
.foo { color: v-bind(color); }
.bar { background: color; }
</style>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.rfind("background: color").unwrap() + "background: ".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();

        assert!(DefinitionService::definition(&ctx).is_none());
    }

    #[test]
    fn test_definition_does_not_panic_on_non_ascii_before_identifier() {
        // Regression for #964: a non-ASCII character right before an
        // identifier used to place `word_start - 6` inside a multi-byte
        // codepoint and panic the LSP with "byte index N is not a char
        // boundary". The handler must return a normal (possibly empty)
        // response instead.
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("CJK.vue");
        let source = "<script setup lang=\"ts\">\nconst title = 'こんにちは'\n</script>\n\n<template>\n  <div>あいうえおtitle</div>\n</template>\n";
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        // Land right after the `title` identifier that is preceded by CJK.
        let offset = source.rfind("title").unwrap() + "title".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();

        // Must not panic. Returning Some or None are both acceptable.
        let _ = DefinitionService::definition(&ctx);
    }

    // A vue-class-component SFC: a decorated class default export whose
    // members (fields, getters, methods, `@Prop`s) are the template scope.
    const CLASS_COMPONENT_SFC: &str = r#"<script lang="ts">
import { Vue, Component, Prop } from 'vue-property-decorator'
@Component
export default class Counter extends Vue {
  count = 0
  @Prop() readonly title!: string
  get doubled() { return this.count * 2 }
  inc() { this.count++ }
}
</script>
<template><p>{{ count }} {{ title }} {{ doubled }} {{ inc }}</p></template>
"#;

    fn open_doc(state: &ServerState, source: &str, name: &str) -> Url {
        let dir = tempfile::tempdir().unwrap();
        // Leak the tempdir so the file outlives the call; tests are short-lived.
        let path = Box::leak(Box::new(dir)).path().join(name);
        fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        uri
    }

    /// Class-component members are auto-detected by AST shape, so go-to-definition
    /// on a class member used in the template must resolve to its class-body
    /// declaration with **no** `optionsApi` flag enabled (plain Vue 3 project).
    #[test]
    fn definition_resolves_class_component_member_in_template_without_flag() {
        let state = ServerState::new();
        let uri = open_doc(&state, CLASS_COMPONENT_SFC, "Counter.vue");

        for (member, decl) in [
            ("count", "count = 0"),
            ("doubled", "doubled()"),
            ("inc", "inc()"),
            ("title", "title!"),
        ] {
            let needle = format!("{member} }}}}");
            let offset = CLASS_COMPONENT_SFC.find(&needle).unwrap();
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let location = scalar_location(
                DefinitionService::definition(&ctx)
                    .unwrap_or_else(|| panic!("no definition for class member `{member}`")),
            );
            assert_eq!(location.uri, uri);
            let decl_offset = CLASS_COMPONENT_SFC.find(decl).unwrap();
            let (line, _) = crate::ide::offset_to_position(CLASS_COMPONENT_SFC, decl_offset);
            assert_eq!(
                location.range.start.line, line,
                "definition for `{member}` should point at its class-body declaration"
            );
        }
    }

    /// `find_analyzed_binding_location` self-gates on the optionsApi flag: with
    /// `optionsApi: false` (explicit opt-out) Options API object bindings must
    /// not resolve.
    #[test]
    fn definition_options_api_data_absent_when_opted_out() {
        let source = r#"<script>
export default {
  data() { return { greeting: 'hello' } },
}
</script>
<template><p>{{ greeting }}</p></template>
"#;
        let dir = tempfile::tempdir().unwrap();
        let dir_path = Box::leak(Box::new(dir)).path();
        fs::write(
            dir_path.join("vize.config.json"),
            r#"{ "typeChecker": { "optionsApi": false } }"#,
        )
        .unwrap();
        let path = dir_path.join("Greeting.vue");
        fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();

        let state = ServerState::new();
        state.load_workspace_config(dir_path);
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        assert!(!state.options_api_enabled());
        let offset = source.find("greeting }}").unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        assert!(
            script::find_analyzed_binding_location(&ctx, "greeting").is_none(),
            "Options API data() binding must not resolve while optionsApi is opted out"
        );
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
