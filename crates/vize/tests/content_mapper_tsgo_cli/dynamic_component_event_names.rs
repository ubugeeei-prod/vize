//! #11140: computed component event names are expressions, not type keys.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

const CHILD: &str = r#"<script setup lang="ts">
defineEmits<{ pointerdown: []; pointerup: []; ping: [] }>();
</script>

<template><div /></template>
"#;

const PROBE: &str = r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";

const Events = { POINTER_DOWN: "pointerdown", POINTER_UP: "pointerup" } as const;
const keys = { ping: "ping" } as const;
const getKey = (name: string) => name;
const pressed = ref(false);
</script>

<template>
  <Child
    @[`${Events.POINTER_DOWN}`]="pressed = true"
    @["ping"]="pressed = true"
    @[getKey(Events.POINTER_UP)]="pressed = true"
    @[keys.ping]="pressed = true"
  />
</template>
"#;

#[test]
fn standard_tsgo_accepts_computed_component_event_names() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-dynamic-component-events-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("src/Child.vue"), CHILD).unwrap();
    std::fs::write(project.path().join("src/Probe.vue"), PROBE).unwrap();
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true,
                "noEmit": true,
                "skipLibCheck": true,
                "target": "ESNext",
                "module": "preserve",
                "types": []
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();

    assert_success(&check_project(&tsgo, project.path(), "tsconfig.json"));

    let broken = PROBE.replacen("pressed = true", "pressed = 1", 1);
    std::fs::write(project.path().join("src/Probe.vue"), broken).unwrap();
    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    let diagnostics = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect::<Vec<_>>();
    assert_eq!(
        diagnostics,
        ["src/Probe.vue(13,34): error TS2322: Type 'number' is not assignable to type 'boolean'."]
    );
}
