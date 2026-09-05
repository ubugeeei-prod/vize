use super::collect_files;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use vize_s0::{String, ToCompactString};

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

#[test]
fn explicit_relative_glob_respects_nested_gitignore() {
    let cwd = std::env::current_dir().unwrap();
    let relative_root =
        PathBuf::from("tests").join(unique_case_dir("explicit-glob-ignore").file_name().unwrap());
    let root = cwd.join(&relative_root);
    let source = root.join("apps/web/src/App.vue");
    let dependency = root.join("apps/web/node_modules/dependency/Hidden.vue");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::create_dir_all(dependency.parent().unwrap()).unwrap();
    fs::write(&source, "<template><main /></template>").unwrap();
    fs::write(&dependency, "<template><aside /></template>").unwrap();
    fs::write(root.join("apps/web/.gitignore"), "node_modules/\n").unwrap();

    let pattern = relative_root
        .join("apps/**/*.vue")
        .to_string_lossy()
        .into_owned();
    let files = collect_files(&[pattern], None);
    let _ = fs::remove_dir_all(&root);

    assert_eq!(files, vec![relative_root.join("apps/web/src/App.vue")]);
}

#[test]
fn relative_recursive_globs_include_dot_directories() {
    let cwd = std::env::current_dir().unwrap();
    let relative_root =
        PathBuf::from("tests").join(unique_case_dir("hidden-glob").file_name().unwrap());
    let root = cwd.join(&relative_root);
    let source = root.join("src/App.vue");
    let hidden = root.join("docs/.vitepress/components/DownloadPage.vue");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::create_dir_all(hidden.parent().unwrap()).unwrap();
    fs::write(&source, "<template><main /></template>").unwrap();
    fs::write(&hidden, "<template><aside /></template>").unwrap();

    let implicit = collect_files(&[relative_root.join("**/*.vue").to_string_lossy()], None);
    let explicit = collect_files(
        &[relative_root
            .join("docs/.vitepress/components/**/*.vue")
            .to_string_lossy()],
        None,
    );
    let _ = fs::remove_dir_all(&root);

    let mut expected = vec![
        relative_root.join("docs/.vitepress/components/DownloadPage.vue"),
        relative_root.join("src/App.vue"),
    ];
    expected.sort();
    assert_eq!(implicit, expected);
    assert_eq!(
        explicit,
        vec![relative_root.join("docs/.vitepress/components/DownloadPage.vue")]
    );
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
