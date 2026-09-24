#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

mod content_mapper_declaration_map_support;
use content_mapper_declaration_map_support::assert_authored_vue_declaration_map;

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";

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

fn install_packages(project_root: &Path) {
    let mapper_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&mapper_root).unwrap();
    std::fs::write(
        mapper_root.join("package.json"),
        serde_json::to_vec_pretty(&json!({
            "name": "vize",
            "private": true,
            "typescript": { "contentMapper": {
                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "compilerOptions": ["noUnusedLocals"],
            } },
        }))
        .unwrap(),
    )
    .unwrap();

    let vue_source = std::env::var_os(VUE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(workspace_vue_package);
    assert!(vue_source.join("package.json").is_file());
    let vue_target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue_source, vue_target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue_source, vue_target).unwrap();
}

fn workspace_vue_package() -> PathBuf {
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

fn prepare_project() -> tempfile::TempDir {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-incremental-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());
    project
}

fn run_build(tsgo: &Path, project_root: &Path) -> Output {
    Command::new(tsgo)
        .current_dir(project_root)
        .args([
            "--build",
            "references/tsconfig.json",
            "--runExternalCode",
            "--pretty",
            "false",
            "--verbose",
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", tsgo.display()))
}

fn output_text(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8 stdout>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8 stderr>")
    )
}

fn read_build_info(project_root: &Path, path: &str) -> Value {
    serde_json::from_slice(&std::fs::read(project_root.join(path)).unwrap()).unwrap()
}

fn assert_build_info_sources(build_info: &Value, expected: &[&str]) {
    let file_names = build_info["fileNames"].as_array().expect("fileNames");
    let actual = file_names
        .iter()
        .map(|value| value.as_str().expect("fileNames entries"))
        .collect::<Vec<_>>();
    let missing = expected
        .iter()
        .copied()
        .filter(|expected_path| {
            actual
                .iter()
                .all(|actual_path| actual_path != expected_path)
        })
        .collect::<Vec<_>>();
    assert_eq!(missing, Vec::<&str>::new(), "fileNames: {actual:#?}");

    let leaked = actual
        .iter()
        .copied()
        .filter(|path| {
            path_has_segment(path, "__vize") || path_has_segments(path, &["node_modules", ".vize"])
        })
        .collect::<Vec<_>>();
    assert_eq!(leaked, Vec::<&str>::new(), "fileNames: {actual:#?}");
}

fn path_has_segment(path: &str, segment: &str) -> bool {
    path.split('/').any(|part| part == segment)
}

fn path_has_segments(path: &str, segments: &[&str]) -> bool {
    let parts = path.split('/').collect::<Vec<_>>();
    parts
        .windows(segments.len())
        .any(|window| window == segments)
}

fn replace_once(path: &Path, from: &str, to: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    if !source.contains(from) {
        panic!("{} missing {from:?}", path.display());
    }
    std::fs::write(path, source.replacen(from, to, 1)).unwrap();
}

fn text_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn assert_text_has_lines(text: &str, expected: &[&str]) {
    let actual = text_lines(text);
    let missing = expected
        .iter()
        .copied()
        .filter(|expected_line| {
            actual
                .iter()
                .all(|actual_line| actual_line != expected_line)
        })
        .collect::<Vec<_>>();
    assert_eq!(missing, Vec::<&str>::new(), "lines: {actual:#?}");
}

fn assert_text_lacks_lines(text: &str, stale: &[&str]) {
    let actual = text_lines(text);
    let present = stale
        .iter()
        .copied()
        .filter(|stale_line| actual.iter().any(|actual_line| actual_line == stale_line))
        .collect::<Vec<_>>();
    assert_eq!(present, Vec::<&str>::new(), "lines: {actual:#?}");
}

fn up_to_date_projects(output: &Output) -> Vec<&'static str> {
    output_text(output)
        .lines()
        .filter_map(|line| {
            let lower = line.to_ascii_lowercase();
            if !lower.contains("is up to date") {
                return None;
            }
            if line.contains("references/ui/tsconfig.json") {
                Some("references/ui/tsconfig.json")
            } else if line.contains("references/app/tsconfig.json") {
                Some("references/app/tsconfig.json")
            } else {
                Some("unknown")
            }
        })
        .collect()
}

#[test]
fn standard_tsgo_build_refreshes_authored_vue_incremental_outputs() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!(
            "skipping exact Content Mapper incremental build conformance: {TSGO_ENV} is not set"
        );
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let project = prepare_project();
    let initial = run_build(&tsgo, project.path());
    assert!(initial.status.success(), "{}", output_text(&initial));
    assert_build_info_sources(
        &read_build_info(project.path(), "references/ui/tsconfig.tsbuildinfo"),
        &["./src/Counter.vue"],
    );
    assert_build_info_sources(
        &read_build_info(project.path(), "references/app/tsconfig.tsbuildinfo"),
        &["../ui/dist/Counter.d.vue.ts", "./src/App.vue"],
    );
    let counter_declaration = project.path().join("references/ui/dist/Counter.d.vue.ts");
    let app_declaration = project.path().join("references/app/dist/App.d.vue.ts");
    assert_text_has_lines(
        &std::fs::read_to_string(&counter_declaration).unwrap(),
        &["count: number;"],
    );

    replace_once(
        &project.path().join("references/ui/src/Counter.vue"),
        "defineProps<{ count: number }>();",
        "defineProps<{ count: string }>();",
    );
    replace_once(
        &project.path().join("references/ui/src/Counter.vue"),
        "count.toFixed(0)",
        "count.toUpperCase()",
    );
    replace_once(
        &project.path().join("references/app/src/App.vue"),
        "defineProps<{ initial: number }>();",
        "defineProps<{ initial: string }>();",
    );

    let rebuilt = run_build(&tsgo, project.path());
    assert!(rebuilt.status.success(), "{}", output_text(&rebuilt));
    let counter_text = std::fs::read_to_string(&counter_declaration).unwrap();
    let app_text = std::fs::read_to_string(&app_declaration).unwrap();
    assert_text_has_lines(&counter_text, &["count: string;"]);
    assert_text_lacks_lines(&counter_text, &["count: number;"]);
    assert_text_has_lines(&app_text, &["initial: string;"]);
    assert_eq!(
        read_build_info(project.path(), "references/ui/tsconfig.tsbuildinfo")["latestChangedDtsFile"]
            .as_str(),
        Some("./dist/Counter.d.vue.ts")
    );
    assert_eq!(
        read_build_info(project.path(), "references/app/tsconfig.tsbuildinfo")
            ["latestChangedDtsFile"]
            .as_str(),
        Some("./dist/App.d.vue.ts")
    );
    let authored = assert_authored_vue_declaration_map(
        project.path(),
        "references/ui/dist/Counter.d.vue.ts",
        "Counter",
    );
    assert_text_has_lines(
        &authored.content,
        &["<button>{{ count.toUpperCase() }}</button>"],
    );

    let unchanged = run_build(&tsgo, project.path());
    assert!(unchanged.status.success(), "{}", output_text(&unchanged));
    assert_eq!(
        up_to_date_projects(&unchanged),
        vec![
            "references/ui/tsconfig.json",
            "references/app/tsconfig.json"
        ]
    );
}
