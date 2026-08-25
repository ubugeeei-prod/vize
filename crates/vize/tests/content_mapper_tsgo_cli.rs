use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::json;
use vize_s0::{String as CompactString, cstr};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const JAVASCRIPT_TSC_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC";
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

fn install_vue_package(project_root: &Path) {
    let source = workspace_vue_package();
    let target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn run_tsgo(tsgo: &Path, project_root: &Path, arguments: &[&str]) -> Output {
    Command::new(tsgo)
        .current_dir(project_root)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", tsgo.display()))
}

fn check_project(tsgo: &Path, project_root: &Path, config: &str) -> Output {
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

fn output_text(output: &Output) -> CompactString {
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8 stdout>");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8 stderr>");
    cstr!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr,
    )
}

fn assert_success(output: &Output) {
    assert!(output.status.success(), "{}", output_text(output));
}

fn emitted_vue_declaration(project_root: &Path, name: &str) -> PathBuf {
    [
        project_root.join(cstr!("dist/{name}.d.vue.ts").as_str()),
        project_root.join(cstr!("dist/{name}.vue.d.ts").as_str()),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .unwrap_or_else(|| panic!("no emitted declaration for {name}.vue"))
}

#[test]
fn standard_tsgo_checks_vue_project_and_emits_consumable_declarations() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );
    let javascript_tsc = std::env::var_os(JAVASCRIPT_TSC_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{JAVASCRIPT_TSC_ENV} must accompany {TSGO_ENV}"));
    assert!(
        javascript_tsc.is_file(),
        "{JAVASCRIPT_TSC_ENV} is not a file: {}",
        javascript_tsc.display()
    );

    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-tsgo-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let check = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_success(&check);

    let options_enabled = check_project(&tsgo, project.path(), "tsconfig.options-api-enabled.json");
    assert_success(&options_enabled);

    let options_disabled =
        check_project(&tsgo, project.path(), "tsconfig.options-api-disabled.json");
    assert!(
        !options_disabled.status.success(),
        "Options API disabled fixture passed unexpectedly"
    );
    let options_disabled_output = output_text(&options_disabled);
    assert!(
        options_disabled_output.contains("src/Options.vue")
            && options_disabled_output.contains("TS2304")
            && options_disabled_output.contains("Cannot find name 'count'"),
        "{options_disabled_output}"
    );

    let broken = check_project(&tsgo, project.path(), "tsconfig.error.json");
    assert!(
        !broken.status.success(),
        "broken fixture passed unexpectedly"
    );
    let broken_output = output_text(&broken);
    assert!(
        broken_output.contains("errors/Broken.vue"),
        "{broken_output}"
    );
    assert!(
        broken_output.contains("TS2322")
            && broken_output.contains("not assignable to type 'number'"),
        "{broken_output}"
    );
    for script_error in ["errors/JavaScriptConsumer.js", "errors/JsxConsumer.jsx"] {
        assert!(
            broken_output
                .lines()
                .any(|line| line.contains(script_error) && line.contains("TS2322")),
            "{script_error} was not checked:\n{broken_output}"
        );
    }
    assert!(
        broken_output.contains("src/Unused.vue")
            && broken_output.contains("TS6133")
            && broken_output.contains("'unused' is declared but its value is never read"),
        "{broken_output}"
    );
    assert!(
        !broken_output.contains("'used' is declared"),
        "{broken_output}"
    );
    assert_eq!(
        broken_output.matches("TS6133").count(),
        1,
        "{broken_output}"
    );
    assert!(!broken_output.contains("TS6196"), "{broken_output}");

    let emit = run_tsgo(
        &tsgo,
        project.path(),
        &[
            "--runExternalCode",
            "-p",
            "tsconfig.emit.json",
            "--pretty",
            "false",
        ],
    );
    assert_success(&emit);

    let app_declaration = emitted_vue_declaration(project.path(), "App");
    let child_declaration = emitted_vue_declaration(project.path(), "Child");
    let options_declaration = emitted_vue_declaration(project.path(), "Options");
    let public_declaration = emitted_vue_declaration(project.path(), "Public");
    for declaration in [&app_declaration, &child_declaration] {
        let text = std::fs::read_to_string(declaration).unwrap();
        assert!(
            text.contains("$props"),
            "{}:\n{text}",
            declaration.display()
        );
        assert!(
            text.contains("count: number"),
            "{}:\n{text}",
            declaration.display()
        );
    }
    let options_text = std::fs::read_to_string(&options_declaration).unwrap();
    assert!(options_text.contains("$props"), "{options_text}");
    assert!(
        options_text.contains("declare const Options: typeof __vize_component__")
            && options_text.contains("export default Options"),
        "{options_text}"
    );
    let public_text = std::fs::read_to_string(&public_declaration).unwrap();
    for public_surface in ["$props", "$emit", "$slots", "focus", "modelValue"] {
        assert!(
            public_text.contains(public_surface),
            "missing {public_surface} in {}:\n{public_text}",
            public_declaration.display()
        );
    }

    let main_declaration = project.path().join("dist/main.d.ts");
    let main_text = std::fs::read_to_string(&main_declaration).unwrap();
    assert!(main_text.contains("./App.vue"), "{main_text}");
    assert!(main_text.contains("export type AppProps"), "{main_text}");

    std::fs::copy(
        project.path().join("consumer/verify.ts"),
        project.path().join("dist/verify.ts"),
    )
    .unwrap();
    let consume = run_tsgo(
        &tsgo,
        project.path(),
        &[
            "--ignoreConfig",
            "--noEmit",
            "--strict",
            "--module",
            "preserve",
            "--moduleResolution",
            "bundler",
            "--allowArbitraryExtensions",
            "--pretty",
            "false",
            "dist/verify.ts",
        ],
    );
    assert_success(&consume);

    let consume = run_tsgo(
        &javascript_tsc,
        project.path(),
        &[
            "--ignoreConfig",
            "--noEmit",
            "--strict",
            "--module",
            "preserve",
            "--moduleResolution",
            "bundler",
            "--allowArbitraryExtensions",
            "--pretty",
            "false",
            "dist/verify.ts",
        ],
    );
    assert_success(&consume);
}
