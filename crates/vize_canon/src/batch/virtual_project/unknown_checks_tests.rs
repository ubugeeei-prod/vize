//! `vueCompilerOptions` unknown-component, unknown-prop, and unknown-directive
//! checks are projection switches. Absent and explicit false stay silent.

use std::fs;
use std::path::{Path, PathBuf};

use super::VirtualProject;

fn case_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(vize_carton::cstr!("unknown-checks-{name}-{}", std::process::id()).as_str())
}

fn project_source(name: &str, vue_compiler_options: &str) -> vize_carton::String {
    let root = case_dir(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    let tsconfig = if vue_compiler_options.is_empty() {
        vize_carton::cstr!(
            r#"{{
  "compilerOptions": {{ "strict": true }},
  "include": ["src/**/*"]
}}
"#
        )
    } else {
        vize_carton::cstr!(
            r#"{{
  "compilerOptions": {{ "strict": true }},
  "vueCompilerOptions": {{ {vue_compiler_options} }},
  "include": ["src/**/*"]
}}
"#
        )
    };
    fs::write(root.join("tsconfig.json"), tsconfig).unwrap();
    let child = root.join("src/Child.vue");
    let app = root.join("src/App.vue");
    fs::write(
        &child,
        "<script setup lang=\"ts\">\ndefineProps<{ label: string }>()\n</script>\n<template><span>{{ label }}</span></template>\n",
    )
    .unwrap();
    fs::write(
        &app,
        "<script setup lang=\"ts\">\nimport Child from './Child.vue'\n</script>\n<template>\n  <MissingWidget />\n  <Child extra=\"1\" />\n  <div v-not-registered=\"1\" />\n</template>\n",
    )
    .unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_tsconfig_path(Some(root.join("tsconfig.json")));
    project.register_path(&child).unwrap();
    project.register_path(&app).unwrap();
    let content = project.find_by_original(&app).unwrap().content.clone();
    let _ = fs::remove_dir_all(&root);
    content
}

fn assert_unknown_checks(content: &str, enabled: bool) {
    let component = content.contains("__vize_global_component_");
    let props = content.contains("__VizeGlobalHtmlAttrs");
    let directive = content.contains("__vize_unknown_directive_");
    let open_tail = content.contains(
        "type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;",
    );
    assert_eq!(component, enabled, "{content}");
    assert_eq!(props, enabled, "{content}");
    assert_eq!(directive, enabled, "{content}");
    assert_eq!(open_tail, !enabled, "{content}");
}

#[test]
fn unknown_checks_follow_vue_compiler_options() {
    assert_unknown_checks(
        project_source(
            "on",
            "\"strictTemplates\": false, \"checkUnknownComponents\": true, \"checkUnknownProps\": true, \"checkUnknownDirectives\": true",
        )
        .as_str(),
        true,
    );
    assert_unknown_checks(project_source("absent", "").as_str(), false);
    assert_unknown_checks(
        project_source(
            "off",
            "\"strictTemplates\": true, \"checkUnknownComponents\": false, \"checkUnknownProps\": false, \"checkUnknownDirectives\": false",
        )
        .as_str(),
        false,
    );
    assert_unknown_checks(
        project_source("strict", "\"strictTemplates\": true").as_str(),
        true,
    );
}

#[path = "unknown_original_options_tests.rs"]
mod originals;
