//! Q16solver #11451 and #11452: the inference-only callback copy is unmapped,
//! so the host ignores every diagnostic in it, whatever the callback's shape.
//! The mapped copy keeps the authored tokens and their diagnostics.

use super::super::protocol::DIRECTIVE_POLICY_IGNORE;
use super::published;

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

fn edit(source: &str, from: &str, to: &str) -> Box<str> {
    assert_eq!(source.matches(from).count(), 1, "{from}");
    source.replace(from, to).into_boxed_str()
}

/// The resolver copy is wholly ignored, and `authored` is mapped elsewhere.
fn assert_inert_copy(case: &str, source: &str, authored: &str) {
    let result = published(source);
    let text = result.text.as_str();
    let start = text.find("const __VResolvedProps").expect(case);
    let end = start + text[start..].find("}));\n").expect(case) + "}));\n".len();
    let mut ignored = vec![false; end - start];
    for row in result
        .diagnostic_directives
        .iter()
        .flat_map(|table| &table.directives)
    {
        let [_, _, from, to, policy, _] = row.0;
        if policy == DIRECTIVE_POLICY_IGNORE {
            for offset in from.max(start)..to.min(end) {
                ignored[offset - start] = true;
            }
        }
    }
    assert!(
        ignored.iter().all(|byte| *byte),
        "{case}: unignored copy byte"
    );
    let authored_start = source.find(authored).expect(case);
    let mut owners = 0;
    for span in &result.mappings {
        let [generated, length, original, original_length, ..] = span.0;
        assert!(
            length == 0 || generated + length <= start || generated >= end,
            "{case}: mapped copy byte at {generated}"
        );
        owners += usize::from(
            length > 0 && original <= authored_start && authored_start < original + original_length,
        );
    }
    assert!(owners > 0, "{case}: authored `{authored}` lost its mapping");
}

#[test]
fn destructured_nested_parameters_stay_in_the_ignored_copy() {
    let original = DESTRUCTURED_11451;
    let cases = [
        ("original", Box::from(original), "qux }"),
        (
            "renamed",
            edit(
                &edit(original, "({ qux })", "({ qux: quux })"),
                "log(qux)",
                "log(quux)",
            ),
            "quux }",
        ),
        (
            "array",
            edit(
                &edit(original, "(baz: { qux: number })", "(baz: [number])"),
                "({ qux })",
                "([qux])",
            ),
            "qux]",
        ),
        (
            "rest",
            edit(
                &edit(original, "(baz: { qux: number })", "(...baz: number[])"),
                "({ qux })",
                "(...qux)",
            ),
            "qux)",
        ),
        (
            "default",
            edit(original, "({ qux })", "({ qux } = { qux: 1 })"),
            "qux } =",
        ),
        (
            "function",
            edit(original, "({ qux }) => {", "function ({ qux }) {"),
            "qux }",
        ),
        (
            "expression-body",
            edit(
                original,
                "({ qux }) => {\n          console.log(qux);\n        });",
                "({ qux }) => console.log(qux));",
            ),
            "qux }",
        ),
        (
            "authored-error",
            edit(
                original,
                "console.log(qux);",
                "console.log(qux);\n          qux.nope();",
            ),
            "nope",
        ),
    ];
    for (case, source, authored) in &cases {
        assert_inert_copy(case, source, authored);
    }
}

#[test]
fn assignments_in_callback_bodies_stay_in_the_ignored_copy() {
    let original = ASSIGNED_11452;
    let cases = [
        ("original", Box::from(original), "foo.width"),
        (
            "local-let",
            edit(
                &edit(original, "import { ref } from \"vue\";\n", ""),
                "const foo = ref<Foo | undefined>();",
                "let foo: Foo | undefined;",
            ),
            "foo.width",
        ),
        (
            "same-line",
            edit(
                original,
                "foo = baz;\n        console.log",
                "foo = baz; console.log",
            ),
            "foo.width",
        ),
        (
            "authored-error",
            edit(
                original,
                "console.log(foo.width);",
                "console.log(foo.width);\n        console.log(foo.nope);",
            ),
            "nope",
        ),
    ];
    for (case, source, authored) in &cases {
        assert_inert_copy(case, source, authored);
    }
}
