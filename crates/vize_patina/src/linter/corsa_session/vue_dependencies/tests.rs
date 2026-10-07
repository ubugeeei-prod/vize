use super::{materialize, prepare};
use vize_l0::{String, cstr};

#[test]
fn real_component_imports_have_private_mirrors_and_keep_query_offsets() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding/component-values");
    let child = fixtures.join("ChildPanel.vue");
    let missing = fixtures.join("DefinitelyMissing.vue");
    let source = cstr!(
        "import Child from '{}';\nimport Missing from '{}';\nconst value = Child;\n",
        child.display(),
        missing.display()
    );
    let session = fixtures.join("private-test-session");
    let mut cache = vize_l0::FxHashMap::default();
    let prepared = prepare(
        &source,
        &fixtures.join("ParentPanel.vue").to_string_lossy(),
        &session,
        &fixtures,
        &mut cache,
    );
    assert_eq!(prepared.files.len(), 1);
    let (target, component) = &prepared.files[0];
    assert!(target.starts_with(&session));
    assert!(component.contains("export default __vize_component__"));
    let rewritten = prepared.code.expect("component import was rewritten");
    assert!(rewritten.contains(&*cstr!("from '{}'", target.display())));
    assert!(rewritten.contains(&*cstr!("from '{}'", missing.display())));
    let before = source.rfind("Child;").unwrap() as u32;
    let after = rewritten.rfind("Child;").unwrap() as u32;
    assert_eq!(prepared.source_map.get_virtual_offset(before), after);
}

#[test]
fn component_mirror_refreshes_report_actual_creations_changes_and_deletions() {
    let root = std::env::temp_dir().join(&*cstr!(
        "vize-patina-component-mirror-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = root.join("Child.vue.patina.ts");
    let files = vec![(path.clone(), String::from("export default 1;\n"))];
    let created = materialize(&files, &[]).expect("create component mirror");
    assert_eq!(
        (
            created.created.len(),
            created.changed.len(),
            created.deleted.len()
        ),
        (1, 0, 0)
    );
    let unchanged =
        materialize(&files, std::slice::from_ref(&path)).expect("unchanged component mirror");
    assert_eq!(
        (
            unchanged.created.len(),
            unchanged.changed.len(),
            unchanged.deleted.len()
        ),
        (0, 0, 0)
    );
    let changed_files = vec![(path.clone(), String::from("export default 2;\n"))];
    let changed =
        materialize(&changed_files, std::slice::from_ref(&path)).expect("changed component mirror");
    assert_eq!(
        (
            changed.created.len(),
            changed.changed.len(),
            changed.deleted.len()
        ),
        (0, 1, 0)
    );
    let deleted = materialize(&[], std::slice::from_ref(&path)).expect("removed component mirror");
    assert_eq!(
        (
            deleted.created.len(),
            deleted.changed.len(),
            deleted.deleted.len()
        ),
        (0, 0, 1)
    );
    assert!(!path.exists());
    std::fs::remove_dir_all(root).expect("remove private mirror test directory");
}
