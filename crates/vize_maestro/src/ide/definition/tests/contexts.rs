use std::fs;

use tower_lsp::lsp_types::Url;
use vize_s0::cstr;

use super::super::{DefinitionService, script};
use super::scalar_location;
use crate::{ide::IdeContext, server::ServerState};

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
        let needle = cstr!("{member} }}}}");
        let offset = CLASS_COMPONENT_SFC.find(needle.as_str()).unwrap();
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
