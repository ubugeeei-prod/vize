//! Constrained `__type` facts behind the no-unnecessary-* type rules.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const SOURCE: &str = r#"<script setup lang="ts">
declare const text: string;
declare const maybe: string | undefined;
declare const box: { a: number };
declare const items: object[];
declare const flag: boolean;
const wrapped = `${text}`;
const optional = `${maybe}`;
const chained = box?.a;
if (items[0]) {
  console.log(items);
}
function convert<T extends string>(value: T): string {
  return String(value);
}
const doubled = !!flag;
function compare<T extends boolean>(value: T): boolean {
  return value === true;
}
function isText(value: unknown): value is string {
  return typeof value === "string";
}
if (isText(text)) {
  console.log(text);
}
</script>"#;

/// `(line, rule)` pairs typescript-eslint reports on [`SOURCE`] with default
/// options (`noUncheckedIndexedAccess` off, `checkTypePredicates` off).
const EXPECTED: &[(u32, &str)] = &[
    (7, "type/no-unnecessary-template-expression"),
    (9, "type/no-unnecessary-condition"),
    (14, "type/no-unnecessary-type-conversion"),
    (16, "type/no-unnecessary-type-conversion"),
    (18, "type/no-unnecessary-boolean-literal-compare"),
];

const RULES: &[&str] = &[
    "type/no-unnecessary-condition",
    "type/no-unnecessary-template-expression",
    "type/no-unnecessary-type-conversion",
    "type/no-unnecessary-boolean-literal-compare",
];

#[test]
fn type_shapes_drive_the_no_unnecessary_rules() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "TypeShapes.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| RULES.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            (
                line_of(SOURCE, diagnostic.start),
                diagnostic.rule_name,
                diagnostic.message.to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    let keys = actual
        .iter()
        .map(|(line, rule, _)| (*line, *rule))
        .collect::<Vec<_>>();
    assert_eq!(keys, EXPECTED, "findings: {actual:#?}");
    assert!(actual.iter().any(|(_, _, message)| message
        == "Passing a string to String() does not change the type or value of the string."));
    assert!(
        actual
            .iter()
            .any(|(_, _, message)| message == "Unnecessary optional chain on a non-nullish value.")
    );
}

fn line_of(source: &str, offset: u32) -> u32 {
    let prefix = source.get(..offset as usize).unwrap_or(source);
    1 + prefix.matches('\n').count() as u32
}
