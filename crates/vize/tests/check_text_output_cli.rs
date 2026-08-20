#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::cstr;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root()))
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    for (path, source) in files {
        let target = project_root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, source).unwrap();
    }
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}
"#,
    )
    .unwrap();
    project_root
}

#[test]
fn check_text_output_is_plain_when_captured() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "text-output-plain",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        )],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .env("NO_COLOR", "1")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FORCE_COLOR")
        .args(["check", "."])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("\u{1b}["),
        "captured stdout must be parser-friendly plain text:\n{stdout}"
    );
    assert!(
        !stderr.contains("\u{1b}["),
        "captured stderr must be parser-friendly plain text:\n{stderr}"
    );
    assert!(
        stdout.contains("src/App.vue"),
        "stdout should retain file headings:\n{stdout}"
    );
    assert!(
        stdout.contains("error:2:7 [TS2322]"),
        "stdout should retain parseable diagnostics:\n{stdout}"
    );
    assert!(
        stdout.contains("Type checked") && stdout.contains("1 error(s)"),
        "stdout should retain the text summary:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
