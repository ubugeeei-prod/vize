use crate::{LintPreset, Linter};

const SHAPES: &str = r#"<script setup lang="ts">
import { computed, ref } from "vue";
import type { Component } from "vue";

const usedInTemplate = 1;
const unused = 2;
const _ignored = 3;
const source = { first: 1, second: 2 };
const { first, second } = source;
let counter = 0;
let neverReassigned = 4;
const total = ref(first);
function keepFirst(a: number, b: number): number {
  return a;
}
function nested(): number {
  const inner = 5;
  return 6;
}
function handler(_event: Event, value: number): number {
  return value + total.value + keepFirst(1, 2) + nested();
}
</script>

<template>
  <button type="button" @click="counter++">{{ usedInTemplate }} {{ neverReassigned }} {{ handler }}</button>
</template>
"#;

#[test]
fn opinionated_reports_the_seven_script_findings_and_keeps_counter() {
    let result = Linter::with_preset(LintPreset::Opinionated).lint_sfc(SHAPES, "UnusedShapes.vue");
    let mut found: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.rule_name,
                "vue/no-unused-script-vars" | "vue/prefer-const"
            )
        })
        .map(|diagnostic| {
            let (line, column) = line_col(SHAPES, diagnostic.start);
            (
                line,
                column,
                diagnostic.rule_name,
                diagnostic.message.as_str().to_string(),
            )
        })
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            (
                2,
                10,
                "vue/no-unused-script-vars",
                "'computed' is defined but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                3,
                15,
                "vue/no-unused-script-vars",
                "'Component' is defined but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                6,
                7,
                "vue/no-unused-script-vars",
                "'unused' is assigned a value but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                9,
                16,
                "vue/no-unused-script-vars",
                "'second' is assigned a value but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                11,
                5,
                "vue/prefer-const",
                "'neverReassigned' is never reassigned. Use 'const' instead".to_string(),
            ),
            (
                13,
                31,
                "vue/no-unused-script-vars",
                "'b' is defined but never used. Allowed unused args must match /^_/u".to_string(),
            ),
            (
                17,
                9,
                "vue/no-unused-script-vars",
                "'inner' is assigned a value but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
        ]
    );
}

#[test]
fn happy_path_leaves_the_script_rules_quiet() {
    let result = Linter::new().lint_sfc(SHAPES, "UnusedShapes.vue");
    assert!(result.diagnostics.iter().all(|diagnostic| {
        !matches!(
            diagnostic.rule_name,
            "vue/no-unused-script-vars" | "vue/prefer-const"
        )
    }));
}

#[test]
fn script_only_sfc_still_reports_both_rules() {
    let source = "<script setup>\nconst unused = 1;\nlet kept = 2;\n</script>\n";
    let result = Linter::with_preset(LintPreset::Opinionated).lint_sfc(source, "Only.vue");
    let mut found: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.rule_name,
                "vue/no-unused-script-vars" | "vue/prefer-const"
            )
        })
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.message.as_str().to_string(),
            )
        })
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            (
                "vue/no-unused-script-vars",
                "'kept' is assigned a value but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                "vue/no-unused-script-vars",
                "'unused' is assigned a value but never used. Allowed unused vars must match /^_/u"
                    .to_string(),
            ),
            (
                "vue/prefer-const",
                "'kept' is never reassigned. Use 'const' instead".to_string(),
            ),
        ]
    );
}

fn line_col(source: &str, offset: u32) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    for (index, byte) in source.bytes().enumerate() {
        if index == offset as usize {
            break;
        }
        if byte == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
