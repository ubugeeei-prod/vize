//! Out-of-root workspace `.vue` imports keep their types (#3887).
//!
//! `vize check src` from a workspace app previously left every component
//! imported from a sibling package on the ambient `*.vue` stub: props and
//! emit payloads silently became `any`, and the only trace was a knock-on
//! `TS7006` on handler parameters. The reachability pass registers those
//! files, so the single-root invocation reports what the two-root control
//! run always did.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    let workspace_bin = workspace_root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.display().to_string())
}

#[test]
fn a_workspace_package_component_is_type_checked_through_its_barrel() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let monorepo = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("workspace-vue-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&monorepo);

    write_file(
        &monorepo,
        "packages/ui/src/UiButton.vue",
        r#"<script setup lang="ts">
defineProps<{ variant: "primary" | "ghost" }>();
const emit = defineEmits<{ press: [variant: "primary" | "ghost"] }>();
</script>
<template>
  <button @click="emit('press', 'primary')"><slot /></button>
</template>
"#,
    );
    write_file(
        &monorepo,
        "packages/ui/src/index.ts",
        "export { default as UiButton } from \"./UiButton.vue\";\n",
    );
    write_file(
        &monorepo,
        "apps/web/tsconfig.json",
        r##"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "paths": {
      "#ui": ["../../packages/ui/src/index.ts"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"##,
    );
    write_file(
        &monorepo,
        "apps/web/src/Panel.vue",
        r##"<script setup lang="ts">
import { UiButton } from "#ui";
</script>
<template>
  <UiButton variant="danger" @press="(v) => v.toFixed(2)" />
</template>
"##,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(monorepo.join("apps/web"))
        .env("CORSA_PATH", corsa_path)
        .args(["check", "src", "--no-config", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        stdout.contains("TS2322") && stdout.contains("danger"),
        "the workspace component's prop union must be enforced:\n{stdout}\n{stderr}"
    );
    assert!(
        stdout.contains("TS2551") && stdout.contains("toFixed"),
        "the emit payload must type the handler parameter:\n{stdout}\n{stderr}"
    );
    assert!(
        !stdout.contains("TS7006"),
        "the implicit-any knock-on must be gone:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&monorepo);
}
