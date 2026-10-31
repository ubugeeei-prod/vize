//! Raw external-mapper regression reported by @Q16solver in #11023.
//!
//! The minimized files are copied byte-for-byte from the report. The report
//! traces the production case to `REPORTER_SOURCE_REVISION` and the regression
//! to `VIZE_REGRESSION_REVISION` (`v0.602.0`).

use std::path::PathBuf;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const REPORT_URL: &str =
    "https://github.com/ubugeeei-prod/vize/issues/11023#issuecomment-6458861166";
const REPORTER_SOURCE_REVISION: &str = "407e7cf15219405110efdd0c977bb30cb1db1644";
const VIZE_REGRESSION_REVISION: &str = "92223735e9c41cefbfe584d7f364e4847569c92b";

const CHILD: &str = r#"<script setup lang="ts">
interface Props {
  bar: (baz: "a" | "b") => boolean;
}

defineProps<Props>();
defineEmits<{
  quit: [];
}>();
</script>

<template>
  <div />
</template>
"#;

const PARENT: &str = r#"<script setup lang="ts">
import type { VNode } from "vue";
import Foo from "./Foo.vue";

const emit = defineEmits<{
  quit: [];
}>();
const slots = defineSlots<Partial<Record<"a" | "b", () => VNode>>>();
</script>

<template>
  <Foo :bar="(baz) => Boolean(slots[baz])" @quit="emit('quit')" />
</template>
"#;

const CONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ESNext", "DOM"],
    "types": [],
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "allowArbitraryExtensions": true
  },
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
  "include": ["*.ts", "*.vue"]
}
"#;

const NESTED_ARROW_CHILD: &str = r#"<script setup lang="ts">
interface Props {
  apply: (
    fn: (value: "a" | "b") => boolean,
    value: "a" | "b",
  ) => boolean;
}

defineProps<Props>();
defineEmits<{
  quit: [];
}>();
</script>

<template>
  <div />
</template>
"#;

const NESTED_ARROW_PARENT: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";

const emit = defineEmits<{
  quit: [];
}>();
</script>

<template>
  <Foo
    :apply="(fn: (value: 'a' | 'b') => boolean,
      value) => fn(value)"
    @quit="emit('quit')"
  />
</template>
"#;

const GENERIC_BRANCH_CHILD: &str = r#"<script setup lang="ts">
defineProps<{
  apply: <T extends () => unknown>(value: T) => T;
}>();
defineEmits<{ quit: [] }>();
</script>

<template><div /></template>
"#;

const GENERIC_BRANCH_PARENT: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";

const condition = true;
const emit = defineEmits<{ quit: [] }>();
</script>

<template>
  <Foo
    :apply="(void 0, <T extends () => unknown>(
      value) => value)"
    @quit="emit('quit')"
  />
  <Foo
    :apply="(
      <T extends () => unknown>(
        value) => value
    )"
    @quit="emit('quit')"
  />
  <Foo
    :apply="condition
      ? <T extends () => unknown>(
          value) => {
          return value
        }
      : <U extends () => unknown>(
          value) => value"
    @quit="emit('quit')"
  />
</template>
"#;

#[test]
fn template_callback_next_to_listener_keeps_its_contextual_type() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!(
            "skipping exact Content Mapper callback regression from {REPORT_URL}: {TSGO_ENV} is not set"
        );
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-template-callback-listener-11023-")
        .tempdir_in(cases_root)
        .unwrap();
    std::fs::write(project.path().join("Foo.vue"), CHILD).unwrap();
    std::fs::write(project.path().join("parent.vue"), PARENT).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "report: {REPORT_URL}\nreporter source: {REPORTER_SOURCE_REVISION}\nvize source: {VIZE_REGRESSION_REVISION}\n{}",
        output_text(&output)
    );
    assert_eq!(output.stdout, b"", "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
}

#[test]
fn multiline_callback_skips_nested_function_type_arrow_for_suppression() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping nested callback-arrow conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-nested-callback-arrow-")
        .tempdir_in(cases_root)
        .unwrap();
    std::fs::write(project.path().join("Foo.vue"), NESTED_ARROW_CHILD).unwrap();
    std::fs::write(project.path().join("parent.vue"), NESTED_ARROW_PARENT).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(
        output.status.code(),
        Some(0),
        "the nested function-type arrow must not expose generated TS7006\n{}",
        output_text(&output)
    );
    assert_eq!(output.stdout, b"", "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
}

#[test]
fn multiline_generic_callback_branches_preserve_authored_parameter_errors_through_repairs() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping generic callback branch conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-generic-callback-branches-")
        .tempdir_in(cases_root)
        .unwrap();
    std::fs::write(project.path().join("Foo.vue"), GENERIC_BRANCH_CHILD).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    // These explicit generic arrows leave their authored parameters untyped.
    // TypeScript and vue-tsc report the same four TS7006 errors: inference-only
    // copies must add no errors, while the mapped authored owners retain them.
    let expected = concat!(
        "parent.vue(11,7): error TS7006: Parameter 'value' implicitly has an 'any' type.\n",
        "parent.vue(17,9): error TS7006: Parameter 'value' implicitly has an 'any' type.\n",
        "parent.vue(24,11): error TS7006: Parameter 'value' implicitly has an 'any' type.\n",
        "parent.vue(28,11): error TS7006: Parameter 'value' implicitly has an 'any' type.\n",
    );
    let mut repaired = GENERIC_BRANCH_PARENT.to_owned();
    for (before, after) in [
        ("\n      value) => value)", "\n      value: T) => value)"),
        ("\n        value) => value", "\n        value: T) => value"),
        ("\n          value) => {", "\n          value: T) => {"),
        (
            "\n          value) => value",
            "\n          value: U) => value",
        ),
    ] {
        assert_eq!(repaired.matches(before).count(), 1);
        repaired = repaired.replace(before, after);
    }

    for (state, source, expected) in [
        ("original-invalid", GENERIC_BRANCH_PARENT, expected),
        ("typed-repair", repaired.as_str(), ""),
        ("invalid-again", GENERIC_BRANCH_PARENT, expected),
        ("repaired-again", repaired.as_str(), ""),
    ] {
        std::fs::write(project.path().join("parent.vue"), source).unwrap();
        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        assert_eq!(
            output.status.code(),
            Some(if expected.is_empty() { 0 } else { 2 }),
            "{state}: sequence, wrapper and conditional callback branches retain only authored diagnostics\n{}",
            output_text(&output)
        );
        assert_eq!(
            output.stdout,
            expected.as_bytes(),
            "{state}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{state}: {}", output_text(&output));
    }
}
