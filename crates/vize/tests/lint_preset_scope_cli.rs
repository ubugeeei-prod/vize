use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-lint-preset-scope-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &std::process::Output) -> vize_s0::String {
    vize_s0::cstr!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lint_without_config_uses_happy_path_instead_of_ecosystem() {
    let project_root = temp_project_dir("default-happy-path");
    write_project_file(
        &project_root,
        "src/App.vue",
        r#"<template><RouterLink to="/settings">Settings</RouterLink></template>"#,
    );

    let default = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "--format", "json", "src/App.vue"])
        .output()
        .unwrap();
    assert!(default.status.success(), "{}", output_details(&default));
    let default_json: serde_json::Value =
        serde_json::from_slice(&default.stdout).unwrap_or_else(|_| {
            panic!(
                "default happy-path lint must emit JSON: {}",
                output_details(&default)
            )
        });
    assert_eq!(
        default_json[0]["messages"],
        serde_json::json!([]),
        "{}",
        output_details(&default)
    );

    let ecosystem = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "ecosystem",
            "--no-config",
            "--format",
            "json",
            "src/App.vue",
        ])
        .output()
        .unwrap();
    assert!(
        ecosystem.status.success(),
        "explicit ecosystem should report the static RouterLink target: {}",
        output_details(&ecosystem)
    );
    assert!(
        String::from_utf8_lossy(&ecosystem.stdout)
            .contains("ecosystem/vue-router-prefer-named-link"),
        "{}",
        output_details(&ecosystem)
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_limits_nuxt_link_guidance_to_nuxt_preset() {
    let project_root = temp_project_dir("nuxt-link-preset-boundary");
    write_project_file(
        &project_root,
        "app/pages/index.vue",
        r#"<template><a href="/settings">Settings</a></template>"#,
    );
    let ecosystem = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "ecosystem",
            "--no-config",
            "--format",
            "json",
            "app/pages/index.vue",
        ])
        .output()
        .unwrap();
    assert!(ecosystem.status.success(), "{}", output_details(&ecosystem));
    assert!(
        !String::from_utf8_lossy(&ecosystem.stdout).contains("ecosystem/nuxt-prefer-nuxt-link"),
        "{}",
        output_details(&ecosystem)
    );

    write_project_file(
        &project_root,
        "nuxt.config.ts",
        "export default defineNuxtConfig({})\n",
    );
    let nuxt = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "nuxt",
            "--no-config",
            "--format",
            "json",
            "app/pages/index.vue",
        ])
        .output()
        .unwrap();
    assert!(nuxt.status.success(), "{}", output_details(&nuxt));
    assert!(
        String::from_utf8_lossy(&nuxt.stdout).contains("ecosystem/nuxt-prefer-nuxt-link"),
        "{}",
        output_details(&nuxt)
    );

    let _ = fs::remove_dir_all(project_root);
}
