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
