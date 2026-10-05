use super::collect_files;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use vize_l0::{String, ToCompactString};

#[test]
fn formatting_never_selects_git_metadata_even_when_explicitly_named() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let source = root.join(".github/App.vue");
    let metadata = root.join(".git/worktrees/cache/App.vue");
    for file in [&source, &metadata] {
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, "<template><div/></template>").unwrap();
    }
    for input in [
        root.display().to_string(),
        root.join("**/*.vue").display().to_string(),
    ] {
        assert_eq!(collect_files(&[input], None), vec![source.clone()]);
    }
    for input in [
        metadata.display().to_string(),
        root.join(".git").display().to_string(),
        root.join("**/.git/**/*.vue").display().to_string(),
    ] {
        assert!(collect_files(&[input], None).is_empty());
    }
}

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

#[test]
fn discovery_prunes_root_and_nested_installed_dependencies() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let source = root.join("src/main.ts");
    let dependencies = [
        root.join("node_modules/dep/index.ts"),
        root.join("apps/web/node_modules/dep/index.ts"),
        root.join("node_modules/.vize/corsa-overlay/index.ts"),
    ];
    for file in std::iter::once(&source).chain(&dependencies) {
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, "const   a=1\n").unwrap();
    }
    for pattern in [root.to_path_buf(), root.join("**/*.ts")] {
        assert_eq!(
            collect_files(&[pattern.to_string_lossy()], None),
            vec![source.clone()]
        );
    }
    for dependency in dependencies {
        assert!(collect_files(&[dependency.to_string_lossy()], None).is_empty());
    }
}

#[test]
fn collect_files_includes_existing_bracket_paths() {
    let root = unique_case_dir("bracket-pages");
    let dynamic = root.join("pages/[id].vue");
    let nested = root.join("pages/[id]/index.vue");
    let plain = root.join("pages/plain.vue");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(nested.parent().unwrap()).unwrap();
    let body = "<script setup lang=\"ts\">\nconst a = {b:1}\n</script>\n";
    fs::write(&dynamic, body).unwrap();
    fs::write(&nested, body).unwrap();
    fs::write(&plain, body).unwrap();

    let files = collect_files(
        &[
            dynamic.to_string_lossy().into_owned(),
            plain.to_string_lossy().into_owned(),
            nested.to_string_lossy().into_owned(),
        ],
        None,
    );
    let _ = fs::remove_dir_all(&root);

    let mut expected = vec![dynamic, nested, plain];
    expected.sort();
    assert_eq!(files, expected);
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
