//! Shared fixture installation and process assertions for Content Mapper conformance.

use super::{Command, CompactString, Output, Path, PathBuf, TSGO_ENV, VUE_ENV, cstr, json};

#[path = "ambient_template_ref.rs"]
mod ambient_template_ref;

pub(super) fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

pub(super) fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "node_modules" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).unwrap();
        }
    }
}

pub(super) fn install_mapper_manifest(project_root: &Path) {
    let package_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&package_root).unwrap();
    let manifest = json!({
        "name": "vize",
        "private": true,
        "typescript": {
            "contentMapper": {
                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "compilerOptions": ["checkJs", "noUnusedLocals", "rootDir"],
                "dynamicConfig": true,
            },
        },
    });
    std::fs::write(
        package_root.join("package.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn workspace_vue_package() -> PathBuf {
    if let Some(path) = std::env::var_os(VUE_ENV).map(PathBuf::from) {
        assert!(
            path.join("package.json").is_file(),
            "{VUE_ENV} does not contain package.json: {}",
            path.display()
        );
        return path;
    }

    let store = workspace_root().join("node_modules/.pnpm");
    let mut candidates = std::fs::read_dir(&store)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", store.display()))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
        .map(|entry| entry.path().join("node_modules/vue"))
        .filter(|path| path.join("package.json").is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop().unwrap_or_else(|| {
        panic!(
            "no Vue 3 package found under {}; set {VUE_ENV}",
            store.display()
        )
    })
}

pub(super) fn install_vue_package(project_root: &Path) {
    let source = workspace_vue_package();
    let target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

pub(super) fn run_tsgo(tsgo: &Path, project_root: &Path, arguments: &[&str]) -> Output {
    Command::new(tsgo)
        .current_dir(project_root)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", tsgo.display()))
}

pub(super) fn check_project(tsgo: &Path, project_root: &Path, config: &str) -> Output {
    run_tsgo(
        tsgo,
        project_root,
        &[
            "--runExternalCode",
            "--noEmit",
            "-p",
            config,
            "--pretty",
            "false",
        ],
    )
}

pub(super) fn output_text(output: &Output) -> CompactString {
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8 stdout>");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8 stderr>");
    cstr!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr,
    )
}

pub(super) fn assert_success(output: &Output) {
    assert!(output.status.success(), "{}", output_text(output));
}

pub(super) fn emitted_vue_declaration(project_root: &Path, name: &str) -> PathBuf {
    [
        project_root.join(cstr!("dist/{name}.d.vue.ts").as_str()),
        project_root.join(cstr!("dist/{name}.vue.d.ts").as_str()),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .unwrap_or_else(|| panic!("no emitted declaration for {name}.vue"))
}
