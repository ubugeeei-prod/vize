//! #10617: a Vue 3 template reads an undeclared `$` name off its public
//! instance, so it reports where `vue-tsc` does. An Options API template reads
//! the authored instance, where mixin and options members named with `$` exist.
//!
//! Oracle: `vue-tsc` 3.3.11 with the pinned `vue` over the same source reports
//! only `Opts.vue(23,58): TS2339 Property '$undeclared' does not exist`.

use super::super::{BatchTypeChecker, relative_path, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ESNext",
    "module": "preserve",
    "moduleResolution": "bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "types": []
  },
  "include": ["src/**/*.vue"]
}
"#;

const OPTIONS_API: &str = r#"<script lang="ts">
import { defineComponent } from "vue";

const counter = defineComponent({
  computed: {
    $fromMixin(): number {
      return 1;
    },
  },
});

export default defineComponent({
  mixins: [counter],
  computed: {
    $mine(): number {
      return 1;
    },
  },
});
</script>

<template>
  <p>{{ $mine.toFixed() }} {{ $fromMixin.toFixed() }} {{ $undeclared.anything }}</p>
</template>
"#;

#[test]
fn options_api_reads_dollar_names_off_the_authored_instance() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = super::real_vue::create_project_case(
        "undeclared-instance-globals-options-api",
        &[("tsconfig.json", TSCONFIG), ("src/Opts.vue", OPTIONS_API)],
    );
    let mut checker = BatchTypeChecker::new(&root).expect("batch checker should initialize");
    checker.enable_options_api();
    checker.scan_project().expect("project scan should succeed");
    let result = checker.check_project().expect("project check should run");
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&root, &diagnostic.file),
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.code,
            )
        })
        .collect();
    let _ = std::fs::remove_dir_all(&root);

    assert_eq!(
        diagnostics,
        vec![("src/Opts.vue".into(), 23, 58, Some(2339))],
        "{:#?}",
        result.diagnostics
    );
}
