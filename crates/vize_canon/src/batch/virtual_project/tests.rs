use super::build::source_type_for_path;
use super::tsconfig_paths::{parse_jsonc_value, strip_json_comments};
use super::{AUTO_IMPORT_STUBS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject};
use crate::batch::SfcBlockType;
use crate::virtual_ts::VirtualTsOptions;
use std::fs;
use std::path::{Path, PathBuf};
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::cstr;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn assert_ts_parses(source: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::ts()).parse();
    assert!(
        parsed.errors.is_empty(),
        "virtual TS should parse without errors: {:?}",
        parsed.errors
    );
}

#[test]
fn test_virtual_project_new() {
    let case_dir = unique_case_dir("new");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();

    let project = VirtualProject::new(&case_dir).unwrap();

    assert_eq!(project.project_root(), case_dir.as_path());
    assert!(project.virtual_root().ends_with("node_modules/.vize/canon"));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_materialize_writes_vue_module_stubs() {
    let case_dir = unique_case_dir("vue-module-stubs");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let main_path = src_dir.join("main.ts");
    fs::write(&main_path, "import App from './App.vue';\nvoid App;\n").unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&main_path).unwrap();
    project.materialize().unwrap();

    let stubs = fs::read_to_string(project.virtual_root().join("__vize_vue_modules.d.ts")).unwrap();
    assert!(stubs.contains(r#"declare module "*.vue.ts""#));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_register_vue_file_rewrites_child_imports() {
    let case_dir = unique_case_dir("register-vue");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("App.vue");
    let vue_content = r#"<script setup lang="ts">
import Child from './Child.vue'
const count = 1
</script>

<template>
  <Child :count="count" />
</template>
"#;
    fs::write(&vue_path, vue_content).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();

    let virtual_file = project.find_by_original(&vue_path).unwrap();
    insta::assert_snapshot!(virtual_file.content.as_str());

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_register_vue_file_rewrites_options_api_export_default() {
    let case_dir = unique_case_dir("options-api-export-default");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("OptionsApi.vue");
    let vue_content = r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "OptionsApi",
});
</script>

<template>
  <div>hello</div>
</template>
"#;
    fs::write(&vue_path, vue_content).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();

    let virtual_file = project.find_by_original(&vue_path).unwrap();
    insta::assert_snapshot!(virtual_file.content.as_str());
    assert_ts_parses(virtual_file.content.as_str());

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_register_vue_file_respects_template_syntax_quirks() {
    let case_dir = unique_case_dir("template-syntax-quirks");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("App.vue");
    let vue_content = r#"<script setup lang="ts">
defineProps<{
  test: string
}>()
</script>

<template>
  <div />
</template>
"#;
    fs::write(&vue_path, vue_content).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.set_template_syntax(TemplateSyntaxMode::Quirks);
    project.register_path(&vue_path).unwrap();

    assert!(
        project.diagnostics().is_empty(),
        "{:#?}",
        project.diagnostics()
    );
    assert!(project.find_by_original(&vue_path).is_some());

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_register_vue_file_reports_script_parse_error_with_fallback() {
    let case_dir = unique_case_dir("script-parse-error");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("Broken.vue");
    let vue_content = r#"<script setup lang="ts">
const count =
</script>

<template>
  <div>{{ count }}</div>
</template>
"#;

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();

    let diagnostics = project.diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Script parse error"));
    assert_eq!(diagnostics[0].block_type, Some(SfcBlockType::ScriptSetup));

    let virtual_file = project.find_by_original(&vue_path).unwrap();
    assert!(
        virtual_file
            .content
            .contains("export default __vize_component")
    );
    assert!(!virtual_file.content.contains("const count ="));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_register_vue_file_reports_props_destructure_default_type_mismatch() {
    // Regression for: `const { msg = 0 } = defineProps<{ msg?: string }>()` should
    // surface in `vize check`. TypeScript itself does not flag the mismatch
    // (destructure defaults widen the binding's type), so the diagnostic has
    // to come from the SFC compiler's validator.
    let case_dir = unique_case_dir("props-destructure-default-type");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("Bad.vue");
    let vue_content = r#"<script setup lang="ts">
const { msg = 0 } = defineProps<{ msg?: string }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();

    let diagnostics = project.diagnostics();
    assert_eq!(diagnostics.len(), 1, "expected one SFC compile diagnostic");
    let diagnostic = &diagnostics[0];
    assert!(
        diagnostic
            .message
            .contains("DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE"),
        "expected DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE in message, got: {}",
        diagnostic.message
    );
    assert!(
        diagnostic.message.contains("Default value of prop \"msg\""),
        "expected message to name the prop, got: {}",
        diagnostic.message
    );
    assert_eq!(diagnostic.severity, 1);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_register_vue_file_allows_matching_props_destructure_default() {
    let case_dir = unique_case_dir("props-destructure-default-ok");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("Good.vue");
    let vue_content = r#"<script setup lang="ts">
const { msg = "ok" } = defineProps<{ msg?: string }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();

    assert!(
        project.diagnostics().is_empty(),
        "no diagnostics expected for matching default, got: {:?}",
        project.diagnostics()
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_register_vue_file_reports_template_parse_error_with_fallback() {
    let case_dir = unique_case_dir("template-parse-error");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("BrokenTemplate.vue");
    let vue_content = r#"<script setup lang="ts">
const count = 1
</script>

<template><div>{{ count }}</template>
"#;

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();

    let diagnostics = project.diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Template parse error"));
    assert_eq!(diagnostics[0].block_type, Some(SfcBlockType::Template));

    let virtual_file = project.find_by_original(&vue_path).unwrap();
    assert!(
        virtual_file
            .content
            .contains("export default __vize_component")
    );
    assert!(!virtual_file.content.contains("__vize_check_template"));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_virtual_ts_exposes_props_from_reexported_vue_interface() {
    let case_dir = unique_case_dir("reexported-vue-interface-props");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let base = src_dir.join("Base.vue");
    let index = src_dir.join("index.ts");
    let child = src_dir.join("Child.vue");
    let parent = src_dir.join("ParentWidget.vue");

    fs::write(
        &base,
        r#"<script lang="ts">
export interface BaseProps {
  as?: string;
  asChild?: boolean;
}
</script>
<template><div></div></template>"#,
    )
    .unwrap();
    fs::write(&index, r#"export { type BaseProps } from "./Base.vue";"#).unwrap();
    fs::write(
        &child,
        r#"<script setup lang="ts">
defineProps<{ as?: string; asChild?: boolean }>();
</script>
<template><div></div></template>"#,
    )
    .unwrap();
    fs::write(
        &parent,
        r#"<script lang="ts">
import type { BaseProps } from "./index";

export interface ParentWidgetProps extends BaseProps {}
</script>
<script setup lang="ts">
import Child from "./Child.vue";

const props = defineProps<ParentWidgetProps>();
</script>
<template>
  <Child :as="as" :as-child="props.asChild" />
</template>"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project
        .register_paths(&[base, index, child, parent.clone()])
        .unwrap();

    let virtual_parent = project.find_by_original(&parent).unwrap();
    assert_ts_parses(&virtual_parent.content);
    assert!(
        virtual_parent
            .content
            .contains(r#"const _as = props["as"];"#),
        "{}",
        virtual_parent.content
    );
    assert!(
        virtual_parent.content.contains(r#"void (props["as"]);"#),
        "{}",
        virtual_parent.content
    );
    assert!(
        virtual_parent
            .content
            .contains(r#"type Props = ParentWidgetProps;"#),
        "{}",
        virtual_parent.content
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_virtual_ts_preserves_ts_as_assertions_when_prop_is_named_as() {
    let case_dir = unique_case_dir("template-as-assertion-prop");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        r#"<script setup lang="ts">
defineProps<{
  as?: string
}>()

const value = 'demo'
const onFocus = (target: HTMLElement) => {
  target.dataset.focused = 'true'
}
</script>

<template>
  <div
    :data-value="(value as any)"
    :style="{
      ['--demo-value' as any]: value,
    }"
    v-on="{
      focusin: (event: FocusEvent) => {
        onFocus(event.target as HTMLElement)
      },
    }"
  ></div>
</template>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project
        .register_vue_file(&vue_path, &fs::read_to_string(&vue_path).unwrap())
        .unwrap();

    let virtual_file = project.find_by_original(&vue_path).unwrap();
    assert_ts_parses(&virtual_file.content);
    assert!(
        virtual_file.content.contains("void ((value as any));"),
        "{}",
        virtual_file.content
    );
    assert!(
        virtual_file
            .content
            .contains("['--demo-value' as any]: value"),
        "{}",
        virtual_file.content
    );
    assert!(
        virtual_file
            .content
            .contains("onFocus(event.target as HTMLElement)"),
        "{}",
        virtual_file.content
    );
    assert!(
        !virtual_file.content.contains(r#"value props["as"] any"#),
        "{}",
        virtual_file.content
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_materialize_writes_tsconfig_and_virtual_files() {
    let case_dir = unique_case_dir("materialize");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        r#"<script setup lang="ts">
const message = 'Hello'
</script>

<template>
  <div>{{ message }}</div>
</template>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    let mut options = VirtualTsOptions::default();
    options
        .auto_import_stubs
        .push("declare function autoGenerated(): string;".into());
    project.set_virtual_ts_options(options);
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let virtual_vue_path = case_dir.join("node_modules/.vize/canon/src/App.vue.ts");
    let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
    let auto_imports_path = case_dir.join("node_modules/.vize/canon/__vize_auto_imports.d.ts");

    assert!(virtual_vue_path.exists());
    assert!(tsconfig_path.exists());
    assert!(auto_imports_path.exists());
    assert!(
        !fs::read_to_string(&virtual_vue_path)
            .unwrap()
            .contains("autoGenerated")
    );
    assert!(
        fs::read_to_string(&auto_imports_path)
            .unwrap()
            .contains("autoGenerated")
    );
    assert!(
        fs::read_to_string(&tsconfig_path)
            .unwrap()
            .contains("__vize_auto_imports.d.ts")
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_materialize_writes_relative_json_modules() {
    let case_dir = unique_case_dir("materialize-json-modules");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    let token_dir = src_dir.join("tokens/source");
    fs::create_dir_all(&token_dir).unwrap();
    let ts_path = src_dir.join("tokens.ts");
    let json_path = token_dir.join("colors.tokens.json");
    fs::write(
        &ts_path,
        "import colors from './tokens/source/colors.tokens.json'\nvoid colors\n",
    )
    .unwrap();
    fs::write(&json_path, "{\"primary\":\"#0057ff\"}\n").unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&ts_path).unwrap();
    project.materialize().unwrap();

    let virtual_json_path =
        case_dir.join("node_modules/.vize/canon/src/tokens/source/colors.tokens.json");
    assert_eq!(
        fs::read_to_string(&virtual_json_path).unwrap(),
        "{\"primary\":\"#0057ff\"}\n"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn materialize_prunes_stale_virtual_project_entries() {
    let case_dir = unique_case_dir("materialize-gc");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        r#"<script setup lang="ts">
const message = 'Hello'
</script>

<template>
  <div>{{ message }}</div>
</template>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    let mut options = VirtualTsOptions::default();
    options
        .auto_import_stubs
        .push("declare function autoGenerated(): string;".into());
    project.set_virtual_ts_options(options);
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let virtual_root = project.virtual_root().to_path_buf();
    let stale_file = virtual_root.join("src/Old.vue.ts");
    let stale_dir_file = virtual_root.join("stale/nested/Unused.vue.ts");
    let stale_dts_config = virtual_root.join("tsconfig.declaration.json");
    let stale_package = virtual_root.join("node_modules/unused/package.json");
    fs::write(&stale_file, "export default {}").unwrap();
    fs::create_dir_all(stale_dir_file.parent().unwrap()).unwrap();
    fs::write(&stale_dir_file, "export default {}").unwrap();
    fs::write(&stale_dts_config, "{}").unwrap();
    fs::create_dir_all(stale_package.parent().unwrap()).unwrap();
    fs::write(&stale_package, "{}").unwrap();
    #[cfg(unix)]
    {
        let expected_virtual_file = virtual_root.join("src/App.vue.ts");
        let hijack_target = case_dir.join("hijack.ts");
        fs::write(&hijack_target, "hijacked").unwrap();
        fs::remove_file(&expected_virtual_file).unwrap();
        std::os::unix::fs::symlink(&hijack_target, &expected_virtual_file).unwrap();
    }

    let mut next_project = VirtualProject::new(&case_dir).unwrap();
    next_project.register_path(&vue_path).unwrap();
    next_project.materialize().unwrap();

    assert!(!stale_file.exists());
    assert!(!stale_dir_file.exists());
    assert!(!stale_dir_file.parent().unwrap().exists());
    assert!(!stale_dts_config.exists());
    assert!(!virtual_root.join(AUTO_IMPORT_STUBS_FILE).exists());
    assert!(!stale_package.exists());
    assert!(!stale_package.parent().unwrap().exists());
    assert!(virtual_root.join("src/App.vue.ts").exists());
    #[cfg(unix)]
    {
        let virtual_file_metadata =
            fs::symlink_metadata(virtual_root.join("src/App.vue.ts")).unwrap();
        assert!(!virtual_file_metadata.file_type().is_symlink());
        assert_eq!(
            fs::read_to_string(case_dir.join("hijack.ts")).unwrap(),
            "hijacked"
        );
    }
    assert!(virtual_root.join(VUE_MODULE_STUBS_FILE).exists());
    assert!(virtual_root.join("tsconfig.json").exists());

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn materialized_tsconfig_preserves_original_path_option_bases() {
    let case_dir = unique_case_dir("tsconfig-path-bases");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    },
    "rootDirs": ["src", "generated"],
    "typeRoots": ["types"]
  }
}"#,
    )
    .unwrap();
    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    let compiler_options = value["compilerOptions"].as_object().unwrap();

    assert_eq!(compiler_options["strict"], serde_json::Value::Bool(true));
    assert_eq!(
        compiler_options["allowImportingTsExtensions"],
        serde_json::Value::Bool(true)
    );
    for option in ["baseUrl", "rootDir", "rootDirs", "typeRoots"] {
        assert!(
            !compiler_options.contains_key(option),
            "{option} should remain owned by the extended tsconfig"
        );
    }

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn materialized_tsconfig_reanchors_paths_into_virtual_mirror() {
    let case_dir = unique_case_dir("tsconfig-paths-reanchor");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r##"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "#shared": ["./shared/index.ts"]
    }
  }
}"##,
    )
    .unwrap();
    let vue_path = src_dir.join("App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    let paths = value["compilerOptions"]["paths"].as_object().unwrap();

    // Each target gets a mirror candidate (relative to the virtual tsconfig
    // in `node_modules/.vize/canon`) first, then the real-tree fallback.
    assert_eq!(
        paths["@/*"],
        serde_json::json!(["./src/*", "../../../src/*"])
    );
    assert_eq!(
        paths["#shared"],
        serde_json::json!(["./shared/index.ts", "../../../shared/index.ts"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn materialized_tsconfig_reanchors_extended_paths_from_declaring_config_dir() {
    let case_dir = unique_case_dir("tsconfig-extended-paths-reanchor");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".nuxt")).unwrap();
    fs::create_dir_all(case_dir.join("app/components")).unwrap();
    fs::write(
        case_dir.join(".nuxt/tsconfig.json"),
        r##"{
  "compilerOptions": {
    "paths": {
      "~/*": ["../app/*"],
      "#imports": ["./imports"]
    }
  }
}"##,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": "./.nuxt/tsconfig.json"
}"#,
    )
    .unwrap();
    let vue_path = case_dir.join("app/components/App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">const count = 1</script>",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&vue_path).unwrap();
    project.materialize().unwrap();

    let tsconfig_path = case_dir.join("node_modules/.vize/canon/tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    let paths = value["compilerOptions"]["paths"].as_object().unwrap();

    assert_eq!(
        paths["~/*"],
        serde_json::json!(["./app/*", "../../../app/*"])
    );
    assert_eq!(
        paths["#imports"],
        serde_json::json!(["./.nuxt/imports", "../../../.nuxt/imports"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn test_parse_jsonc_value_handles_comments_and_trailing_commas() {
    let value = parse_jsonc_value(
        r#"{
  // comment
  "compilerOptions": {
    "strict": true,
    "paths": {
      "@/*": ["src/*",],
    },
  },
}"#,
    )
    .unwrap();

    assert_eq!(
        value["compilerOptions"]["paths"]["@/*"][0],
        serde_json::Value::String("src/*".into())
    );
}

#[test]
fn test_strip_json_comments_preserves_strings() {
    let stripped = strip_json_comments(r#"{ "url": "https://example.com" }"#);
    insta::assert_snapshot!(stripped.as_str());
}

#[test]
fn test_source_type_for_path() {
    assert_eq!(
        source_type_for_path(Path::new("foo.ts")),
        Some(oxc_span::SourceType::ts())
    );
    assert_eq!(
        source_type_for_path(Path::new("foo.tsx")),
        Some(oxc_span::SourceType::tsx())
    );
    assert_eq!(source_type_for_path(Path::new("foo.vue")), None);
}
