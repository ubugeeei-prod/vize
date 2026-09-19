use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};
use vize_carton::cstr;

#[test]
fn out_of_root_declarations_enforce_imported_contracts_and_report_removal() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("app");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"noEmit":true,"moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    let declaration = directory.path().join("shared.d.ts");
    std::fs::write(
        &declaration,
        "export declare function expectString(value: string): void;",
    )
    .unwrap();
    let path = root.join("App.vue");
    let source = "<script setup lang=\"ts\">\nimport { expectString } from '../shared';\nexpectString(1);\n</script>";
    std::fs::write(&path, source).unwrap();
    let checker = BatchTypeChecker::new(&root).unwrap();
    for specifier in ["../shared", "../shared.js"] {
        let source = source.replace("../shared", specifier);
        let diagnostics = checker.check_file(&path, &source).unwrap();
        assert_eq!(diagnostics.len(), 1, "{specifier}: {diagnostics:?}");
        assert_eq!(diagnostics[0].code, Some(2345));
        assert_eq!((diagnostics[0].line, diagnostics[0].column), (2, 13));
        let valid = source.replace("expectString(1)", "expectString('valid')");
        assert!(checker.check_file(&path, &valid).unwrap().is_empty());
    }
    std::fs::remove_file(declaration).unwrap();
    let diagnostics = checker.check_file(&path, source).unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Some(2307));
    assert!(diagnostics[0].message.contains("'../shared'"));
}

#[test]
fn explicit_subsets_retain_transitive_script_contracts() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"noEmit":true,"moduleResolution":"bundler"},"include":["App.vue"]}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("barrel.ts"),
        "export { expectString } from './helper';",
    )
    .unwrap();
    std::fs::write(
        root.join("helper.ts"),
        "export function expectString(value: string) { return value; }",
    )
    .unwrap();
    let path = root.join("App.vue");
    let source = "<script setup lang=\"ts\">\nimport { expectString } from './barrel';\nexpectString(1);\n</script>";
    std::fs::write(&path, source).unwrap();
    let mut checker = BatchTypeChecker::new(root).unwrap();
    checker.scan_paths(std::slice::from_ref(&path)).unwrap();
    assert_eq!(
        checker.file_count(),
        3,
        "{:?}",
        checker
            .virtual_files()
            .iter()
            .map(|f| (&f.original_path, &f.content))
            .collect::<Vec<_>>()
    );
    let result = checker.check_project().unwrap();
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code,
        Some(2345),
        "{:?}",
        result.diagnostics
    );
    let diagnostics = checker.check_file(&path, source).unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Some(2345));
}

#[test]
fn runtime_extensions_preserve_declaration_identity_and_priority() {
    for (runtime, declaration) in [("js", "d.ts"), ("mjs", "d.mts"), ("cjs", "d.cts")] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("app");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"noEmit":true,"moduleResolution":"bundler"}}"#,
        )
        .unwrap();
        std::fs::write(directory.path().join(cstr!("shared.{runtime}")), "").unwrap();
        std::fs::write(
            directory.path().join(cstr!("shared.{declaration}")),
            "export declare function expectString(value: string): void;",
        )
        .unwrap();
        let path = root.join("App.vue");
        let source = cstr!(
            "<script setup lang=\"ts\">\nimport {{ expectString }} from '../shared.{runtime}';\nexpectString(1);\n</script>"
        );
        std::fs::write(&path, &source).unwrap();
        let checker = BatchTypeChecker::new(&root).unwrap();
        let diagnostics = checker.check_file(&path, &source).unwrap();
        assert_eq!(diagnostics.len(), 1, "{runtime}: {diagnostics:?}");
        assert_eq!(
            diagnostics[0].code,
            Some(2345),
            "{runtime}: {diagnostics:?}"
        );
    }
}
