#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("check-canon-define-model-{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn link_workspace_vue(project_root: &Path) -> std::io::Result<()> {
    let Some(vue_package) = workspace_vue_package() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package missing",
        ));
    };
    let workspace_node_modules = vue_package.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package has no node_modules parent",
        )
    })?;
    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    symlink_path(&vue_package, &target.join("vue"))?;
    let vue_namespace = workspace_node_modules.join("@vue");
    if vue_namespace.exists() {
        symlink_path(&vue_namespace, &target.join("@vue"))?;
    }
    Ok(())
}

fn workspace_vue_package() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
        root.join("examples/vite-musea/node_modules/vue"),
        root.join("examples/jsx-tsx/node_modules/vue"),
        root.join("npm/framework/nuxt/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}

fn create_case_with_files(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
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
}"#,
    )
    .unwrap();
    for (path, source) in files {
        let target = project_root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, source).unwrap();
    }
    project_root
}

fn run_check_json(
    project_root: &Path,
    corsa_path: &Path,
    target: &str,
) -> Result<serde_json::Value, serde_json::Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            target,
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let json = serde_json::from_str(stdout).unwrap_or_else(|error| {
        panic!(
            "invalid JSON report: {error}\nstatus: {:?}\nstdout:\n{stdout}\nstderr:\n{}",
            output.status.code(),
            std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
        )
    });
    if output.status.success() {
        Ok(json)
    } else {
        Err(json)
    }
}

fn diagnostics_for<'a>(report: &'a serde_json::Value, file_name: &str) -> Vec<&'a str> {
    report["files"]
        .as_array()
        .unwrap_or_else(|| panic!("files should be an array: {report}"))
        .iter()
        .find(|file| file["file"] == serde_json::json!(file_name))
        .unwrap_or_else(|| panic!("missing diagnostics for {file_name}: {report}"))["diagnostics"]
        .as_array()
        .unwrap_or_else(|| panic!("diagnostics should be an array: {report}"))
        .iter()
        .map(|diagnostic| {
            diagnostic
                .as_str()
                .unwrap_or_else(|| panic!("diagnostic should be a string: {diagnostic}"))
        })
        .collect()
}

#[test]
fn check_define_model_modifier_tuple_types_are_preserved() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "define-model-modifier-tuples",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
const [model, modifiers] = defineModel<string, "trim" | "capitalize">({ required: true });

if (modifiers.trim) {
  model.value = model.value.trim();
}
</script>

<template>
  <span>{{ model }}</span>
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

let text = " hello ";
</script>

<template>
  <Child v-model.trim="text" />
</template>
"#,
            ),
            (
                "src/Setter.vue",
                r#"<script setup lang="ts">
const [model] = defineModel<number, "trim", string, boolean>();

const read: string | undefined = model.value;
model.value = true;
</script>
"#,
            ),
            (
                "src/Broken.vue",
                r#"<script setup lang="ts">
const [model, modifiers] = defineModel<string, "trim" | "capitalize">({ required: true });

if (modifiers.nope) {
  model.value = model.value.trim();
}
</script>
"#,
            ),
            (
                "src/BrokenParent.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

let text = " hello ";
</script>

<template>
  <Child v-model.nope="text" />
</template>
"#,
            ),
        ],
    );

    assert!(run_check_json(&project_root, &corsa_path, "src/Child.vue").is_ok());
    assert!(run_check_json(&project_root, &corsa_path, "src/App.vue").is_ok());
    assert!(run_check_json(&project_root, &corsa_path, "src/Setter.vue").is_ok());

    let broken = run_check_json(&project_root, &corsa_path, "src/Broken.vue").unwrap_err();
    assert_eq!(broken["errorCount"], serde_json::json!(1), "{broken}");
    // Exact oracle: the modifiers object is typed from the authored tuple, so
    // the unknown `nope` modifier must produce exactly this TS2339 diagnostic.
    assert_eq!(
        diagnostics_for(&broken, "src/Broken.vue"),
        [
            "error:4:15 [TS2339] Property 'nope' does not exist on type '__VizeModelModifiers<\"capitalize\" | \"trim\">'."
        ],
        "{broken}"
    );

    let broken_parent =
        run_check_json(&project_root, &corsa_path, "src/BrokenParent.vue").unwrap_err();
    assert_eq!(
        broken_parent["errorCount"],
        serde_json::json!(1),
        "{broken_parent}"
    );
    // Exact oracle: the parent's modifier map is closed over the authored
    // tuple, so `v-model.nope` must produce exactly this TS2353 diagnostic.
    assert_eq!(
        diagnostics_for(&broken_parent, "src/BrokenParent.vue"),
        [
            "error:8:29 [TS2353] Object literal may only specify known properties, and '\"nope\"' does not exist in type 'Partial<Record<\"capitalize\" | \"trim\", true>>'."
        ],
        "{broken_parent}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
