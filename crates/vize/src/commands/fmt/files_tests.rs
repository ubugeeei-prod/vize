use super::collect_files;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use vize_carton::{String, ToCompactString};

#[test]
fn collect_files_ignores_supported_extension_directories() {
    let root = unique_case_dir("format-extension-directories");
    let src = root.join("src");
    let component_dir = src.join("Directory.vue");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&component_dir).unwrap();
    fs::write(src.join("App.vue"), "<template><div/></template>").unwrap();
    fs::write(
        component_dir.join("Nested.vue"),
        "<template><div/></template>",
    )
    .unwrap();

    let pattern = root.to_string_lossy().into_owned();
    let glob_pattern = root.join("**/*.vue").to_string_lossy().into_owned();
    let files_from_dir = collect_files(&[pattern], None);
    let files_from_glob = collect_files(&[glob_pattern], None);
    let _ = fs::remove_dir_all(&root);

    let mut expected = vec![component_dir.join("Nested.vue"), src.join("App.vue")];
    expected.sort();
    assert_eq!(files_from_dir, expected);
    assert_eq!(files_from_glob, expected);
}

fn unique_case_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut dir_name = String::from(name);
    dir_name.push('-');
    let pid = std::process::id().to_compact_string();
    dir_name.push_str(pid.as_str());
    dir_name.push('-');
    let nanos = nanos.to_compact_string();
    dir_name.push_str(nanos.as_str());
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("vize-tests")
        .join("fmt")
        .join(dir_name.as_str())
}
