use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ALPHA: &str = "<template><div>alpha</div></template>";
const BETA: &str = "<template><div>beta</div></template>";

#[test]
fn build_preserves_paths_for_duplicate_basenames() {
    let project = tempfile::tempdir().unwrap();
    write_file(project.path(), "src/a/index.vue", ALPHA.as_bytes());
    write_file(project.path(), "src/b/index.vue", BETA.as_bytes());

    let output = run_build(project.path(), &["src"], "dist", &[]);

    assert_success(&output);
    assert!(project.path().join("dist/a/index.js").is_file());
    assert!(project.path().join("dist/b/index.js").is_file());
    assert!(!project.path().join("dist/index.js").exists());
    let stderr = std::str::from_utf8(&output.stderr)
        .unwrap()
        .replace('\\', "/");
    let first = stderr
        .find("Built: src/a/index.vue -> dist/a/index.js")
        .unwrap();
    let second = stderr
        .find("Built: src/b/index.vue -> dist/b/index.js")
        .unwrap();
    assert!(first < second, "{stderr}");
}

#[test]
fn build_multiple_roots_is_independent_of_pattern_order() {
    let project = tempfile::tempdir().unwrap();
    write_file(
        project.path(),
        "packages/alpha/src/index.vue",
        ALPHA.as_bytes(),
    );
    write_file(
        project.path(),
        "packages/beta/src/index.vue",
        BETA.as_bytes(),
    );
    let forward_patterns = ["packages/alpha/src", "packages/beta/src/**/*.vue"];
    let reverse_patterns = ["packages/beta/src/**/*.vue", "packages/alpha/src"];

    let forward = run_build(project.path(), &forward_patterns, "dist-forward", &[]);
    let reverse = run_build(project.path(), &reverse_patterns, "dist-reverse", &[]);

    assert_success(&forward);
    assert_success(&reverse);
    let forward_files = output_files(&project.path().join("dist-forward"));
    let reverse_files = output_files(&project.path().join("dist-reverse"));
    assert_eq!(forward_files, reverse_files);
    assert_eq!(
        forward_files
            .iter()
            .map(|(path, _)| path.as_path())
            .collect::<Vec<_>>(),
        [
            Path::new("alpha/src/index.js"),
            Path::new("beta/src/index.js")
        ]
    );
}

#[test]
fn build_rejects_fixed_extension_collision_before_compiling_or_writing() {
    let project = tempfile::tempdir().unwrap();
    write_file(project.path(), "src/a.vue", &[0xff]);
    write_file(project.path(), "src/a.js/index.vue", &[0xff]);

    let output = run_build(project.path(), &["src"], "dist", &[]);

    assert!(!output.status.success());
    let stderr = std::str::from_utf8(&output.stderr)
        .unwrap()
        .replace('\\', "/");
    assert!(stderr.contains("output collision"), "{stderr}");
    assert!(stderr.contains("src/a.vue -> dist/a.js"), "{stderr}");
    assert!(
        stderr.contains("src/a.js/index.vue -> dist/a.js/index.js"),
        "{stderr}"
    );
    assert!(!stderr.contains("error(s) occurred"), "{stderr}");
    assert!(!project.path().join("dist").exists());
}

#[test]
fn build_rejects_preserved_extension_collision_before_first_write() {
    let project = tempfile::tempdir().unwrap();
    write_file(project.path(), "src/a.vue", ALPHA.as_bytes());
    write_file(project.path(), "src/a.js/index.vue", BETA.as_bytes());

    let output = run_build(
        project.path(),
        &["src"],
        "dist",
        &["--script-ext", "preserve"],
    );

    assert!(!output.status.success());
    let stderr = std::str::from_utf8(&output.stderr)
        .unwrap()
        .replace('\\', "/");
    assert!(stderr.contains("output collision"), "{stderr}");
    assert!(!project.path().join("dist").exists());
}

#[test]
fn build_reports_write_failures_and_finishes_independent_outputs() {
    let project = tempfile::tempdir().unwrap();
    write_file(project.path(), "src/a.vue", ALPHA.as_bytes());
    write_file(project.path(), "src/b.vue", BETA.as_bytes());
    fs::create_dir_all(project.path().join("dist/a.js")).unwrap();

    let output = run_build(project.path(), &["src"], "dist", &[]);

    assert!(!output.status.success());
    let stderr = std::str::from_utf8(&output.stderr)
        .unwrap()
        .replace('\\', "/");
    assert!(
        stderr.contains("Failed to write src/a.vue -> dist/a.js"),
        "{stderr}"
    );
    assert!(project.path().join("dist/b.js").is_file(), "{stderr}");
    assert!(stderr.contains("Built: src/b.vue -> dist/b.js"), "{stderr}");
}

fn run_build(root: &Path, patterns: &[&str], output: &str, extra: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .arg("build")
        .arg("--format")
        .arg("js");
    for pattern in patterns {
        command.arg(pattern);
    }
    command
        .arg("--output")
        .arg(output)
        .args(extra)
        .output()
        .unwrap()
}

fn write_file(root: &Path, relative: &str, content: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn output_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    collect_output_files(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn collect_output_files(root: &Path, directory: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_output_files(root, &path, files);
        } else {
            files.push((
                path.strip_prefix(root).unwrap().to_path_buf(),
                fs::read(path).unwrap(),
            ));
        }
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        std::str::from_utf8(&output.stdout).unwrap(),
        std::str::from_utf8(&output.stderr).unwrap()
    );
}
