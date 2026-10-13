//! Materialized bytes and the complete map remain attached to their owner.
#![expect(
    clippy::disallowed_macros,
    reason = "complete fixture and map packets use std formatting"
)]

use super::*;

fn assert_materialized(project: &VirtualProject) -> serde_json::Value {
    let before = project
        .virtual_files_sorted()
        .into_iter()
        .map(|file| {
            serde_json::json!({"original": file.original_path, "virtual": file.virtual_path,
            "code": file.content, "map": format!("{:?}", file.source_map)})
        })
        .collect::<Vec<_>>();
    project.materialize().unwrap();
    let after = project
        .virtual_files_sorted()
        .into_iter()
        .map(|file| {
            assert_eq!(
                fs::read(&file.virtual_path).unwrap(),
                file.content.as_bytes()
            );
            serde_json::json!({"original": file.original_path, "virtual": file.virtual_path,
            "code": file.content, "map": format!("{:?}", file.source_map)})
        })
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    let config: serde_json::Value =
        serde_json::from_slice(&fs::read(project.generated_tsconfig_path()).unwrap()).unwrap();
    assert_eq!(config["compilerOptions"]["strict"], true);
    serde_json::json!({"files": after, "config": config})
}

#[test]
fn materialized_lifecycle_restores_complete_maps_and_prunes_removed_target() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("tsconfig.json"),
        "{\"compilerOptions\":{\"strict\":true,\"moduleResolution\":\"bundler\"}}\n",
    )
    .unwrap();
    let mut project = project(root.path());
    register(&mut project, "Panel.vue", EMPTY);
    register(&mut project, "index.ts", BARREL);
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
    let initial = assert_materialized(&project);
    register(
        &mut project,
        "Panel.vue",
        "<script lang=\"ts\">export type Authored = number;</script><template />",
    );
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    let named = assert_materialized(&project);
    assert_ne!(named["files"], initial["files"]);
    assert_eq!(named["config"], initial["config"]);
    register(&mut project, "Panel.vue", EMPTY);
    project.finalize_module_imports();
    assert_eq!(assert_materialized(&project), initial);
    let original = project.project_root().join("Panel.vue");
    let removed = project.remove_registered_source(&original);
    fs::remove_file(&original).unwrap();
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    let without_target = assert_materialized(&project);
    assert_eq!(without_target["files"].as_array().unwrap().len(), 1);
    for path in removed {
        assert!(!path.exists());
    }
    register(&mut project, "Panel.vue", EMPTY);
    project.finalize_module_imports();
    assert_eq!(assert_materialized(&project), initial);
}

#[test]
fn tsx_vue_star_uses_only_its_actual_owned_ts_forwarding_companion() {
    let root = tempfile::tempdir().unwrap();
    let mut project = project(root.path());
    let source = "<script setup lang=\"tsx\">const node = <div />;</script><template />";
    register(&mut project, "Panel.vue", source);
    register(&mut project, "index.ts", BARREL);
    let original = project.project_root().join("Panel.vue");
    let target = project.find_by_original(&original).unwrap();
    assert_eq!(
        target.virtual_path,
        project.virtual_root().join("Panel.vue.tsx")
    );
    let owner = &project.source_artifacts[&original];
    assert_eq!(
        owner.virtual_paths,
        [
            project.virtual_root().join("Panel.vue.ts"),
            project.virtual_root().join("Panel.vue.tsx")
        ]
    );
    assert!(owner.empty_module_exports);
    let complete_target = owner_packet(&project, "Panel.vue");
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
    assert_eq!(owner_packet(&project, "Panel.vue"), complete_target);
    let mirror = project.virtual_root().join("Panel.vue.ts");
    let removed_shim = project.virtual_files.remove(&mirror).unwrap();
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    project.virtual_files.insert(mirror.clone(), removed_shim);
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
    assert_eq!(owner_packet(&project, "Panel.vue"), complete_target);
    let original_shim = project.virtual_files[&mirror].content.clone();
    project.virtual_files.get_mut(&mirror).unwrap().content =
        "export * from './Foreign.vue.tsx';\n".into();
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    project.virtual_files.get_mut(&mirror).unwrap().content = original_shim;
    let duplicate = project.virtual_root().join("Duplicate.vue.tsx");
    register(&mut project, "Duplicate.vue", source);
    let mut extra = project.virtual_files.remove(&duplicate).unwrap();
    project.remove_registered_source(&project.project_root().join("Duplicate.vue"));
    fs::remove_file(project.project_root().join("Duplicate.vue")).unwrap();
    extra.original_path = original.clone();
    project.virtual_files.insert(duplicate.clone(), extra);
    project
        .source_artifacts
        .get_mut(&original)
        .unwrap()
        .virtual_paths
        .push(duplicate.clone());
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    project.virtual_files.remove(&duplicate);
    project
        .source_artifacts
        .get_mut(&original)
        .unwrap()
        .virtual_paths
        .retain(|path| path != &duplicate);
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
    assert_eq!(owner_packet(&project, "Panel.vue"), complete_target);
}
