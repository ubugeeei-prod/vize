//! Package target generation is shared only within one editor preparation.

use std::path::Path;

use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

const HOST: &str = "<script setup lang=\"ts\">import Root from 'root-widget'</script><template><Root /></template>";
const ROOT: &str = "<script setup lang=\"ts\">import Leaf from 'leaf-widget'; defineProps<{ rootCount: number }>()</script><template><Leaf /></template>";
const LEAF: &str = "<script setup lang=\"ts\">defineProps<{ leafLabel: string }>()</script><template><div>{{ leafLabel }}</div></template>";

#[test]
fn missing_package_target_terminates_and_a_later_wave_registers_its_recreated_source() {
    // Keep the regression bounded independently of the implementation's loop:
    // registration silently skips missing files, rather than returning Err.
    let (finished, result) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let host = root.join("App.vue");
        let package = root.join("node_modules/leaf-widget");
        let missing = package.join("Leaf.vue");
        let manifest = package.join("package.json");
        write(&host, HOST);
        write(
            &manifest,
            r#"{"name":"leaf-widget","exports":"./Leaf.vue"}"#,
        );
        let mut project = crate::batch::virtual_project::VirtualProject::new(&root).unwrap();
        project.register_path(&host).unwrap();
        project.set_package_routes([crate::PackageRouteBinding {
            importer_path: host.clone(),
            specifier: "leaf-widget".into(),
            occurrence_mode: crate::PackageResolutionMode::Import,
            context: Default::default(),
            route: Some(crate::PackageRoute {
                source_paths: vec![missing.clone()],
                dependency_paths: Vec::new(),
                source_targets: vec![crate::PackageRouteSource {
                    target_path: missing.clone(),
                    source_path: missing.clone(),
                    native_probe_path: missing.with_extension("d.vue.ts"),
                }],
                package_root: package.clone(),
                package_link_root: package,
                manifest_path: manifest.clone(),
                package_name: Some("leaf-widget".into()),
                workspace_source: false,
                nested_routes: Vec::new(),
            }),
            invalidation_paths: vec![manifest, missing.clone()],
        }]);
        let mut processed = Default::default();
        project
            .register_package_route_targets_with_visited(&mut processed)
            .unwrap();
        assert!(project.find_by_original(&missing).is_none());
        assert!(processed.contains(&missing));

        write(&missing, LEAF);
        project
            .register_package_route_targets_with_visited(&mut processed)
            .unwrap();
        let code = &project.find_by_original(&missing).unwrap().content;
        assert!(
            code.contains("const leafLabel = props[\"leafLabel\"]"),
            "{code}"
        );
        // A successful target is still reused by subsequent discovery waves.
        project
            .register_package_route_targets_with_visited(&mut processed)
            .unwrap();
        project.finalize_package_routes().unwrap();
        let work = project.materialize_incremental_delta().unwrap();
        assert_eq!(work.source_nodes_rebuilt, 2, "{work:?}");
        finished.send(()).unwrap();
    });
    result
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("missing package targets must not be retried forever");
    worker.join().unwrap();
}

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
