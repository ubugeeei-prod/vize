use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::cstr;

#[test]
fn check_ignores_sfc_blocks_inside_script_string_literals() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("sfc-block-string-literal");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_test_vue_stub(&project_root.join("node_modules")).unwrap();
    for component in [
        "Accordion",
        "AccordionContent",
        "AccordionHeader",
        "AccordionPanel",
    ] {
        std::fs::write(
            project_root
                .join("src")
                .join(cstr!("{component}.vue").as_str()),
            r#"<template><div /></template>
"#,
        )
        .unwrap();
    }
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
        project_root.join("src/BasicDoc.vue"),
        r#"<script setup lang="ts">
import { ref } from 'vue';
import Accordion from './Accordion.vue';
import AccordionContent from './AccordionContent.vue';
import AccordionHeader from './AccordionHeader.vue';
import AccordionPanel from './AccordionPanel.vue';

const code = ref(`
<template>
  <Accordion value="0" />
</template>

<script setup lang="ts">
import Accordion from './Accordion.vue';
import AccordionPanel from './AccordionPanel.vue';
import AccordionHeader from './AccordionHeader.vue';
import AccordionContent from './AccordionContent.vue';
<\/script>
`);
</script>

<template>
  <Accordion>
    <AccordionPanel>
      <AccordionHeader />
      <AccordionContent />
    </AccordionPanel>
  </Accordion>
</template>
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "src/BasicDoc.vue",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS2300") && !stdout.contains("Duplicate identifier"),
        "string-literal SFC sample should not add duplicate imports:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

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
        .join(cstr!("check-{name}-{}", std::process::id()).as_str())
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

fn write_test_vue_stub(target: &Path) -> std::io::Result<()> {
    let vue_dir = target.join("vue");
    std::fs::create_dir_all(&vue_dir)?;
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{
  "name": "vue",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export interface Ref<T = unknown> {
  value: T;
}

export interface ShallowRef<T = unknown> {
  value: T;
}

export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}

export declare function ref<T>(value: T): Ref<T>;
"#,
    )?;
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
