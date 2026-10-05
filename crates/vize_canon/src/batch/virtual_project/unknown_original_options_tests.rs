//! Original source projection controls; no native execution is implied.
use super::VirtualProject;
use serde_json::{Value, json};
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
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(root.join("src/Child.vue"), CHILD).unwrap();
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
        Some("  type __VizeAllowedFallthroughAttrs<C> = {};")
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
    let enabled = source(
        None,
        &vize_l0::cstr!("<!-- @checkUnknownProps true -->\n{PARENT}"),
    );
    assert_eq!(
        enabled
            .lines()
            .find(|line| line.starts_with("  type __VizeAllowedFallthroughAttrs")),
        Some("  type __VizeAllowedFallthroughAttrs<C> = {};")
    );
}
