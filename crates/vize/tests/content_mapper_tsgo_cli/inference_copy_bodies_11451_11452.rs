//! Q16solver #11451 and #11452: the inference-only callback copy reports
//! nothing, and each authored error inside the callback is reported once.

use std::path::PathBuf;

use vize_l0::{String, cstr};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const PACKAGE: &str = r#"{
  "private": true,
  "type": "module",
  "devDependencies": {
    "tsrs": "0.20.1-ts7.1.0-beta",
    "typescript": "6.0.3",
    "vize": "0.650.0",
    "vue": "3.5.44",
    "vue-tsc": "3.3.11"
  }
}
"#;

const CONFIG: &str = r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext"
  },
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
  "include": ["src/**/*.vue"]
}
"#;

const DESTRUCTURED_11451: &str = r#"<script setup lang="ts">
interface Foo {
  bar: (callback: (baz: { qux: number }) => void) => void;
}

function Child(props: { onQux?: (foo: Foo) => void }) {
  return null;
}
</script>

<template>
  <Child
    :on-qux="
      (foo) => {
        foo.bar(({ qux }) => {
          console.log(qux);
        });
      }
    "
  />
</template>
"#;

const ASSIGNED_11452: &str = r#"<script setup lang="ts">
import { ref } from "vue";

interface Foo {
  width: number;
}

const foo = ref<Foo | undefined>();

function Child(props: { onBar?: (baz: Foo) => void }) {
  return null;
}
</script>

<template>
  <Child
    :on-bar="
      (baz) => {
        foo = baz;
        console.log(foo.width);
      }
    "
  />
</template>
"#;

const DESTRUCTURED_CHILD: &str = r#"<script setup lang="ts">
interface Foo {
  bar: (callback: (baz: { qux: number }) => void) => void;
}
defineProps<{ onQux?: (foo: Foo) => void }>();
</script>

<template><span /></template>
"#;

const ASSIGNED_CHILD: &str = r#"<script setup lang="ts">
interface Foo {
  width: number;
}
defineProps<{ onBar?: (baz: Foo) => void }>();
</script>

<template><span /></template>
"#;

fn edit(source: &str, from: &str, to: &str) -> String {
    assert_eq!(source.matches(from).count(), 1, "{from}");
    source.replace(from, to).into()
}

fn destructured_cases() -> Vec<(&'static str, String, Option<&'static str>, &'static str)> {
    let original = DESTRUCTURED_11451;
    let script = "interface Foo {\n  bar: (callback: (baz: { qux: number }) => void) => void;\n}\n\nfunction Child(props: { onQux?: (foo: Foo) => void }) {\n  return null;\n}\n";
    vec![
        ("original", String::from(original), None, ""),
        (
            "renamed",
            edit(
                &edit(original, "({ qux })", "({ qux: quux })"),
                "log(qux)",
                "log(quux)",
            ),
            None,
            "",
        ),
        (
            "array",
            edit(
                &edit(original, "(baz: { qux: number })", "(baz: [number])"),
                "({ qux })",
                "([qux])",
            ),
            None,
            "",
        ),
        (
            "rest",
            edit(
                &edit(original, "(baz: { qux: number })", "(...baz: number[])"),
                "({ qux })",
                "(...qux)",
            ),
            None,
            "",
        ),
        (
            "function",
            edit(original, "({ qux }) => {", "function ({ qux }) {"),
            None,
            "",
        ),
        (
            "expression-body",
            edit(
                original,
                "({ qux }) => {\n          console.log(qux);\n        });",
                "({ qux }) => console.log(qux));",
            ),
            None,
            "",
        ),
        (
            "vue-child",
            edit(original, script, "import Child from \"./Child.vue\";\n"),
            Some(DESTRUCTURED_CHILD),
            "",
        ),
        (
            "nested-member",
            edit(
                original,
                "console.log(qux);",
                "console.log(qux);\n          qux.nope();",
            ),
            None,
            "src/Probe.vue(17,15): error TS2339: Property 'nope' does not exist on type 'number'.\n",
        ),
        (
            "nested-assignment",
            edit(
                original,
                "console.log(qux);",
                "console.log(qux);\n          const text: string = qux;",
            ),
            None,
            "src/Probe.vue(17,17): error TS2322: Type 'number' is not assignable to type 'string'.\n",
        ),
        ("repair", String::from(original), None, ""),
    ]
}

fn assigned_cases() -> Vec<(&'static str, String, Option<&'static str>, &'static str)> {
    let original = ASSIGNED_11452;
    let script = "interface Foo {\n  width: number;\n}\n\nconst foo = ref<Foo | undefined>();\n\nfunction Child(props: { onBar?: (baz: Foo) => void }) {\n  return null;\n}\n";
    vec![
        ("original", String::from(original), None, ""),
        (
            "local-let",
            edit(
                &edit(original, "import { ref } from \"vue\";\n", ""),
                "const foo = ref<Foo | undefined>();",
                "let foo: Foo | undefined;",
            ),
            None,
            "",
        ),
        (
            "same-line",
            edit(
                original,
                "foo = baz;\n        console.log",
                "foo = baz; console.log",
            ),
            None,
            "",
        ),
        (
            "vue-child",
            edit(
                original,
                script,
                "import Child, { type Foo } from \"./Child.vue\";\n\nconst foo = ref<Foo | undefined>();\n",
            ),
            Some(ASSIGNED_CHILD),
            "",
        ),
        (
            "wrong-member",
            edit(
                original,
                "console.log(foo.width);",
                "console.log(foo.width);\n        console.log(foo.nope);",
            ),
            None,
            "src/Probe.vue(21,25): error TS2339: Property 'nope' does not exist on type 'Foo'.\n",
        ),
        (
            "unnarrowed-read",
            edit(original, "foo = baz;", "console.log(baz);"),
            None,
            "src/Probe.vue(20,21): error TS18048: 'foo' is possibly 'undefined'.\n",
        ),
        ("restored", String::from(original), None, ""),
    ]
}

#[test]
fn inference_copies_neither_contradict_nor_repeat_callback_bodies() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping #11451/#11452 Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} is not a file");
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    for (issue, cases) in [("11451", destructured_cases()), ("11452", assigned_cases())] {
        let project = tempfile::Builder::new()
            .prefix(cstr!("inference-copy-{issue}-").as_str())
            .tempdir_in(&root)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        std::fs::create_dir(project.path().join("src")).unwrap();
        std::fs::write(project.path().join("package.json"), PACKAGE).unwrap();
        std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
        for (phase, source, child, expected) in cases {
            std::fs::write(project.path().join("src/Probe.vue"), &source).unwrap();
            let child_path = project.path().join("src/Child.vue");
            if let Some(child) = child {
                std::fs::write(&child_path, child).unwrap();
            } else if child_path.exists() {
                std::fs::remove_file(&child_path).unwrap();
            }
            let output = check_project(&native, project.path(), "tsconfig.json");
            assert_eq!(
                std::str::from_utf8(&output.stdout).unwrap(),
                expected,
                "#{issue} {phase}: {}",
                output_text(&output)
            );
            assert_eq!(
                output.status.code(),
                Some(if expected.is_empty() { 0 } else { 2 }),
                "#{issue} {phase}: {}",
                output_text(&output)
            );
            assert_eq!(output.stderr, b"", "#{issue} {phase}");
        }
    }
}
