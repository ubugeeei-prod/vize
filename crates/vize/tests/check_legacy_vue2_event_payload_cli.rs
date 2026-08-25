#![cfg(feature = "legacy")]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[test]
fn legacy_vue2_component_event_payloads_stay_variadic_when_unresolved() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("legacy-vue2-event-payload-arity");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "src/App.vue",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    for unexpected in ["TS2345", "Target signature provides too few arguments"] {
        assert!(
            !stdout.contains(unexpected),
            "Vue 2 custom event handler arity should stay loose:\n{stdout}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src/components")).unwrap();
    write_test_vue2_stub(&project_root.join("node_modules")).unwrap();
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
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{ "typeChecker": { "legacyVue2": true } }"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/components/CalendarRange.vue"),
        r#"<script lang="ts">
import { defineComponent } from "vue"

export default defineComponent({
  emits: {
    "range-change": null,
    submit() {
      return true
    },
  },
})
</script>

<template>
  <button type="button" />
</template>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script lang="ts">
import { defineComponent } from "vue"
import CalendarRange from "./components/CalendarRange.vue"

export default defineComponent({
  components: { CalendarRange },
  setup() {
    function updateRange(
      calendarDate: { start: string; end: string },
      highlightSelectedDate: () => void,
    ) {
      void calendarDate
      highlightSelectedDate()
    }
    function submitHooks(hooks: { before?: () => void; after?: () => void }) {
      hooks.before?.()
    }
    return { updateRange, submitHooks }
  },
})
</script>

<template>
  <CalendarRange
    @range-change="updateRange"
    @submit="submitHooks"
  />
</template>
"#,
    )
    .unwrap();
    project_root
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return Some(PathBuf::from(path));
    }

    let workspace_root = workspace_root();
    [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn write_test_vue2_stub(target: &Path) -> std::io::Result<()> {
    let vue_types_dir = target.join("vue").join("types");
    std::fs::create_dir_all(&vue_types_dir)?;
    std::fs::write(
        target.join("vue").join("package.json"),
        r#"{ "name": "vue", "types": "types/index.d.ts" }"#,
    )?;
    std::fs::write(
        vue_types_dir.join("index.d.ts"),
        r#"export interface Vue {
  $attrs: Record<string, unknown>;
  $refs: Record<string, any>;
  $slots: Record<string, unknown>;
  $emit: (...args: any[]) => void;
}
export declare function defineComponent<T>(options: T): T;
export default { version: '2.7.16' };
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{ "name": "vite", "types": "client.d.ts" }"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}
