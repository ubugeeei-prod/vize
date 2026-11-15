//! #13004: a record indexed by its own type parameter retains the value type.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const SOURCE: &str = r#"<script setup lang="ts">
interface Badge {
  count: number;
}

function generic<K extends string>(map: Record<K, Badge>, key: K): string {
  return String(map[key]);
}
function joined<K extends string>(map: Record<K, Badge>, keys: K[]): string {
  return keys.map((key) => map[key]).join(",");
}
function partial<K extends string>(map: Partial<Record<K, Badge>>, key: K): string {
  return String(map[key]);
}
function plain(map: Record<"a" | "b", Badge>, key: "a" | "b"): string {
  return String(map[key]);
}
function safe<K extends string>(map: Record<K, string>, key: K): string {
  return String(map[key]);
}
</script>"#;

const EXPECTED: &[(u32, u32)] = &[(7, 17), (10, 10), (13, 17), (16, 17)];

#[test]
fn record_indexed_by_its_type_parameter_keeps_the_value_shape() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "Keyed.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "type/no-base-to-string")
        .map(|diagnostic| line_column(SOURCE, diagnostic.start))
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(actual, EXPECTED);
}

fn line_column(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1u32;
    let mut column = 1u32;
    for (index, character) in source.char_indices() {
        if index >= offset as usize {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
