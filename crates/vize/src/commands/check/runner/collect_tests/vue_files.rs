use super::*;

#[test]
fn collect_vue_files_stays_vue_only() {
    let case_dir = unique_case_dir("collect-vue");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "").unwrap();
    fs::write(case_dir.join("src/main.ts"), "").unwrap();

    let files = collect_vue_files(&vec![case_dir.display().to_string()]);

    assert_eq!(files, vec![case_dir.join("src/App.vue")]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn collect_vue_files_filters_quoted_globs() {
    let case_dir = unique_case_dir("collect-vue-glob");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/nested")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "").unwrap();
    fs::write(case_dir.join("src/nested/View.vue"), "").unwrap();
    fs::write(case_dir.join("src/nested/Skip.vue"), "").unwrap();

    let files = collect_vue_files(&vec![
        case_dir.join("src/nested/*.vue").display().to_string(),
    ]);

    assert_eq!(
        files,
        vec![
            case_dir.join("src/nested/Skip.vue"),
            case_dir.join("src/nested/View.vue"),
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn collect_vue_files_recursive_globs_include_dot_directories() {
    let case_dir = unique_case_dir("collect-vue-dot-directory-glob");
    let _ = fs::remove_dir_all(&case_dir);
    let download_page = write_file(&case_dir, "docs/.vitepress/components/DownloadPage.vue", "");

    let files = collect_vue_files(&vec![case_dir.join("**/*.vue").display().to_string()]);

    assert_eq!(files, vec![download_page]);

    let _ = fs::remove_dir_all(&case_dir);
}
