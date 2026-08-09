use super::*;

#[test]
fn collect_check_files_honors_allowjs_for_files_directories_and_globs() {
    let case_dir = unique_case_dir("collect-check-allow-js");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/nested")).unwrap();
    for name in [
        "entry.js",
        "view.jsx",
        "module.mjs",
        "config.cjs",
        "typed.ts",
    ] {
        fs::write(case_dir.join("src").join(name), "").unwrap();
    }
    fs::write(case_dir.join("src/nested/worker.mjs"), "").unwrap();
    let options = CheckFileOptions {
        include_js: true,
        include_jsx: false,
    };

    let direct = collect_check_files_with_ignores(
        &[case_dir.join("src/entry.js").display().to_string()],
        options,
        None,
    );
    let directory = collect_check_files_with_ignores(
        &[case_dir.join("src").display().to_string()],
        options,
        None,
    );
    let glob = collect_check_files_with_ignores(
        &[case_dir.join("src/**/*.mjs").display().to_string()],
        options,
        None,
    );
    let disabled = collect_check_files_with_ignores(
        &[case_dir.join("src/entry.js").display().to_string()],
        CheckFileOptions::default(),
        None,
    );

    assert_eq!(direct, vec![case_dir.join("src/entry.js")]);
    assert_eq!(
        directory,
        vec![
            case_dir.join("src/config.cjs"),
            case_dir.join("src/entry.js"),
            case_dir.join("src/module.mjs"),
            case_dir.join("src/nested/worker.mjs"),
            case_dir.join("src/typed.ts"),
            case_dir.join("src/view.jsx"),
        ]
    );
    assert_eq!(
        glob,
        vec![
            case_dir.join("src/module.mjs"),
            case_dir.join("src/nested/worker.mjs"),
        ]
    );
    assert!(disabled.is_empty());

    let _ = fs::remove_dir_all(&case_dir);
}
