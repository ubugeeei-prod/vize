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
