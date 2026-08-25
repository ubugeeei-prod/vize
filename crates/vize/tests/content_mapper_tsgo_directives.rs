//! Exact-upstream conformance for template diagnostic directives:
//! `<!-- @vue-expect-error -->` and `<!-- @vue-ignore -->` travel to TypeScript
//! through the content-mapper protocol's `diagnosticDirectives`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::json;
use vize_s0::cstr;

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";

#[test]
fn template_directives_suppress_and_report_through_standard_tsgo() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper directive conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-directives-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let suppressed = check_project(&tsgo, project.path(), "tsconfig.directives.json");
    assert!(
        suppressed.status.success(),
        "directive-suppressed fixture failed unexpectedly:\n{}",
        output_text(&suppressed)
    );

    let unused = check_project(&tsgo, project.path(), "tsconfig.directives-unused.json");
    assert!(
        !unused.status.success(),
        "unused expect directive fixture passed unexpectedly:\n{}",
        output_text(&unused)
    );
    let unused_stdout = std::str::from_utf8(&unused.stdout).unwrap();
    let diagnostic_lines: Vec<&str> = unused_stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(
        diagnostic_lines,
        ["directives/UnusedExpect.vue(6,8): error vize4: Unused '@vue-expect-error' directive"],
        "unused directive fixture must report exactly the vize4 diagnostic:\n{}",
        output_text(&unused)
    );
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn copy_fixture(source: &Path, destination: &Path) {
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

fn install_mapper_manifest(project_root: &Path) {
    let package_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&package_root).unwrap();
    let manifest = json!({
        "name": "vize",
        "private": true,
        "typescript": {
            "contentMapper": {
                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "compilerOptions": ["noUnusedLocals"],
            },
        },
    });
    std::fs::write(
        package_root.join("package.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn install_vue_package(project_root: &Path) {
    let source = if let Some(path) = std::env::var_os(VUE_ENV).map(PathBuf::from) {
        path
    } else {
        let store = workspace_root().join("node_modules/.pnpm");
        let mut candidates = std::fs::read_dir(&store)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", store.display()))
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
            .map(|entry| entry.path().join("node_modules/vue"))
            .filter(|path| path.join("package.json").is_file())
            .collect::<Vec<_>>();
        candidates.sort();
        candidates
            .pop()
            .unwrap_or_else(|| panic!("no Vue 3 package under {}; set {VUE_ENV}", store.display()))
    };
    let target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn check_project(tsgo: &Path, project_root: &Path, config: &str) -> Output {
    Command::new(tsgo)
        .current_dir(project_root)
        .args([
            "--runExternalCode",
            "--noEmit",
            "-p",
            config,
            "--pretty",
            "false",
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", tsgo.display()))
}

fn output_text(output: &Output) -> vize_s0::String {
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8 stdout>");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8 stderr>");
    cstr!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr,
    )
}
