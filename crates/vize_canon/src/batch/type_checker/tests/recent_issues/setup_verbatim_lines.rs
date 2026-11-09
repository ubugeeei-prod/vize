//! Authored setup lines are copied verbatim: multi-line literals keep their
//! values, and setup errors keep their authored positions (#12481).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "jsx": "preserve",
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.vue"]
}
"#;

/// Continuation lines inside template and string literals are literal text.
const LITERALS: &str = r#"<script setup lang="ts">
const sql = `
SELECT *
FROM t`
const exact: "\nSELECT *\nFROM t" = sql
const marker = `
export default 1
`
const markerExact: "\nexport default 1\n" = marker
const continued = "a\
b"
const joined: "ab" = continued
const lines = `${[1].map((n) => {
  const doubled = n * 2
  return doubled
}).join('')}
  indented`
</script>
<template><div>{{ exact }}{{ markerExact }}{{ joined }}{{ lines }}</div></template>
"#;

/// A real error nested in authored blocks keeps its line and column.
const NESTED_ERROR: &str = r#"<script setup lang="ts">
function pick(flag: boolean) {
  if (flag) {
    const wrong: number = "s"
    return wrong
  }
  return 0
}
</script>
<template><div>{{ pick(true) }}</div></template>
"#;

#[test]
fn multi_line_setup_literals_keep_their_values_and_errors_keep_their_positions() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "setup-verbatim-lines-12481",
        &[
            ("src/Literals.vue", LITERALS),
            ("src/NestedError.vue", NESTED_ERROR),
            ("tsconfig.json", TSCONFIG),
        ],
    );
    let actual = snapshot_project_diagnostics(&project).expect("native diagnostics");
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        vec![(
            "src/NestedError.vue".into(),
            Some(2322),
            "4:11:error Type 'string' is not assignable to type 'number'.".into(),
        )],
    );
}
