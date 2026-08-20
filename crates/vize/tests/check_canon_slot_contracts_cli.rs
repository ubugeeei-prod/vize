#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

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
        .join(format!(
            "check-canon-slot-contracts-{name}-{}",
            std::process::id()
        ))
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

fn write_tsconfig(project_root: &Path) {
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
}

fn create_case_with_files(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    for (path, source) in files {
        let target = project_root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, source).unwrap();
    }
    project_root
}

struct CheckReport {
    status: ExitStatus,
    json: serde_json::Value,
}

fn run_check_json(project_root: &Path, corsa_path: &Path, target: &str) -> CheckReport {
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
    let json = serde_json::from_str(stdout).unwrap_or_else(|_| {
        panic!(
            "invalid JSON report\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout,
            std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
        )
    });
    CheckReport {
        status: output.status,
        json,
    }
}

fn assert_clean(project_root: &Path, corsa_path: &Path, target: &str) {
    let report = run_check_json(project_root, corsa_path, target);
    assert!(report.status.success(), "{:?}", report.json);
    assert_eq!(
        report.json["errorCount"],
        serde_json::json!(0),
        "{:?}",
        report.json
    );
}

fn assert_diagnostic(
    report: &serde_json::Value,
    file_suffix: &str,
    line: u32,
    column: u32,
    code: u32,
    expected: &str,
) {
    let files = report["files"]
        .as_array()
        .expect("JSON report should include files");
    let file = files
        .iter()
        .find(|file| {
            file["file"]
                .as_str()
                .is_some_and(|file| file.ends_with(file_suffix))
        })
        .unwrap_or_else(|| panic!("missing diagnostics for {file_suffix}: {report:#}"));
    let diagnostics: Vec<_> = file["diagnostics"]
        .as_array()
        .expect("JSON file entry should include diagnostics")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    let prefix = format!("error:{line}:{column} [TS{code}]");
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.starts_with(&prefix))
        .unwrap_or_else(|| {
            panic!("missing diagnostic {prefix:?} in {file_suffix}: {diagnostics:#?}")
        });

    // Exact oracle (assurance §4): the whole diagnostic line, not fragments.
    assert_eq!(
        *diagnostic, expected,
        "diagnostic for {file_suffix} diverged from the pinned line"
    );
}

#[test]
fn check_child_slot_payloads_and_omitted_slots_are_supported() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case_with_files(
        "child-slot-contracts",
        &[
            (
                "src/env.d.ts",
                r#"declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<object, object, unknown>;
  export default component;
}
"#,
            ),
            (
                "src/DefaultSlotChild.vue",
                r#"<script setup lang="ts">
defineSlots<{
  default(props: { msg: string }): unknown;
}>();
</script>

<template>
  <slot :msg="'hello'" />
</template>
"#,
            ),
            (
                "src/HeaderSlotChild.vue",
                r#"<script setup lang="ts">
defineSlots<{
  header(props: { title: string }): unknown;
}>();
</script>

<template>
  <slot name="header" :title="'Hi'" />
</template>
"#,
            ),
            (
                "src/LibraryButton.ts",
                r#"declare const button: { new (): { $props: {}; $slots: { default(props: { label: string }): unknown; icon(): unknown; }; }; }; export default button;"#,
            ),
            (
                "src/Clean.vue",
                r#"<script setup lang="ts">
import DefaultSlotChild from "./DefaultSlotChild.vue";
import HeaderSlotChild from "./HeaderSlotChild.vue";
import LibraryButton from "./LibraryButton";
</script>

<template>
  <DefaultSlotChild>
    <template #default="{ msg }">{{ msg }}</template>
  </DefaultSlotChild>
  <HeaderSlotChild>
    <template #header="{ title }">{{ title }}</template>
  </HeaderSlotChild>
  <HeaderSlotChild />
  <LibraryButton />
</template>
"#,
            ),
            (
                "src/UnknownSlotProp.vue",
                r#"<script setup lang="ts">
import DefaultSlotChild from "./DefaultSlotChild.vue";
</script>

<template>
  <DefaultSlotChild>
    <template #default="{ missing }">{{ missing }}</template>
  </DefaultSlotChild>
</template>
"#,
            ),
        ],
    );

    assert_clean(&project_root, &corsa_path, "src/Clean.vue");

    let unknown_slot = run_check_json(&project_root, &corsa_path, "src/UnknownSlotProp.vue");
    assert!(!unknown_slot.status.success(), "{:?}", unknown_slot.json);
    assert_eq!(
        unknown_slot.json["errorCount"],
        serde_json::json!(1),
        "{:?}",
        unknown_slot.json
    );
    assert_diagnostic(
        &unknown_slot.json,
        "src/UnknownSlotProp.vue",
        7,
        27,
        2339,
        "error:7:27 [TS2339] Property 'missing' does not exist on type '{ msg: string; }'.",
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
