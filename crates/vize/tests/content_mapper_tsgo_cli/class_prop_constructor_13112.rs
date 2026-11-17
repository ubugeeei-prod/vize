//! #13112: a runtime prop typed by a user class is that class in the template,
//! under the content mapper and `vize check` alike.

use std::{fs, path::PathBuf, process::Command};

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

const CONFIG: &str = r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "types": []
  },
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
  "include": ["src/**/*.vue", "src/**/*.ts"]
}
"#;

const FILES: [(&str, &str); 5] = [
    (
        "src/Counter.ts",
        r#"import type { PropType } from "vue";

export class Counter {
  count = 0;
}

export const CounterProp = Object as PropType<Counter>;
export const shapes = { Counter };
"#,
    ),
    // The reported component, byte for byte.
    (
        "src/ClassProps.vue",
        r#"<script setup lang="ts">
import type { PropType } from "vue";
import { Counter } from "./Counter";

const props = defineProps({
  counter: { type: Counter },
  required: { type: Counter, required: true },
  date: { type: Date },
  viaPropType: { type: Object as PropType<Counter> },
});

const inScript: number | undefined = props.counter?.count;
</script>

<template>
  <p v-if="counter">{{ counter.count }}</p>
  <p>{{ required.count }}</p>
  <p v-if="date">{{ date.getTime() }}</p>
  <p v-if="viaPropType">{{ viaPropType.count }}</p>
  <p>{{ inScript }}</p>
</template>
"#,
    ),
    (
        "src/Variants.vue",
        r#"<script lang="ts">
class Local {
  label = "";
}
</script>

<script setup lang="ts">
import { Counter, CounterProp, shapes } from "./Counter";

defineProps({
  local: { type: Local, required: true },
  member: { type: shapes.Counter, required: true },
  viaConst: { type: CounterProp, required: true },
  either: { type: [Counter, String], required: true },
  defaulted: { type: Counter, default: () => new Counter() },
});
</script>

<template>
  <p>{{ local.label }} {{ member.count }} {{ viaConst.count }}</p>
  <p>{{ typeof either === "string" ? either.length : either.count }}</p>
  <p>{{ defaulted.count }}</p>
</template>
"#,
    ),
    (
        "src/OptionsApi.vue",
        r#"<script lang="ts">
import { defineComponent } from "vue";
import { Counter } from "./Counter";

export default defineComponent({
  props: { counter: { type: Counter, required: true } },
});
</script>

<template>
  <p>{{ counter.count }}</p>
</template>
"#,
    ),
    // The class type stays exact: unknown members and the optional
    // prop's `undefined` are still reported.
    (
        "src/Negative.vue",
        r#"<script setup lang="ts">
import { Counter } from "./Counter";

defineProps({
  counter: { type: Counter },
  required: { type: Counter, required: true },
});
</script>

<template>
  <p v-if="counter">{{ counter.missing }}</p>
  <p>{{ required.missing }}</p>
  <p>{{ counter.count }}</p>
</template>
"#,
    ),
];

const EXPECTED: [(u32, u32, u32, &str); 3] = [
    (
        11,
        32,
        2339,
        "Property 'missing' does not exist on type 'Counter'.",
    ),
    (
        12,
        18,
        2339,
        "Property 'missing' does not exist on type 'Counter'.",
    ),
    (13, 9, 18048, "'counter' is possibly 'undefined'."),
];

#[test]
fn class_constructor_props_keep_the_instance_type_in_the_template() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("class-prop-constructor-13112-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    for (name, source) in FILES {
        fs::write(project.path().join(name), source).unwrap();
    }

    let mapper = check_project(&tsgo, project.path(), "tsconfig.json");
    let mut expected = CompactString::default();
    for (line, column, code, message) in EXPECTED {
        expected.push_str(&cstr!(
            "src/Negative.vue({line},{column}): error TS{code}: {message}\n"
        ));
    }
    assert_eq!(
        std::str::from_utf8(&mapper.stdout).unwrap(),
        expected,
        "{}",
        output_text(&mapper)
    );
    assert_eq!(mapper.status.code(), Some(2), "{}", output_text(&mapper));
    assert_eq!(mapper.stderr, b"", "{}", output_text(&mapper));

    let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("CORSA_PATH", &tsgo)
        .args([
            "check",
            "--quiet",
            "--format",
            "json",
            "--tsconfig",
            "tsconfig.json",
            "src",
        ])
        .output()
        .unwrap();
    assert_eq!(cli.status.code(), Some(1), "{}", output_text(&cli));
    let report: serde_json::Value = serde_json::from_slice(&cli.stdout).unwrap();
    let mut observed = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| {
                    let text = diagnostic.as_str().unwrap();
                    let head = text.split(" (source:").next().unwrap_or(text);
                    cstr!("{name} {head}")
                })
        })
        .collect::<Vec<_>>();
    observed.sort();
    let expected = EXPECTED
        .iter()
        .map(|(line, column, code, message)| {
            cstr!("src/Negative.vue error:{line}:{column} [TS{code}] {message}")
        })
        .collect::<Vec<_>>();
    assert_eq!(observed, expected, "{}", output_text(&cli));
}
