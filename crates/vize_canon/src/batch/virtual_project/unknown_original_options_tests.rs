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
            "  type __VizeAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof __VizeFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof __VizePublicComponentAttrs | keyof __VizeGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };"
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
            "  type __VizeAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof __VizeFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof __VizePublicComponentAttrs | keyof __VizeGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };"
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
