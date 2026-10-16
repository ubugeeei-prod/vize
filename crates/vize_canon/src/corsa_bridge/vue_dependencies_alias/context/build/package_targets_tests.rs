//! Package target generation is shared only within one editor preparation.

use std::path::Path;

use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

const HOST: &str = "<script setup lang=\"ts\">import Root from 'root-widget'</script><template><Root /></template>";
const ROOT: &str = "<script setup lang=\"ts\">import Leaf from 'leaf-widget'; defineProps<{ rootCount: number }>()</script><template><Leaf /></template>";
const LEAF: &str = "<script setup lang=\"ts\">defineProps<{ leafLabel: string }>()</script><template><div>{{ leafLabel }}</div></template>";

#[test]
fn editor_package_fixpoint_generates_each_target_once_and_keeps_fresh_revisions() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("src/App.vue");
    let root_component = root.join("node_modules/root-widget/Root.vue");
    let leaf = root.join("node_modules/leaf-widget/Leaf.vue");
    write(&root.join("tsconfig.json"), "{}");
    write(&host, HOST);
    write(&root_component, ROOT);
    write(&leaf, LEAF);
    for (name, component) in [("root-widget", "Root.vue"), ("leaf-widget", "Leaf.vue")] {
        write(
            &root.join("node_modules").join(name).join("package.json"),
            &vize_carton::cstr!(r#"{{"name":"{name}","exports":"./{component}"}}"#),
        );
    }

    let mut context = AliasContext::for_host(&host, HOST, &Default::default());
    let mirror = context.mirror.as_mut().unwrap();
    assert_eq!(mirror.registered_original_paths_sorted().len(), 3);
    let (_, generated) = mirror.editor_vue_document(&leaf).unwrap();
    assert!(
        generated
            .code
            .contains("const leafLabel = props[\"leafLabel\"]"),
        "{}",
        generated.code
    );
    // Before the fix, every package discovery wave plus the final open
    // regenerated the already registered package components.
    let work = mirror.materialize_incremental_delta().unwrap();
    assert_eq!(work.source_nodes_rebuilt, 3, "{work:?}");

    let edited = LEAF
        .replace("leafLabel: string", "leafTotal: number")
        .replace("leafLabel", "leafTotal");
    write(&leaf, &edited);
    let context = AliasContext::for_host(&host, HOST, &Default::default());
    let (_, generated) = context
        .mirror
        .as_ref()
        .unwrap()
        .editor_vue_document(&leaf)
        .unwrap();
    assert!(
        generated
            .code
            .contains("const leafTotal = props[\"leafTotal\"]"),
        "{}",
        generated.code
    );
    assert!(!generated.code.contains("leafLabel"), "{}", generated.code);

    // A new preparation must reread the manifest after the selected source
    // changes, and must not retain the deleted source from a prior fixpoint.
    let replacement = leaf.with_file_name("Replacement.vue");
    write(&replacement, &edited);
    std::fs::remove_file(&leaf).unwrap();
    write(
        &root.join("node_modules/leaf-widget/package.json"),
        r#"{"name":"leaf-widget","exports":"./Replacement.vue"}"#,
    );
    write(
        &root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"Bundler"}}"#,
    );
    let context = AliasContext::for_host(&host, HOST, &Default::default());
    let mirror = context.mirror.as_ref().unwrap();
    assert!(mirror.find_by_original(&leaf).is_none());
    let (_, generated) = mirror.editor_vue_document(&replacement).unwrap();
    assert!(
        generated
            .code
            .contains("const leafTotal = props[\"leafTotal\"]"),
        "{}",
        generated.code
    );
    assert_eq!(
        mirror.effective_tsconfig_path(),
        Some(root.join("tsconfig.json"))
    );
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
