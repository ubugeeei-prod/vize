//! `type/no-unnecessary-type-parameters` (#12947): written uses decide most
//! parameters, the checker counts the uses an inferred type or a class
//! member adds.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const SOURCE: &str = r#"<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
function identity<T>(value: T): T {
  return value;
}
function wrap<T>(value: T) {
  return [value];
}
function first<K, V>(map: Map<K, V>): K | undefined {
  return map.keys().next().value;
}
class Box<T> {
  value?: T;
}
class Pair<T> {
  constructor(readonly left: T, readonly right: T) {}
}
logValue(1);
const kept = identity("a");
export { kept, wrap, first, Box, Pair };
</script>"#;

/// `(line, column, message)`, with the `<script>` tag on line 1.
const EXPECTED: &[(u32, u32, &str)] = &[
    (
        2,
        19,
        "Type parameter T is used only once in the function signature.",
    ),
    (
        14,
        11,
        "Type parameter T is used only once in the class signature.",
    ),
];

#[test]
fn reports_type_parameters_used_once() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "TypeParameters.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "type/no-unnecessary-type-parameters")
        .map(|diagnostic| {
            let (line, column) = line_column(SOURCE, diagnostic.start);
            (line, column, diagnostic.message.to_string())
        })
        .collect::<Vec<_>>();
    actual.sort();
    let expected = EXPECTED
        .iter()
        .map(|(line, column, message)| (*line, *column, (*message).to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{:#?}", result.diagnostics);
}

fn line_column(source: &str, offset: u32) -> (u32, u32) {
    let prefix = source.get(..offset as usize).unwrap_or(source);
    let line = prefix.matches('\n').count() as u32 + 1;
    let column = prefix
        .rsplit('\n')
        .next()
        .map_or(0, |last| last.chars().count()) as u32
        + 1;
    (line, column)
}
