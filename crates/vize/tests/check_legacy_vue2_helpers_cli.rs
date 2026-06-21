#![cfg(feature = "legacy")]

use std::{path::Path, process::Command};

use vize_carton::cstr;

#[test]
fn check_legacy_vue2_show_virtual_ts_omits_shared_helpers() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "src/App.vue", "--show-virtual-ts"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains(vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME),
        "legacy Vue 2 should not show a shared helpers preamble:\n{stderr}"
    );
    assert!(
        stderr.contains("declare function __vizeDefineComponent<T>(options: T): T;"),
        "expected per-file legacy defineComponent helper:\n{stderr}"
    );
    for vue3_only_helper in [
        "import('vue').Ref",
        "import('vue').ShallowRef",
        "import('vue').ComponentPublicInstance",
        "import('vue').defineComponent",
    ] {
        assert!(
            !stderr.contains(vue3_only_helper),
            "legacy Vue 2 virtual TS must not contain {vue3_only_helper}:\n{stderr}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project() -> std::path::PathBuf {
    let project_root = unique_case_dir("legacy-vue2-show-virtual-ts-no-shared-helpers");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_test_vue2_6_stub(&project_root.join("node_modules")).unwrap();
    write_test_vite_stub(&project_root.join("node_modules")).unwrap();
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
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{
  "typeChecker": {
    "legacyVue2": true
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
}
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
    )
    .unwrap();
    project_root
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<String> {
    if let Ok(path) = std::env::var("CORSA_PATH") {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }

    let workspace_root = workspace_root();
    [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
    .map(|candidate| candidate.to_string_lossy().into_owned())
}

fn write_test_vue2_6_stub(target: &Path) -> std::io::Result<()> {
    let vue_types_dir = target.join("vue").join("types");
    std::fs::create_dir_all(&vue_types_dir)?;
    std::fs::write(
        target.join("vue").join("package.json"),
        r#"{
  "name": "vue",
  "types": "types/index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_types_dir.join("index.d.ts"),
        r#"export interface Vue {
  $attrs: Record<string, unknown>;
  $refs: Record<string, any>;
  $slots: Record<string, unknown>;
  $emit: (...args: any[]) => void;
}

export type PropType<T> = { new (...args: any[]): T & {} } | { (): T } | null;

declare const VueConstructor: {
  version: string;
};

export default VueConstructor;
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{
  "name": "vite",
  "types": "client.d.ts"
}"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}
