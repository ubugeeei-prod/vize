use super::{ServerState, declared_facts};
use tower_lsp::lsp_types::Url;
use vize_l0::String;

#[test]
fn authored_global_values_and_vue_augmentations_have_distinct_roles() {
    let values =
        declared_facts("export {}; declare global { const ref: typeof import('vue')['ref']; }");
    assert!(values.global_values);
    assert!(!values.vue_components);
    assert!(values.names.is_empty());
    let components =
        declared_facts("declare module 'vue' { interface GlobalComponents {} } export {};");
    assert!(components.vue_components);
    assert!(!components.global_values);
}

#[test]
fn unrelated_interfaces_comments_and_nested_namespace_values_are_not_global_roles() {
    for source in [
        "// declare global { const ref: unknown; }\nexport {};",
        "export {}; declare module 'other' { interface GlobalComponents { Wrong: unknown } }",
        "export {}; declare global { interface Ref { value: unknown } type Alias = string; }",
        "export {}; declare global { namespace Private { const ref: unknown; } }",
        "export {}; declare global { const = ; }",
    ] {
        let facts = declared_facts(source);
        assert!(!facts.global_values, "{source}");
        assert!(!facts.vue_components, "{source}");
        assert!(facts.names.is_empty(), "{source}");
    }
}

#[test]
fn corpus_uses_the_original_volt_and_compatibility_declaration_roles() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../tests/_fixtures/lsp-declaration-reference-roles.json"
    ))
    .unwrap();
    let values = declared_facts(corpus.get("voltAmbient").unwrap().as_str().unwrap());
    assert!(values.global_values);
    assert!(!values.vue_components);
    for entry in corpus
        .get("componentAugmentations")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let facts = declared_facts(entry.get("declaration").unwrap().as_str().unwrap());
        assert!(facts.vue_components);
        assert!(!facts.global_values);
        assert_eq!(
            facts.names,
            entry
                .get("names")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|name| String::from(name.as_str().unwrap()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn cached_roles_follow_the_same_unsaved_revision_and_reopen_as_tag_names() {
    let workspace = tempfile::tempdir().unwrap();
    let path = workspace.path().join("ambient.d.ts");
    std::fs::write(
        &path,
        "declare module 'vue' { interface GlobalComponents { Disk: unknown } }",
    )
    .unwrap();
    let path = std::fs::canonicalize(path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(workspace.path().to_path_buf());
    let values = || crate::runtime::block_on(state.global_value_reference_paths());
    let components = || crate::runtime::block_on(state.vue_component_augmentation_paths());
    assert!(values().is_empty());
    assert_eq!(components(), vec![path.clone()]);
    assert_eq!(
        state
            .global_component_references
            .tag_names
            .names
            .read()
            .len(),
        1
    );
    let uri = Url::from_file_path(&path).unwrap();
    state.documents.open(
        uri.clone(),
        "export {}; declare global { const ref: unknown; }".into(),
        1,
        "typescript".into(),
    );
    assert_eq!(values(), vec![path.clone()]);
    assert!(components().is_empty());
    assert!(crate::runtime::block_on(state.global_component_tag_names()).is_empty());
    state.documents.close(&uri);
    state.documents.open(uri, "export {}; declare global { const ref: unknown; } declare module 'vue' { interface GlobalComponents { Reopened: unknown } }".into(), 1, "typescript".into());
    // A whole mixed declaration is not injected into canonical hover: exposing
    // its component augmentation would change existing slot hover output.
    assert!(values().is_empty());
    assert_eq!(components(), vec![path]);
    assert_eq!(
        crate::runtime::block_on(state.global_component_tag_names()),
        vec![String::from("Reopened")]
    );
    assert_eq!(
        state
            .global_component_references
            .tag_names
            .names
            .read()
            .len(),
        1
    );
}
