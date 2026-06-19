use std::{path::Path, process::Command};

use vize_carton::cstr;

#[test]
fn check_without_patterns_resolves_imports_outside_tsconfig_include_for_types() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["fileCount"], 1, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert_eq!(
        json["errorCount"], 0,
        "transitive import should be registered for type resolution; stdout:\n{stdout}\nstderr:\n{stderr}"
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
        "export const ITEMS = [{ code: 'en', name: 'English' }, { code: 'ru', name: 'Russian' }]\n",
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
