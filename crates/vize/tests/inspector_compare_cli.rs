#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use std::{fs, process::Command};

#[cfg(unix)]
#[test]
fn inspector_compare_hides_missing_vue_compiler_stack_trace() {
    use std::os::unix::fs::PermissionsExt;

    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("App.vue"),
        "<template><div>legacy vue</div></template>\n",
    )
    .unwrap();

    let fake_node = project.path().join("fake-node");
    fs::write(
        &fake_node,
        "#!/bin/sh\n\
         echo \"node:internal/modules/esm/resolve:271\" >&2\n\
         echo \"Error [ERR_MODULE_NOT_FOUND]: Cannot find module '/app/node_modules/vue/compiler-sfc'\" >&2\n\
         exit 1\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_node).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_node, permissions).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("VIZE_INSPECTOR_NODE", &fake_node)
        .args(["inspector", "src/App.vue", "--format", "compare"])
        .output()
        .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("currently requires Vue 3"), "{stderr}");
    assert!(
        stderr.contains("Vue 2 / Nuxt 2 projects are not supported"),
        "{stderr}"
    );
    assert!(!stderr.contains("node:internal/modules"), "{stderr}");
    assert!(!stderr.contains("ERR_MODULE_NOT_FOUND"), "{stderr}");
}

#[cfg(unix)]
fn compare_with_closed_node_stdin(script: &str) -> std::process::Output {
    use std::os::unix::fs::PermissionsExt;

    let project = tempfile::tempdir().unwrap();
    let source = format!(
        "{}\n<!--{}-->\n",
        include_str!("fixtures/inspector_compare/early_exit.vue.txt"),
        "x".repeat(2 * 1024 * 1024)
    );
    fs::write(project.path().join("App.vue"), source).unwrap();
    let fake_node = project.path().join("fake-node");
    fs::write(&fake_node, format!("#!/bin/sh\nexec 0<&-\n{script}\n")).unwrap();
    fs::set_permissions(&fake_node, fs::Permissions::from_mode(0o755)).unwrap();
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("VIZE_INSPECTOR_NODE", fake_node)
        .args(["inspector", "App.vue", "--format", "compare"])
        .output()
        .unwrap()
}

#[cfg(unix)]
#[test]
fn inspector_compare_retains_missing_module_diagnostic_after_broken_pipe() {
    let output = compare_with_closed_node_stdin(
        "echo \"node:internal/modules/esm/resolve:271\" >&2\n\
         echo \"Error [ERR_MODULE_NOT_FOUND]: Cannot find module '/app/node_modules/vue/compiler-sfc'\" >&2\n\
         exit 17",
    );
    assert_eq!(
        output.stderr.as_slice(),
        include_bytes!("fixtures/inspector_compare/missing_compiler.stderr.txt")
    );
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(17), "{stderr}");
    assert!(stderr.contains("currently requires Vue 3"), "{stderr}");
    assert!(
        stderr.contains("Vue 2 / Nuxt 2 projects are not supported"),
        "{stderr}"
    );
    assert!(!stderr.contains("node:internal/modules"), "{stderr}");
    assert!(!stderr.contains("ERR_MODULE_NOT_FOUND"), "{stderr}");
    assert!(!stderr.contains("Failed to write"), "{stderr}");
}

#[cfg(unix)]
#[test]
fn inspector_compare_retains_child_failure_status_and_stderr_after_broken_pipe() {
    let output =
        compare_with_closed_node_stdin("echo 'authoritative compiler failure' >&2\nexit 23");
    assert_eq!(
        output.stderr.as_slice(),
        include_bytes!("fixtures/inspector_compare/compiler_failure.stderr.txt")
    );
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(23), "{stderr}");
    assert!(
        stderr.contains("authoritative compiler failure"),
        "{stderr}"
    );
    assert!(!stderr.contains("Failed to write"), "{stderr}");
}

#[cfg(unix)]
#[test]
fn inspector_compare_rejects_successful_child_that_did_not_receive_input() {
    let output = compare_with_closed_node_stdin("echo '{\"files\":[]}'\nexit 0");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("Failed to write inspector compare input"),
        "{stderr}"
    );
    assert!(!stderr.contains("Failed to parse"), "{stderr}");
    assert!(output.stdout.is_empty());
}
