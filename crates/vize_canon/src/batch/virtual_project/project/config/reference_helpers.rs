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
    assert_eq!(
        added.delta.created,
        [project.virtual_root().join("App.vue.ts"), helpers.clone()]
    );
    assert_eq!(
        std::fs::read_to_string(&helpers).unwrap(),
        crate::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
    );
    project.register_path(&component).unwrap();
    let unchanged = project.materialize_incremental_delta().unwrap();
    assert_eq!(unchanged.considered, 1);
    assert!(unchanged.delta.is_empty());
    project.remove_source_and_dependencies(&component);
    let removed = project.materialize_incremental_delta().unwrap();
    assert_eq!(
        removed.delta.deleted,
        [project.virtual_root().join("App.vue.ts"), helpers.clone()]
    );
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
fn editor_projects_materialize_one_shared_helper_program() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    let source = |name: &str| {
        vize_carton::cstr!(
            "<script setup lang=\"ts\">const {name} = 1;</script><template>{{{{ {name} }}}}</template>\n"
        )
    };
    std::fs::write(&first, source("first").as_str()).unwrap();
    std::fs::write(&second, source("second").as_str()).unwrap();

    let mut project = VirtualProject::new(&root).unwrap();
    project.set_editor_document_options(Default::default());
    project
        .register_paths(&[first.clone(), second.clone()])
        .unwrap();

    for path in [&first, &second] {
        let generated = &project.find_by_original(path).unwrap().content;
        assert_eq!(generated.matches("Shared preamble hoisted").count(), 1);
        assert_eq!(generated.matches("type $IsUnion<").count(), 0);
        assert_eq!(generated.matches("type $ComponentInstanceFor<").count(), 1);
        assert_eq!(generated.matches("type $ComponentInputProps<").count(), 1);
        assert_eq!(generated.matches("type $ReplaceComponentProps<").count(), 1);
        assert_eq!(
            generated
                .matches("export type { __RuntimePropShape, __LooseRequired")
                .count(),
            1
        );
    }

    project.materialize().unwrap();
    let helpers = project.shared_helpers_path();
    assert_eq!(
        std::fs::read_to_string(&helpers).unwrap(),
        project.shared_helpers_preamble(),
    );
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(project.generated_tsconfig_path()).unwrap()).unwrap();
    assert!(
        config["include"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry == "__vize_helpers.d.ts")
    );
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
    std::fs::write(&component, include_str!("../../../../../../../tests/_fixtures/differential/typechecker/config-scoped-vue-helpers/apps/web/Counter.vue.txt")).unwrap();
    project.register_path(&component).unwrap();
    let added = project.materialize_incremental_delta().unwrap();
    assert_eq!(
        added.delta.created,
        [
            project.virtual_root().join("apps/web/Counter.vue.ts"),
            helpers.clone()
        ]
    );
    let mut expected_files = project
        .expected_materialized_files()
        .into_iter()
        .collect::<Vec<_>>();
    expected_files.sort();
    assert_eq!(
        expected_files,
        [
            "__vize_vue_modules.d.ts",
            "apps/web/Counter.vue.ts",
            "apps/web/__vize_helpers.d.ts",
            "apps/web/tsconfig.json",
            "package.json",
            "shared/plain.ts",
        ]
        .map(|path| project.virtual_root().join(path))
    );
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
    // A CLI shard config lists its existing roots as `files`.
    assert_eq!(
        shard_config["files"],
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
    assert_eq!(
        removed.delta.deleted,
        [
            project.virtual_root().join("apps/web/Counter.vue.ts"),
            helpers.clone()
        ]
    );
    assert!(!helpers.exists());
    assert!(!old_helpers.exists());
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(project.generated_tsconfig_path()).unwrap()).unwrap();
    assert_eq!(
        config["include"],
        serde_json::json!(["../../__vize_vue_modules.d.ts", "../../shared/plain.ts"])
    );
}
