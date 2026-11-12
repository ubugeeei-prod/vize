//! #12875: every typescript-eslint type-aware check fires on TypedShapes.vue.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const SOURCE: &str = r#"<script setup lang="ts">
declare const anyValue: any;

/** @deprecated use current() */
function legacy(): number {
  return 1;
}
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
function plainNumber(): number {
  return 1;
}
function logVoid(): void {}
function takesNumber(value: number): number {
  return value;
}
function identity<T = number>(value: T): T {
  return value;
}

enum Color {
  Red = 0,
  Green = 1,
}
enum Mixed {
  A = 0,
  B = "b",
}
type Dup = string | string;
type Redundant = unknown | string;
type Emptied = Omit<{ a: number }, "a">;
interface Box {
  get size(): string;
  set size(value: number);
}
class Builder {
  setValue(): Builder {
    return this;
  }
}
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}

const numbers = [1, 2, 3];
const label = "a";
const plain = { a: 1 };
const known: number = 1;
const counter = new Counter();

async function awaitsNumber(): Promise<number> {
  const result = await plainNumber();
  return result;
}
function deletesItem(): void {
  delete numbers[0];
}
const plainText = plain.toString();
const voidValue = logVoid();
const legacyValue = legacy();
function loopsKeys(): void {
  for (const key in numbers) {
    console.log(key);
  }
}
setTimeout("logVoid()", 10);
void logVoid();
numbers.forEach(async (n) => {
  console.log(await getNumber(), n);
});
const chars = [...label];
function isFlag(flag: boolean): boolean {
  return flag === true;
}
function orDefault(value: string): string {
  return value ?? "x";
}
const wrapped = `${label}`;
const fromIdentity = identity<number>(1);
const same = String(label);
const color: Color = 1;
function isRed(value: number): boolean {
  return value === Color.Red;
}
const negated = -label;
function fail(): never {
  throw "failure";
}
const rejected = Promise.reject("failure");
const collected = numbers.reduce((all, n) => [...all, n], [] as number[]);
async function noAwait(): Promise<number> {
  return 1;
}
const mixedSum = label + numbers;
const listText = `${numbers}`;
async function returnsInTry(): Promise<number> {
  await Promise.resolve();
  try {
    return getNumber();
  } catch {
    return 0;
  }
}
const unbound = counter.increment;
getNumber().catch((error) => {
  console.log(error);
});
const asserted = known as number;
takesNumber(anyValue);
const assigned: number = anyValue;
anyValue();
console.log(anyValue.member);
function returnsAny(): number {
  return anyValue;
}
getNumber();
</script>

<template>
  <p>{{ label }}</p>
</template>"#;

const EXPECTED: &[(u32, u32, &str)] = &[
    (28, 7, "type/no-mixed-enums"),
    (30, 21, "type/no-duplicate-type-constituents"),
    (31, 18, "type/no-redundant-type-constituents"),
    (32, 16, "type/no-generated-empty-object-type"),
    (34, 15, "type/related-getter-setter-pairs"),
    (38, 15, "type/prefer-return-this-type"),
    (56, 18, "type/await-thenable"),
    (60, 3, "type/no-array-delete"),
    (62, 19, "type/no-base-to-string"),
    (63, 19, "type/no-confusing-void-expression"),
    (64, 21, "type/no-deprecated"),
    (66, 3, "type/no-for-in-array"),
    (70, 12, "type/no-implied-eval"),
    (71, 1, "type/no-meaningless-void-operator"),
    (71, 6, "type/no-confusing-void-expression"),
    (72, 17, "type/no-misused-promises"),
    (75, 16, "type/no-misused-spread"),
    (77, 10, "type/no-unnecessary-boolean-literal-compare"),
    (80, 10, "type/no-unnecessary-condition"),
    (82, 18, "type/no-unnecessary-template-expression"),
    (83, 31, "type/no-unnecessary-type-arguments"),
    (84, 14, "type/no-unnecessary-type-conversion"),
    (85, 7, "type/no-unsafe-enum-assignment"),
    (87, 10, "type/no-unsafe-enum-comparison"),
    (89, 17, "type/no-unsafe-unary-minus"),
    (91, 9, "type/only-throw-error"),
    (93, 18, "type/prefer-promise-reject-errors"),
    (94, 59, "type/prefer-reduce-type-parameter"),
    (95, 1, "type/require-await"),
    (98, 26, "type/restrict-plus-operands"),
    (99, 21, "type/restrict-template-expressions"),
    (103, 12, "type/return-await"),
    (108, 17, "type/unbound-method"),
    (109, 20, "type/use-unknown-in-catch-callback-variable"),
    (112, 18, "type/no-unnecessary-type-assertion"),
    (113, 13, "type/no-unsafe-argument"),
    (114, 7, "type/no-unsafe-assignment"),
    (115, 1, "type/no-unsafe-call"),
    (116, 22, "type/no-unsafe-member-access"),
    (118, 3, "type/no-unsafe-return"),
];

#[test]
fn typed_shapes_report_the_typescript_eslint_checks() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "TypedShapes.vue");
    let wanted: std::collections::HashSet<&str> =
        EXPECTED.iter().map(|(_, _, rule)| *rule).collect();
    let mut actual = Vec::new();
    for diagnostic in &result.diagnostics {
        if !wanted.contains(diagnostic.rule_name) {
            continue;
        }
        let (line, column) = line_column(SOURCE, diagnostic.start);
        actual.push((
            line,
            column,
            diagnostic.rule_name,
            diagnostic.message.to_string(),
        ));
    }
    actual.sort_by_key(|(line, column, rule, _)| (*line, *column, *rule));
    let mut expected = EXPECTED.to_vec();
    expected.sort();
    let actual_keys: Vec<(u32, u32, &str)> = actual
        .iter()
        .map(|(line, column, rule, _)| (*line, *column, *rule))
        .collect();
    assert_eq!(
        actual_keys,
        expected,
        "typescript-eslint findings:\n{}",
        actual
            .iter()
            .map(|(line, column, rule, message)| format!("{line}:{column} {rule} {message}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
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
