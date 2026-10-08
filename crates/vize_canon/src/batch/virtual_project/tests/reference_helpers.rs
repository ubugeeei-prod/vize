//! Vue helper membership follows incremental project source membership.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]
use super::super::super::{SHARED_HELPERS_FILE, VirtualProject};

#[test]
fn plain_ts_to_vue_to_plain_ts_updates_shared_helpers() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let script = root.join("a.ts");
    let component = root.join("App.vue");
    let authored = "export const text: string = 'source';\n";
    std::fs::write(&script, authored).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_path(&script).unwrap();
    project.materialize().unwrap();
    let helpers = project.virtual_root().join(SHARED_HELPERS_FILE);
    assert!(!helpers.exists());
    project.discard_incremental_materialization();
    std::fs::write(
        &component,
        "<script setup lang=\"ts\">const count = 1;</script><template>{{ count }}</template>\n",
    )
    .unwrap();
    project.register_path(&component).unwrap();
    let added = project.materialize_incremental_delta().unwrap();
    assert!(added.delta.created.contains(&helpers));
    assert!(
        std::fs::read_to_string(&helpers)
            .unwrap()
            .contains("__vize_defineProps")
    );
    project.register_path(&component).unwrap();
    let unchanged = project.materialize_incremental_delta().unwrap();
    assert_eq!(unchanged.considered, 1);
    assert!(unchanged.delta.is_empty());
    project.remove_source_and_dependencies(&component);
    let removed = project.materialize_incremental_delta().unwrap();
    assert!(removed.delta.deleted.contains(&helpers));
    assert!(!helpers.exists());
    let config: serde_json::Value = serde_json::from_slice(
        &std::fs::read(project.virtual_root().join("tsconfig.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        config["include"],
        serde_json::json!(["__vize_vue_modules.d.ts", "a.ts"])
    );
    assert_eq!(std::fs::read_to_string(script).unwrap(), authored);
}

#[test]
fn nested_config_owns_the_same_helpers_in_cold_sharded_and_incremental_views() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let app = root.join("apps/web");
    let shared = root.join("shared");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::create_dir_all(&shared).unwrap();
    let config = app.join("tsconfig.json");
    let script = shared.join("plain.ts");
    let component = app.join("Counter.vue");
    std::fs::write(
        &config,
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"Bundler"}}"#,
    )
    .unwrap();
    std::fs::write(&script, "export const plain = 1;\n").unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_tsconfig_path(Some(config));
    project.register_path(&script).unwrap();
    project.materialize().unwrap();
    let helpers = project.virtual_root().join("apps/web/__vize_helpers.d.ts");
    let old_helpers = project.virtual_root().join(SHARED_HELPERS_FILE);
    assert_eq!(project.shared_helpers_path(), helpers);
    assert!(!helpers.exists());
    assert!(!old_helpers.exists());
    project.discard_incremental_materialization();
    std::fs::write(&component, include_str!("../../../../../../tests/_fixtures/differential/typechecker/config-scoped-vue-helpers/apps/web/Counter.vue.txt")).unwrap();
    project.register_path(&component).unwrap();
    let added = project.materialize_incremental_delta().unwrap();
    assert!(added.delta.created.contains(&helpers));
    assert!(project.expected_materialized_files().contains(&helpers));
    assert!(!project.expected_materialized_files().contains(&old_helpers));
    assert!(!old_helpers.exists());
    let helper_source = std::fs::read(&helpers).unwrap();
    let main_config = std::fs::read(project.generated_tsconfig_path()).unwrap();
    let config: serde_json::Value = serde_json::from_slice(&main_config).unwrap();
    assert_eq!(
        config["include"],
        serde_json::json!([
            "../../__vize_vue_modules.d.ts",
            "../../apps/web/Counter.vue.ts",
            "../../apps/web/__vize_helpers.d.ts",
            "../../shared/plain.ts"
        ])
    );
    let virtual_component = project.virtual_root().join("apps/web/Counter.vue.ts");
    let shard = project
        .write_shard_tsconfig(0, &[virtual_component.as_path()])
        .unwrap();
    let shard_config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(shard).unwrap()).unwrap();
    assert_eq!(
        shard_config["include"],
        serde_json::json!([
            "../../__vize_vue_modules.d.ts",
            "../../apps/web/Counter.vue.ts",
            "../../apps/web/__vize_helpers.d.ts"
        ])
    );
    project.materialize().unwrap();
    assert_eq!(std::fs::read(&helpers).unwrap(), helper_source);
    assert_eq!(
        std::fs::read(project.generated_tsconfig_path()).unwrap(),
        main_config
    );
    project.discard_incremental_materialization();
    project.register_path(&component).unwrap();
    let unchanged = project.materialize_incremental_delta().unwrap();
    assert_eq!(unchanged.considered, 1);
    assert!(unchanged.delta.is_empty());
    project.remove_source_and_dependencies(&component);
    let removed = project.materialize_incremental_delta().unwrap();
    assert!(removed.delta.deleted.contains(&helpers));
    assert!(!helpers.exists());
    assert!(!old_helpers.exists());
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(project.generated_tsconfig_path()).unwrap()).unwrap();
    assert_eq!(
        config["include"],
        serde_json::json!(["../../__vize_vue_modules.d.ts", "../../shared/plain.ts"])
    );
}
