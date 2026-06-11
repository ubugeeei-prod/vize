use super::ServerState;
use crate::virtual_code::{ArtCursorPosition, BlockType, find_art_block_at_offset};
use tower_lsp::lsp_types::Url;

#[cfg(feature = "native")]
#[test]
fn corsa_init_failure_is_recorded() {
    let state = ServerState::new();
    assert!(state.corsa_init_failure().is_none());
    state.record_corsa_init_failure("spawn failed: missing tsgo");
    let reason = state
        .corsa_init_failure()
        .expect("failure reason should be recorded");
    assert!(
        reason.contains("spawn failed"),
        "reason should preserve the recorded message: {reason}"
    );
}

#[test]
fn default_format_options() {
    let state = ServerState::new();
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
    assert_eq!(opts.tab_width, 2);
    assert!(!opts.use_tabs);
    assert!(opts.semi);
    assert!(!opts.single_quote);
    assert!(opts.sort_attributes);
    assert!(opts.normalize_directive_shorthands);
}

#[test]
fn load_format_config_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let state = ServerState::new();
    state.load_format_config(dir.path());
    // options remain default
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
}

#[test]
fn load_format_config_from_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "fmt": {
                    "printWidth": 80,
                    "tabWidth": 4,
                    "useTabs": true,
                    "semi": false,
                    "singleQuote": true
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 80);
    assert_eq!(opts.tab_width, 4);
    assert!(opts.use_tabs);
    assert!(!opts.semi);
    assert!(opts.single_quote);
}

#[test]
fn load_format_config_partial() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "fmt": { "printWidth": 120 } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 120);
    // defaults preserved
    assert_eq!(opts.tab_width, 2);
    assert!(opts.semi);
}

#[test]
fn load_format_config_no_fmt_section() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "check": { "globals": ["$t"] } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    // no fmt section → options remain default
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
}

#[test]
fn load_format_config_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("vize.config.json"), "not valid json").unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    // options remain default
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
}

#[test]
fn lsp_features_enable_non_opinionated_defaults() {
    let state = ServerState::new();
    let features = state.lsp_features();
    assert!(features.lint);
    assert!(features.typecheck);
    assert!(features.ecosystem);
    assert!(features.completion);
    assert!(features.code_actions);
    assert!(!features.formatting);
    assert!(state.is_lsp_lint_enabled());
    assert!(state.is_lsp_typecheck_enabled());
}

#[test]
fn load_lsp_config_from_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "lsp": {
                    "lint": true,
                    "typecheck": true,
                    "editor": true,
                    "formatting": false
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    let features = state.lsp_features();
    assert!(features.lint);
    assert!(features.typecheck);
    assert!(features.ecosystem);
    assert!(features.completion);
    assert!(features.definition);
    assert!(!features.formatting);
}

#[test]
fn load_lsp_config_updates_type_checker_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "typeChecker": {
                    "strict": true,
                    "checkProps": false,
                    "checkEmits": false,
                    "tsconfig": "tsconfig.app.json",
                    "corsaPath": "./node_modules/.bin/corsa"
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    let config = state.get_type_checker_config();
    assert!(config.strict);
    assert!(!config.check_props);
    assert!(!config.check_emits);
    assert_eq!(config.tsconfig.as_deref(), Some("tsconfig.app.json"));
    assert_eq!(config.runtime_path(), Some("./node_modules/.bin/corsa"));
}

#[test]
fn options_api_enabled_by_default() {
    let state = ServerState::new();
    assert!(
        state.options_api_enabled(),
        "Options API resolution is default-on (matches vue-tsc); template bindings \
         resolve without configuration"
    );
}

#[test]
fn type_checker_options_api_opt_out_from_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "typeChecker": { "optionsApi": false } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    assert!(
        !state.options_api_enabled(),
        "typeChecker.optionsApi: false should opt out of Options API binding resolution in the LSP"
    );
}

#[test]
fn type_checker_options_api_explicit_opt_in_from_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "typeChecker": { "optionsApi": true } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    assert!(
        state.options_api_enabled(),
        "typeChecker.optionsApi: true keeps Options API binding resolution enabled in the LSP"
    );
}

#[test]
fn legacy_vue2_config_implies_options_api() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "typeChecker": { "legacyVue2": true } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    assert!(
        state.legacy_vue2_enabled(),
        "typeChecker.legacyVue2 should enable Vue 2 compatibility"
    );
    assert!(
        state.options_api_enabled(),
        "legacy Vue 2 mode is a superset of Options API binding resolution"
    );
}

#[test]
fn load_lsp_config_updates_linter_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "linter": {
                    "preset": "opinionated",
                    "rules": {
                        "vue/prop-name-casing": "off"
                    }
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    let config = state.get_linter_config();
    assert_eq!(config.preset.as_deref(), Some("opinionated"));
    assert_eq!(config.disabled_rules(), ["vue/prop-name-casing"]);
}

#[test]
fn load_lsp_config_invalid_json_keeps_default() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("vize.config.json"), "not valid json").unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    assert_eq!(state.lsp_features(), super::LspFeatureConfig::default());
}

#[test]
fn apply_lsp_initialization_options() {
    let state = ServerState::new();
    let options = serde_json::json!({
        "lint": true,
        "codeActions": true,
        "definition": true,
        "ecosystem": true
    });

    state.apply_lsp_initialization_options(Some(&options));

    let features = state.lsp_features();
    assert!(features.lint);
    assert!(features.code_actions);
    assert!(features.definition);
    assert!(features.ecosystem);
    assert!(features.typecheck);
}

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
    let info = match find_art_block_at_offset(source, offset) {
        Some(BlockType::Art(ArtCursorPosition::VariantTemplate(info))) => info,
        other => panic!("expected secondary variant template, got {other:?}"),
    };

    let generated_offset = secondary_template
        .source_map
        .to_generated(info.relative_offset as u32);
    assert!(generated_offset.is_some());
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
