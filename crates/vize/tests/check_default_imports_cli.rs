#[path = "check_default_imports_cli/aliased_diagnostics.rs"]
mod aliased_diagnostics;
#[path = "check_default_imports_cli/base_url_diagnostics.rs"]
mod base_url_diagnostics;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "check_default_imports_cli/workspace_package_diagnostics.rs"]
mod workspace_package_diagnostics;
#[path = "check_default_imports_cli/workspace_package_vue_exports.rs"]
mod workspace_package_vue_exports;

use std::{path::Path, process::Command};

use vize_s0::{cstr, path::canonicalize_non_verbatim};

#[test]
fn default_check_reports_diagnostics_from_imports_outside_tsconfig_include() {
    assert_imported_source_is_reported(&["check", "--format", "json"]);
}

#[test]
fn explicit_check_reports_diagnostics_from_imported_sources() {
    assert_imported_source_is_reported(&["check", "inside/use.ts", "--format", "json"]);
}

#[test]
fn explicit_absolute_input_reports_imported_sources_outside_cwd() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let case_root = unique_case_dir("absolute-input-outside-cwd");
    let _ = std::fs::remove_dir_all(&case_root);
    let cwd = case_root.join("cwd");
    let source_root = case_root.join("source");
    let source_file = source_root.join("src/entry.ts");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(source_file.parent().unwrap()).unwrap();
    std::fs::write(
        source_root.join("shared.ts"),
        "export const message = 'hello'\n",
    )
    .unwrap();
    std::fs::write(
        &source_file,
        "import { message } from '../shared'\nexport const value: string = message\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&cwd)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--no-config",
            source_file.to_string_lossy().as_ref(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["fileCount"], 2, "{stdout}\n{stderr}");
    let files = json["files"].as_array().expect("files should be an array");
    for expected in [source_file, source_root.join("shared.ts")] {
        let expected = canonicalize_non_verbatim(&expected).display().to_string();
        assert!(
            files.iter().any(|file| file["file"] == expected),
            "missing {expected}:\n{stdout}\n{stderr}"
        );
    }
    assert!(
        !stderr.contains("Failed to strip prefix from path"),
        "{stderr}"
    );

    let _ = std::fs::remove_dir_all(&case_root);
}

fn assert_imported_source_is_reported(args: &[&str]) {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(args)
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["fileCount"], 2, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert_eq!(
        json["errorCount"], 1,
        "imported authored sources must be diagnosed; stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let imported = json["files"]
        .as_array()
        .and_then(|files| files.iter().find(|file| file["file"] == "outside/lib.ts"))
        .unwrap_or_else(|| panic!("missing imported source result:\n{stdout}\n{stderr}"));
    assert!(
        imported["diagnostics"]
            .as_array()
            .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .as_str()
                    .is_some_and(|diagnostic| diagnostic.contains("TS2322"))
            })),
        "missing imported source diagnostic:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(project_root);
}

fn create_project() -> std::path::PathBuf {
    let project_root = unique_case_dir("default-transitive-imports-outside-include");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("inside")).unwrap();
    std::fs::create_dir_all(project_root.join("outside")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["inside/**/*.ts"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("inside/use.ts"),
        r#"import { ITEMS } from '../outside/lib'

export const r = ITEMS.map(({ code, name }) => `${code}:${name}`)
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("outside/lib.ts"),
        r#"export const ITEMS = [{ code: 'en', name: 'English' }]
const invalid: string = 42
void invalid
"#,
    )
    .unwrap();
    project_root
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    for candidate in [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ] {
        if candidate.exists() {
            return Some(candidate.display().to_string());
        }
    }

    None
}
