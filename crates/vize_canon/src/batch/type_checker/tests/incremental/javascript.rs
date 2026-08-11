use super::{BatchTypeChecker, TypeChecker, create_project_case, resolve_test_tsgo_binary};

fn diagnostic<'a>(
    diagnostics: &'a [crate::batch::Diagnostic],
    path: &std::path::Path,
    code: u32,
) -> &'a crate::batch::Diagnostic {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.file == path && diagnostic.code == Some(code))
        .unwrap_or_else(|| panic!("missing {code} for {}: {diagnostics:#?}", path.display()))
}

#[test]
fn allowjs_project_refreshes_javascript_family_lifecycle() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "incremental-allowjs-lifecycle",
        &[
            (
                "src/entry.js",
                "/** @type {number} */\nexport const entry = 1;\n",
            ),
            (
                "src/view.jsx",
                "/** @type {number} */\nexport const view = 1;\n",
            ),
            (
                "src/module.mjs",
                "/** @type {number} */\nexport const moduleValue = 1;\n",
            ),
            (
                "src/config.cjs",
                "/** @type {number} */\nconst config = 1;\nvoid config;\n",
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.base.json"),
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{ "extends": "./tsconfig.base.json", "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let entry_path = project_root.join("src/entry.js");
    let added_path = project_root.join("src/added.mjs");
    let renamed_path = project_root.join("src/renamed.cjs");
    let mut checker = BatchTypeChecker::new(&project_root).expect("checker should start");
    checker.scan_project().expect("initial scan should succeed");
    assert_eq!(
        checker.file_count(),
        4,
        "allowJs should admit every authored JavaScript extension"
    );

    let clean = checker.check_project().expect("clean check should succeed");
    assert!(
        clean
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2322)),
        "clean JavaScript family reported TS2322: {:#?}",
        clean.diagnostics
    );

    std::fs::write(
        &entry_path,
        "/** @type {number} */\nexport const entry = 'broken edit';\n",
    )
    .unwrap();
    let edited = checker
        .check_incremental(std::slice::from_ref(&entry_path))
        .expect("JavaScript edit should refresh");
    let edit_error = diagnostic(&edited.diagnostics, &entry_path, 2322);
    assert_eq!((edit_error.line, edit_error.column), (1, 13));

    std::fs::write(
        &added_path,
        "/** @type {number} */\nexport const added = 'broken create';\n",
    )
    .unwrap();
    let created = checker
        .check_incremental(std::slice::from_ref(&added_path))
        .expect("JavaScript create should refresh");
    assert!(
        checker.incremental_metrics().last_created_files > 0,
        "JavaScript create must enter the materialized delta: {:?}",
        checker.incremental_metrics()
    );
    diagnostic(&created.diagnostics, &entry_path, 2322);
    diagnostic(&created.diagnostics, &added_path, 2322);

    std::fs::rename(&added_path, &renamed_path).unwrap();
    let renamed = checker
        .check_incremental(&[added_path.clone(), renamed_path.clone()])
        .expect("JavaScript rename should refresh");
    assert!(
        renamed
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.file != added_path),
        "rename retained stale diagnostics: {:#?}",
        renamed.diagnostics
    );
    diagnostic(&renamed.diagnostics, &renamed_path, 2322);

    std::fs::write(
        &entry_path,
        "/** @type {number} */\nexport const entry = 1;\n",
    )
    .unwrap();
    std::fs::remove_file(&renamed_path).unwrap();
    let repaired = checker
        .check_incremental(&[entry_path, renamed_path.clone()])
        .expect("JavaScript repair and delete should refresh");
    assert!(
        repaired
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != Some(2322) && diagnostic.file != renamed_path),
        "repair or delete retained stale diagnostics: {:#?}",
        repaired.diagnostics
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn base_config_allowjs_and_checkjs_flips_rebuild_membership_and_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "incremental-config-javascript-policy",
        &[
            ("src/index.ts", "export const ready = true\n"),
            (
                "src/contract.js",
                "/** @type {number} */\nexport const contract = 'wrong';\n",
            ),
        ],
    );
    let base = project_root.join("tsconfig.base.json");
    let disabled = r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowJs":false,"checkJs":false},"pad":""}"#;
    let enabled = r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowJs":true,"checkJs":true},"pad":"xx"}"#;
    let unchecked = r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowJs":true,"checkJs":false},"pad":"x"}"#;
    assert_eq!(disabled.len(), enabled.len());
    assert_eq!(enabled.len(), unchecked.len());
    std::fs::write(&base, disabled).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{"extends":"./tsconfig.base.json","include":["src/**/*"]}"#,
    )
    .unwrap();
    let index = project_root.join("src/index.ts");
    let javascript = project_root.join("src/contract.js");

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.scan_project().unwrap();
    assert_eq!(checker.file_count(), 1);
    let initial = checker
        .check_incremental(std::slice::from_ref(&index))
        .unwrap();
    assert!(
        initial
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.file != javascript || diagnostic.code != Some(2322) })
    );

    replace_with_same_mtime(&base, enabled);
    let checked = checker
        .check_incremental(std::slice::from_ref(&base))
        .unwrap();
    diagnostic(&checked.diagnostics, &javascript, 2322);
    assert_eq!(checker.file_count(), 2);
    assert!(checker.incremental_metrics().last_full_rebuild);

    replace_with_same_mtime(&base, unchecked);
    let no_check = checker
        .check_incremental(std::slice::from_ref(&base))
        .unwrap();
    assert!(
        no_check
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.file != javascript || diagnostic.code != Some(2322)),
        "checkJs=false retained JavaScript diagnostics: {:#?}",
        no_check.diagnostics
    );
    assert_eq!(
        checker.file_count(),
        2,
        "allowJs=true must retain membership"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}

fn replace_with_same_mtime(path: &std::path::Path, content: &str) {
    let modified = std::fs::metadata(path).unwrap().modified().unwrap();
    std::fs::write(path, content).unwrap();
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(modified)
        .unwrap();
}
