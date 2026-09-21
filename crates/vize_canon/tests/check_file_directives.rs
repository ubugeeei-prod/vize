use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

#[test]
fn in_memory_checks_own_directives_from_the_current_document() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"noEmit":true}}"#,
    )
    .unwrap();
    let path = root.join("App.vue");
    std::fs::write(&path, "<template>{{ staleOnDisk }}</template>").unwrap();
    let checker = BatchTypeChecker::new(root).unwrap();
    let source = "<script setup lang=\"ts\">const value = 1;</script><template><div><!-- @vue-expect-error -->\n{{ missing }}</div></template>";
    let diagnostics = checker.check_file(&path, source).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let repaired = source.replace("{{ missing }}", "{{ value }}");
    let diagnostics = checker.check_file(&path, &repaired).unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Some(2578));
    let unguarded = source.replace("<!-- @vue-expect-error -->", "");
    let diagnostics = checker.check_file(&path, &unguarded).unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Some(2339));
    std::fs::remove_file(&path).unwrap();
    let diagnostics = checker.check_file(&path, &unguarded).unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Some(2339));
}
