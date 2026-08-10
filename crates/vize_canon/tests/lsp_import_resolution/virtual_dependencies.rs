//! Editor queries that resolve imports across in-memory virtual dependencies.

use super::*;

#[test]
fn bridge_editor_queries_resolve_in_memory_virtual_dependencies() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(src_dir.join("Parent.vue"), "<template />\n").unwrap();
    std::fs::write(src_dir.join("Child.vue"), "<template />\n").unwrap();

    let child_virtual = r#"declare const Child: new () => {
  $props: { message: string };
};
export default Child;
"#;
    let changed_child_virtual = child_virtual.replace("message: string", "message: number");
    let parent_virtual = r#"import Child from './Child.vue.ts';
type Props = InstanceType<typeof Child>['$props'];
const props = {} as Props;
props.message;
"#;
    let child_path = src_dir.join("Child.vue.ts");
    let parent_path = src_dir.join("Parent.vue.ts");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let (
        initial_hover,
        initial_definition,
        changed_hover,
        changed_definition,
        closed_hover,
        closed_definition,
    ) = corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let child_uri = child_path.display().to_compact_string();
        let child_uri = bridge
            .open_or_update_virtual_document(child_uri.as_str(), child_virtual)
            .await
            .unwrap();
        let parent_uri = parent_path.display().to_compact_string();
        let parent_uri = bridge
            .open_or_update_virtual_document(parent_uri.as_str(), parent_virtual)
            .await
            .unwrap();
        let initial_hover = bridge.hover(parent_uri.as_str(), 3, 7).await.unwrap();
        let initial_definition = bridge.definition(parent_uri.as_str(), 3, 7).await.unwrap();
        bridge
            .update_virtual_document(child_uri.as_str(), changed_child_virtual.as_str(), 2)
            .await
            .unwrap();
        let changed_hover = bridge.hover(parent_uri.as_str(), 3, 7).await.unwrap();
        let changed_definition = bridge.definition(parent_uri.as_str(), 3, 7).await.unwrap();
        bridge
            .close_virtual_document(child_uri.as_str())
            .await
            .unwrap();
        let closed_hover = bridge.hover(parent_uri.as_str(), 3, 7).await.unwrap();
        let closed_definition = bridge.definition(parent_uri.as_str(), 3, 7).await.unwrap();
        bridge.shutdown().await.unwrap();
        (
            initial_hover,
            initial_definition,
            changed_hover,
            changed_definition,
            closed_hover,
            closed_definition,
        )
    });

    assert!(
        hover_contains(&initial_hover, "message") && hover_contains(&initial_hover, "string"),
        "editor LSP should see every in-memory virtual dependency: {initial_hover:?}"
    );
    assert!(
        hover_contains(&changed_hover, "message") && hover_contains(&changed_hover, "number"),
        "editor LSP should refresh changed virtual dependencies: {changed_hover:?}"
    );
    for definitions in [&initial_definition, &changed_definition] {
        assert_eq!(
            definitions.len(),
            1,
            "unexpected definitions: {definitions:#?}"
        );
        assert!(
            definitions[0].uri.ends_with("/src/Child.vue.ts"),
            "definition should target the in-memory Vue dependency: {definitions:#?}"
        );
        assert_eq!(definitions[0].range.start.line, 1);
    }
    assert!(
        !hover_contains(&closed_hover, "string") && !hover_contains(&closed_hover, "number"),
        "editor LSP should close removed virtual dependencies: {closed_hover:?}"
    );
    assert!(
        closed_definition.is_empty(),
        "definition should disappear with the closed virtual dependency: {closed_definition:#?}"
    );
    assert!(!child_path.exists());
    assert!(!parent_path.exists());
}

#[test]
fn bridge_editor_references_and_rename_span_in_memory_virtual_dependencies() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(src_dir.join("Parent.vue"), "<template />\n").unwrap();
    std::fs::write(src_dir.join("Child.vue"), "<template />\n").unwrap();

    let child_virtual = "export const message = 'hello';\n";
    let parent_virtual =
        "import { message } from './Child.vue.ts';\nexport const displayed = message;\n";
    let child_path = src_dir.join("Child.vue.ts");
    let parent_path = src_dir.join("Parent.vue.ts");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let (child_uri, parent_uri, references, references_with_declaration, prepare, rename) =
        corsa::runtime::block_on(async {
            bridge.spawn().await.unwrap();
            let child_uri = child_path.display().to_compact_string();
            let child_uri = bridge
                .open_or_update_virtual_document(child_uri.as_str(), child_virtual)
                .await
                .unwrap();
            let parent_uri = parent_path.display().to_compact_string();
            let parent_uri = bridge
                .open_or_update_virtual_document(parent_uri.as_str(), parent_virtual)
                .await
                .unwrap();
            let declaration_character = child_virtual.find("message").unwrap() as u32;
            let references = bridge
                .references(child_uri.as_str(), 0, declaration_character, false)
                .await
                .unwrap();
            let references_with_declaration = bridge
                .references(child_uri.as_str(), 0, declaration_character, true)
                .await
                .unwrap();
            let prepare = bridge
                .prepare_rename(child_uri.as_str(), 0, declaration_character)
                .await
                .unwrap();
            let rename = bridge
                .rename(
                    child_uri.as_str(),
                    0,
                    declaration_character,
                    "renamedMessage",
                )
                .await
                .unwrap();
            bridge.shutdown().await.unwrap();
            (
                child_uri,
                parent_uri,
                references,
                references_with_declaration,
                prepare,
                rename,
            )
        });

    assert!(
        references.iter().all(|location| location.uri != child_uri),
        "declaration must be excluded: {references:#?}"
    );
    assert!(
        references_with_declaration.iter().any(|location| {
            location.uri == child_uri
                && location.range.start.line == 0
                && location.range.start.character == child_virtual.find("message").unwrap() as u32
        }),
        "declaration must be included: {references_with_declaration:#?}"
    );
    assert!(
        references_with_declaration
            .iter()
            .any(|location| location.uri == parent_uri && location.range.start.line == 0),
        "import must be included: {references_with_declaration:#?}"
    );
    assert!(
        references_with_declaration
            .iter()
            .any(|location| location.uri == parent_uri && location.range.start.line == 1),
        "usage must be included: {references_with_declaration:#?}"
    );
    assert!(prepare.is_some(), "the declaration must be renameable");

    let rename = rename.expect("rename must return a workspace edit");
    let rename_json = serde_json::to_string(&rename).unwrap();
    assert!(rename_json.contains(child_uri.as_str()), "{rename_json}");
    assert!(rename_json.contains(parent_uri.as_str()), "{rename_json}");
    assert!(rename_json.contains("renamedMessage"), "{rename_json}");
    assert!(!child_path.exists());
    assert!(!parent_path.exists());
}
