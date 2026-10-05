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
