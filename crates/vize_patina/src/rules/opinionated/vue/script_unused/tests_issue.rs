//! #12883: template, `generic`, and `<style>` reads, a `let` assigned once
//! later, and caught errors without a default ignore pattern.

use super::tests::line_col;
use crate::{LintPreset, Linter};
use vize_l0::config::ConfigLintRuleOptions;

const TEMPLATE_USES: &str = r#"<script setup lang="ts" generic="Row extends RowBase">
import { ref } from "vue";

import { vFocus } from "./focus";
import type { RowBase } from "./rows";
import Swatch from "./Swatch.vue";
import Marker from "./Marker.vue";
import List from "./List.vue";

defineProps<{ rows: Row[] }>();

const tint = ref("red");
const panel = ref<HTMLElement | null>(null);
</script>

<template>
  <div ref="panel" v-focus>
    <svg viewBox="0 0 4 4"><Swatch /></svg>
    <List>
      <template #default="{ item }">
        <svg viewBox="0 0 4 4"><g><Marker :item="item" /></g></svg>
      </template>
    </List>
  </div>
</template>

<style scoped>
div {
  color: v-bind(tint);
}
</style>
"#;

const SCRIPT_SHAPES: &str = r#"<script setup lang="ts">
const { first, ...others } = { first: 1, second: 2 };
type LocalAlias = string;
interface LocalShape {
  size: number;
}
try {
  JSON.parse("{}");
} catch (error) {
  console.log("bad json");
}
try {
  JSON.parse("[]");
} catch (_ignored) {
  console.log("bad list");
}
let late: number;
late = 5;
for (let item of [1, 2]) {
  console.log(item, others, late);
}
let counter = 0;
let { left, right } = { left: 1, right: 2 };
right = 3;
console.log(left, right);
</script>

<template>
  <button type="button" @click="counter++">{{ counter }}</button>
</template>
"#;

pub(super) fn reports(
    source: &str,
    options: Option<&ConfigLintRuleOptions>,
) -> Vec<(usize, usize, &'static str, String)> {
    let mut linter = Linter::with_preset(LintPreset::Opinionated);
    if let Some(options) = options {
        linter = linter.with_config_rule_options(options);
    }
    let result = linter.lint_sfc(source, "Fixture.vue");
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
            let (line, column) = line_col(source, diagnostic.start);
            (
                line,
                column,
                diagnostic.rule_name,
                diagnostic.message.as_str().to_string(),
            )
        })
        .collect();
    found.sort();
    found
}

fn unused(line: usize, column: usize, message: &str) -> (usize, usize, &'static str, String) {
    (
        line,
        column,
        "vue/no-unused-script-vars",
        message.to_string(),
    )
}

fn prefer(line: usize, column: usize, name: &str) -> (usize, usize, &'static str, String) {
    (
        line,
        column,
        "vue/prefer-const",
        format!("'{name}' is never reassigned. Use 'const' instead"),
    )
}

#[test]
fn directive_generic_svg_style_and_ref_reads_count() {
    assert_eq!(reports(TEMPLATE_USES, None), vec![]);
}

#[test]
fn unread_template_shapes_still_report() {
    let source = r#"<script setup lang="ts">
import { vFocus } from "./focus";
import type { RowBase } from "./rows";
const tint = 1;
const panel = 2;
console.log(tint, panel);
</script>

<template>
  <div ref="other" v-blur>{{ RowBase }}</div>
</template>
"#;
    assert_eq!(
        reports(source, None),
        vec![unused(
            2,
            10,
            "'vFocus' is defined but never used. Allowed unused vars must match /^_/u"
        )]
    );
}

#[test]
fn script_shapes_match_typescript_eslint_and_prefer_const() {
    assert_eq!(
        reports(SCRIPT_SHAPES, None),
        vec![
            unused(
                2,
                9,
                "'first' is assigned a value but never used. Allowed unused vars must match /^_/u"
            ),
            unused(
                3,
                6,
                "'LocalAlias' is defined but never used. Allowed unused vars must match /^_/u"
            ),
            unused(
                4,
                11,
                "'LocalShape' is defined but never used. Allowed unused vars must match /^_/u"
            ),
            unused(9, 10, "'error' is defined but never used."),
            unused(14, 10, "'_ignored' is defined but never used."),
            prefer(18, 1, "late"),
            prefer(19, 10, "item"),
            prefer(23, 7, "left"),
        ]
    );
}

#[test]
fn a_let_assigned_once_later_follows_eslint_prefer_const() {
    let source = r#"<script setup lang="ts">
let late: number;
late = 5;
let readFirst: number;
console.log(readFirst);
readFirst = 1;
let inBlock: number;
if (late) {
  inBlock = 2;
}
let twice: number;
twice = 1;
twice = 2;
let never: number;
let a: number, b: number;
[a, b] = [1, 2];
let compound = 0;
compound += 1;
console.log(late, inBlock, twice, never, a, b, compound);
</script>
"#;
    assert_eq!(
        reports(source, None),
        vec![
            prefer(3, 1, "late"),
            prefer(4, 5, "readFirst"),
            prefer(16, 2, "a"),
            prefer(16, 5, "b"),
        ]
    );
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/prefer-const": { "ignoreReadBeforeAssign": true }
    }))
    .unwrap();
    assert_eq!(
        reports(source, Some(&options)),
        vec![prefer(3, 1, "late"), prefer(16, 2, "a"), prefer(16, 5, "b")]
    );
}

#[test]
fn typescript_eslint_options_reach_the_rule() {
    let source = r#"<script setup lang="ts">
const _unused = 1;
const [first, _second] = [1, 2];
function handler(_event: Event, value: number): number {
  return value + first;
}
try {
  handler(new Event("x"), 1);
} catch (ignoredError) {
  console.log("failed");
}
const { used, ...rest } = { used: 1, other: 2 };
console.log(rest);
class Registry {
  static {
    console.log("ready");
  }
}
const ignoredButRead = 2;
console.log(ignoredButRead);
</script>
"#;
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-unused-script-vars": {
            "vars": "all",
            "varsIgnorePattern": "^ignored",
            "args": "all",
            "argsIgnorePattern": "^_",
            "caughtErrors": "all",
            "caughtErrorsIgnorePattern": "^ignored",
            "destructuredArrayIgnorePattern": "^_",
            "ignoreRestSiblings": true,
            "ignoreClassWithStaticInitBlock": true,
            "reportUsedIgnorePattern": true
        }
    }))
    .unwrap();
    assert_eq!(
        reports(source, Some(&options)),
        vec![
            unused(
                2,
                7,
                "'_unused' is assigned a value but never used. Allowed unused vars must match /^ignored/u"
            ),
            unused(
                19,
                7,
                "'ignoredButRead' is marked as ignored but is used. Used vars must not match /^ignored/u"
            ),
        ]
    );
    // The defaults keep `^_` for variables and parameters and check `catch`.
    let defaults = reports(source, None);
    assert_eq!(
        defaults,
        vec![
            unused(9, 10, "'ignoredError' is defined but never used."),
            unused(
                12,
                9,
                "'used' is assigned a value but never used. Allowed unused vars must match /^_/u"
            ),
            unused(
                14,
                7,
                "'Registry' is defined but never used. Allowed unused vars must match /^_/u"
            ),
        ]
    );
}

#[test]
fn an_empty_pattern_turns_the_default_off() {
    let source = "<script setup lang=\"ts\">\nconst _unused = 1;\nfunction run(_event: Event) {}\nrun(new Event(\"x\"));\n</script>\n";
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-unused-script-vars": { "varsIgnorePattern": "", "argsIgnorePattern": "" }
    }))
    .unwrap();
    assert_eq!(
        reports(source, Some(&options)),
        vec![
            unused(2, 7, "'_unused' is assigned a value but never used."),
            unused(3, 14, "'_event' is defined but never used."),
        ]
    );
}
