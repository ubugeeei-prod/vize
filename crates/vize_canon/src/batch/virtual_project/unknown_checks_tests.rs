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
    let props = content.contains("__VizeComponentGlobalHtmlAttrs");
    let directive = content.contains("__vize_unknown_directive_");
    let open_tail = content.contains(
        "type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;",
    );
    assert_eq!(component, enabled, "{content}");
    assert_eq!(props, enabled, "{content}");
    assert_eq!(directive, enabled, "{content}");
    assert_eq!(open_tail, !enabled, "{content}");
}

fn type_declaration<'a>(content: &'a str, name: &str) -> Option<&'a str> {
    content.lines().find(|line| {
        line.strip_prefix("  type ")
            .and_then(|rest| rest.split(['<', '=']).next())
            .is_some_and(|declared| declared.trim_end() == name)
    })
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

#[test]
fn unknown_props_reuse_the_program_wide_native_attribute_union() {
    let content = project_source("shared-native-attrs", "\"checkUnknownProps\": true");
    assert_eq!(
        type_declaration(&content, "__VizeAllowedFallthroughAttrs"),
        Some(
            "  type __VizeAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof __VizeFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof __VizePublicComponentAttrs | keyof __VizeComponentGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };"
        )
    );
    assert_eq!(type_declaration(&content, "__VizeComponentAttrCamel"), None);
    assert_eq!(
        type_declaration(&content, "__VizeComponentNativeAttrNames"),
        None
    );
}

mod originals {
    //! Original source projection controls; no native execution is implied.
    use super::VirtualProject;
    use serde_json::{Value, json};
    use vize_l0::{String, cstr};
    const CHILD: &str = include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/unknown-template-options/src/Child.vue.txt"
    );
    const PARENT: &str = include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/unknown-template-options/src/Parent.vue.txt"
    );
    const CONFIG: &str = include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/unknown-template-options/tsconfig.json.txt"
    );

    fn source(config: Option<Value>, script: &str) -> vize_carton::String {
        source_with_child(config, script, CHILD)
    }

    fn source_with_child(config: Option<Value>, script: &str, child: &str) -> String {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::write(root.join("src/Child.vue"), child).unwrap();
        std::fs::write(root.join("src/Parent.vue"), script).unwrap();
        if let Some(config) = config {
            std::fs::write(
                root.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
        }
        let mut project = VirtualProject::new(&root).unwrap();
        if root.join("tsconfig.json").is_file() {
            project.set_tsconfig_path(Some(root.join("tsconfig.json")));
        }
        project.register_path(&root.join("src/Child.vue")).unwrap();
        project.register_path(&root.join("src/Parent.vue")).unwrap();
        project
            .find_by_original(&root.join("src/Parent.vue"))
            .unwrap()
            .content
            .clone()
    }

    #[test]
    fn original_valueless_directive_and_native_root_prop_have_strict_projection_authority() {
        let enabled = source(Some(serde_json::from_str(CONFIG).unwrap()), PARENT);
        let presence: Vec<_> = enabled
            .lines()
            .filter(|line| line.starts_with("  const { \"vNotADirective\""))
            .collect();
        assert_eq!(presence.len(), 1);
        assert_eq!(
            enabled
                .lines()
                .find(|line| line.starts_with("  type __VizeAllowedFallthroughAttrs")),
            Some(
                "  type __VizeAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof __VizeFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof __VizePublicComponentAttrs | keyof __VizeComponentGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };"
            )
        );
        let mut off: Value = serde_json::from_str(CONFIG).unwrap();
        off["vueCompilerOptions"] = json!({"checkUnknownComponents":false,"checkUnknownProps":false,"checkUnknownDirectives":false});
        let disabled = source(Some(off), PARENT);
        assert!(!disabled.contains("__vize_unknown_directive_"));
        assert_eq!(
            disabled
                .lines()
                .find(|line| line.starts_with("  type __VizeComponentCheckTail")),
            Some(
                "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;"
            )
        );
        let absent = source(None, PARENT);
        assert!(!absent.contains("__vize_unknown_directive_"));
        assert_eq!(
            absent
                .lines()
                .find(|line| line.starts_with("  type __VizeAllowedFallthroughAttrs")),
            Some(
                "  type __VizeAllowedFallthroughAttrs<C> = __VizeHasFallthroughProps<C> extends true ? Record<string, unknown> : {};"
            )
        );
    }

    #[test]
    fn explicit_prop_comment_keeps_absent_configuration_compatible() {
        let enabled = source(None, &cstr!("<!-- @checkUnknownProps true -->\n{PARENT}"));
        assert_eq!(
            enabled
                .lines()
                .find(|line| line.starts_with("  type __VizeAllowedFallthroughAttrs")),
            Some(
                "  type __VizeAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof __VizeFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof __VizePublicComponentAttrs | keyof __VizeComponentGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };"
            )
        );
    }

    #[test]
    fn strict_only_configuration_keeps_the_original_native_root_fallthrough_open() {
        let mut config: Value = serde_json::from_str(CONFIG).unwrap();
        config["vueCompilerOptions"] = json!({"strictTemplates":true});
        let content = source(Some(config), PARENT);
        assert_eq!(
            content
                .lines()
                .find(|line| line.starts_with("  type __VizeAllowedFallthroughAttrs")),
            Some(
                "  type __VizeAllowedFallthroughAttrs<C> = __VizeHasFallthroughProps<C> extends true ? Record<string, unknown> : {};"
            ),
        );
    }

    #[test]
    fn explicit_unknown_props_false_overrides_strict_without_changing_the_original_projection() {
        let mut config: Value = serde_json::from_str(CONFIG).unwrap();
        config["vueCompilerOptions"] = json!({
            "checkUnknownComponents":true,"checkUnknownDirectives":true,
            "checkUnknownEvents":true,"strictVModel":true,"checkUnknownProps":false
        });
        let baseline = source(Some(config.clone()), PARENT);
        config["vueCompilerOptions"]["strictTemplates"] = json!(true);
        assert_eq!(source(Some(config), PARENT), baseline);
    }

    #[test]
    fn complete_upstream_unknown_prop_plant_retains_legacy_strict_and_explicit_false_precedence() {
        const APP: &str = include_str!(
            "../../../../../tests/_fixtures/differential/typechecker/strict-template-fallback/App.vue.txt"
        );
        const CHILD: &str = include_str!(
            "../../../../../tests/_fixtures/differential/typechecker/strict-template-fallback/Child.vue.txt"
        );
        const CONFIG: &str = include_str!(
            "../../../../../tests/_fixtures/differential/typechecker/strict-template-fallback/tsconfig.json.txt"
        );
        let mut strict: Value = serde_json::from_str(CONFIG).unwrap();
        let inherited = source_with_child(Some(strict.clone()), APP, CHILD);
        strict["vueCompilerOptions"]["checkUnknownProps"] = json!(true);
        assert_ne!(
            inherited,
            source_with_child(Some(strict.clone()), APP, CHILD)
        );
        strict["vueCompilerOptions"]["checkUnknownProps"] = json!(false);
        let overridden = source_with_child(Some(strict.clone()), APP, CHILD);
        assert_ne!(inherited, overridden);
        let _ = strict["vueCompilerOptions"]
            .as_object_mut()
            .unwrap()
            .remove("strictTemplates");
        strict["vueCompilerOptions"]["checkUnknownComponents"] = json!(true);
        strict["vueCompilerOptions"]["checkUnknownDirectives"] = json!(true);
        strict["vueCompilerOptions"]["checkUnknownEvents"] = json!(true);
        strict["vueCompilerOptions"]["strictVModel"] = json!(true);
        assert_eq!(overridden, source_with_child(Some(strict), APP, CHILD));
    }

    #[test]
    fn strict_component_attrs_is_opt_in_and_obeys_unknown_props_false() {
        let mut config: Value = serde_json::from_str(CONFIG).unwrap();
        config["vueCompilerOptions"] = json!({"strictTemplates":true});
        let legacy = source(Some(config.clone()), PARENT);
        config["vueCompilerOptions"]["strictComponentAttrs"] = json!(false);
        assert_eq!(legacy, source(Some(config.clone()), PARENT));
        config["vueCompilerOptions"]["strictComponentAttrs"] = json!(true);
        let strict = source(Some(config.clone()), PARENT);
        assert_ne!(legacy, strict);
        config["vueCompilerOptions"]["checkUnknownProps"] = json!(false);
        let unchecked = source(Some(config.clone()), PARENT);
        config["vueCompilerOptions"]["strictComponentAttrs"] = json!(false);
        assert_eq!(unchecked, source(Some(config), PARENT));
    }
}
