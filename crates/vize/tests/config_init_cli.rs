//! Public setup commands must work without creating a dedicated Vize config.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::fs;
use std::path::Path;
use std::process::Command;

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .expect("failed to execute the public vize CLI")
}

fn assert_no_dedicated_config(root: &Path) {
    assert!(fs::read_dir(root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("vize.config.")
    }));
}

#[test]
fn musea_new_uses_defaults_and_preserves_existing_config_files() {
    let project = tempfile::tempdir().unwrap();
    let vite = include_str!("fixtures/config-init/vite.config.mjs");
    fs::write(project.path().join("vite.config.mjs"), vite).unwrap();
    let output = run(project.path(), &["musea", "new", "gallery"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(project.path().join("stories/Button.art.vue").is_file());
    assert_eq!(
        fs::read_to_string(project.path().join("vite.config.mjs")).unwrap(),
        vite
    );
    assert_no_dedicated_config(project.path());

    let existing = "{\"musea\": {\"include\": [\"stories/**/*.art.vue\"]}}\n";
    fs::write(project.path().join("vize.config.json"), existing).unwrap();
    let output = run(project.path(), &["musea", "new"]);
    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(project.path().join("vize.config.json")).unwrap(),
        existing
    );
    assert!(!project.path().join("vize.config.ts").exists());
}

#[test]
fn library_init_writes_and_reads_vite_lib_settings_without_a_dedicated_config() {
    let project = tempfile::tempdir().unwrap();
    let output = run(
        project.path(),
        &["lib", "--json", "--offline", "init", "--ui-dir", "src/ui"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["configPath"], "vite.config.mjs");
    assert_eq!(report["action"], "create");
    assert_no_dedicated_config(project.path());

    let registry = project.path().join("registry");
    fs::create_dir_all(registry.join("files")).unwrap();
    let source = include_str!("fixtures/config-init/button.ts");
    fs::write(registry.join("files/button.ts"), source).unwrap();
    fs::write(
        registry.join("registry.json"),
        include_str!("fixtures/config-init/registry.json"),
    )
    .unwrap();
    let registry_path = registry.to_str().unwrap();
    let output = run(
        project.path(),
        &[
            "lib",
            "--offline",
            "--registry",
            registry_path,
            "pull",
            "button",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(project.path().join("src/ui/button.ts")).unwrap(),
        source
    );
    assert!(
        !project
            .path()
            .join("src/components/vize/button.ts")
            .exists()
    );
    assert_no_dedicated_config(project.path());
}

#[test]
fn library_init_preserves_an_existing_vite_configuration() {
    let project = tempfile::tempdir().unwrap();
    let vite = include_str!("fixtures/config-init/vite.config.mjs");
    fs::write(project.path().join("vite.config.mjs"), vite).unwrap();
    let output = run(project.path(), &["lib", "--json", "--offline", "init"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["action"], "unchanged");
    assert_eq!(report["configPath"], "vite.config.mjs");
    assert_eq!(
        fs::read_to_string(project.path().join("vite.config.mjs")).unwrap(),
        vite
    );
    assert_no_dedicated_config(project.path());
}
