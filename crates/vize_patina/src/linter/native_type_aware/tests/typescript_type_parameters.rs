//! #12947: parity for no-unnecessary-type-parameters in SFC scripts.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const RULE: &str = "type/no-unnecessary-type-parameters";

#[test]
fn single_use_type_parameters_report_without_flagging_relational_generics() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
function identity<T>(value: T) {
  return value;
}

logValue(1);
const kept = identity("a");
</script>

<template>
  <p>{{ kept }}</p>
</template>"#;
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, source, "Generics.vue");
    let diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 1, "{:#?}", result.diagnostics);
    assert_eq!(
        diagnostics[0].message,
        "Type parameter T is used only once in the function signature."
    );
    assert_eq!(
        diagnostics[0].start as usize,
        source.find("logValue<T>").unwrap() + "logValue<".len()
    );
    assert_eq!(
        source.get(diagnostics[0].start as usize..diagnostics[0].end as usize),
        Some("T")
    );
}

#[test]
fn resolved_signature_usage_matches_typescript_eslint_8_72_2() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
class Box<T> {
  val: T | null = null;
  get(): T | null { return this.val; }
}
declare function fetchJson<T>(url: string): Promise<T>;
function getData<T>(url: string): Promise<T | null> {
  console.info(url);
  return Promise.resolve(null);
}
declare function makeSet<K>(): Set<K>;
declare function makeArray<T>(): T[];
function makeMap<K, V>() { return new Map<K, V>(); }
type Fn = <T>(input: T) => typeof input;
interface ItemProps<T> {
  item: T;
  onSelect: (item: T) => void;
}
function selectItem<T>(props: ItemProps<T>): void {
  props.onSelect(props.item);
}
function getLength<T extends { length: number }>(x: T) { return x.length; }
function join<T extends string | number>(els: T[]) {
  return els.map((el) => `${el}`).join(",");
}
function logValue<T>(value: T): void { console.info(value); }
function first<T>(items: T[]): T | undefined { return items[0]; }
interface Marker<T> { name: string; }
type Wrapped<T> = { value: T };
</script>"#;
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, source, "Shapes.vue");
    let actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            source
                .get(..diagnostic.start as usize)
                .unwrap_or(source)
                .matches('\n')
                .count()
                + 1
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, vec![22, 23, 26], "{:#?}", result.diagnostics);
}

#[test]
fn inferred_generic_returns_keep_shadowed_parameters_separate() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
function outer<T>() {
  return <T>(value: T): T => value;
}
class Container<T> {
  method<T>(value: T) { return value; }
}
</script>"#;
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, source, "Shadowed.vue");
    let actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            source
                .get(..diagnostic.start as usize)
                .unwrap_or(source)
                .matches('\n')
                .count()
                + 1
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, vec![2, 5], "{:#?}", result.diagnostics);
}

mod inferred;

mod corpus {
    include!("typescript_no_unnecessary_type_parameters_corpus.rs");
}
