#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

#[test]
fn check_nuxt2_compiler_compat_warning_does_not_suggest_nuxi_prepare() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("nuxt2-fallback-guidance");

    write_file(&project_root, "nuxt.config.ts", "export default {};\n");
    write_file(
        &project_root,
        "vize.config.json",
        r#"{
  "compiler": {
    "compatibility": {
      "vueVersion": "2",
      "hostCompiler": true
    }
  },
  "typeChecker": {
    "tsconfig": "tsconfig.json"
  }
}
"#,
    );
    write_file(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["components/**/*.vue"]
}
"#,
    );
    write_file(
        &project_root,
        "components/AppLink.vue",
        r#"<template>
  <NuxtLink to="/">{{ label }}</NuxtLink>
</template>

<script lang="ts">
export default {
  data() {
    return { label: "home" };
  },
};
</script>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--config", "vize.config.json", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        output.status.success(),
        "Nuxt 2 fallback check should still complete\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("no generated `.nuxt` types found"),
        "missing generated Nuxt types should still be surfaced:\n{stderr}"
    );
    assert!(
        stderr.contains("Nuxt 2/Bridge"),
        "Vue 2 compatibility config should select Nuxt 2 guidance:\n{stderr}"
    );
    assert!(
        !stderr.contains("nuxi prepare"),
        "Nuxt 2 guidance must not suggest Nuxt 3 prepare:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root);
    project_root
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    if source.exists() {
        symlink_path(&source, &project_root.join("node_modules")).unwrap();
    }
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    let workspace_root = workspace_root();
    [workspace_root.join("node_modules/.bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
