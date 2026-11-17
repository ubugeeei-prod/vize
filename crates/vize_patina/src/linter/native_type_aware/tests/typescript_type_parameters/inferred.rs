//! #13084: a type parameter's uses are read from the checker's signature, so a
//! use in an inferred type predicate, in a parameter typed by its default, or
//! in a class property typed by its initializer counts like a written one.
//! The issue-authored files are the pinned differential inputs.
use super::super::{corsa_available, failure_context, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const RULE: &str = "type/no-unnecessary-type-parameters";

macro_rules! pinned {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/no-unnecessary-type-parameters-13084/",
            $path
        ))
    };
}

const INFERRED: &str = pinned!("src/Inferred.vue.txt");
const PROPERTY: &str = pinned!("src/Property.vue.txt");

/// The same inferred uses in a declaration and in class methods, beside
/// signatures whose only use really is single.
const SHAPES: &str = r#"<script setup lang="ts">
declare const seed: number;

function guard<T>(value: T | null | undefined) {
  return value != null;
}
class Tools {
  present<T>(value: T | null | undefined) {
    return value != null;
  }
  fallback<T>(value: T, other = value) {
    return String(other);
  }
}
class Pair<T> {
  first = undefined as T | undefined;
  second?: T;
}
const once = <T,>(value: T) => String(value);
const unused = <T,>(other = seed) => String(other);
</script>

<template>
  <p>{{ guard }} {{ Tools }} {{ Pair }} {{ once }} {{ unused }}</p>
</template>
"#;

fn reported_lines(source: &str, name: &str) -> Vec<usize> {
    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let result = lint_sfc_with_corsa(&linter, source, name);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name == RULE),
        "{}",
        failure_context(&result)
    );
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            source
                .get(..diagnostic.start as usize)
                .unwrap_or(source)
                .matches('\n')
                .count()
                + 1
        })
        .collect()
}

#[test]
fn inferred_predicates_and_default_parameters_count_as_uses() {
    if !corsa_available() {
        return;
    }
    assert_eq!(
        reported_lines(INFERRED, "Inferred.vue"),
        Vec::<usize>::new()
    );
    // Only `once` (one use) and `unused` (no use) remain.
    assert_eq!(reported_lines(SHAPES, "Shapes.vue"), vec![19, 20]);
}

#[test]
fn class_properties_count_their_inferred_type_once() {
    if !corsa_available() {
        return;
    }
    // `Holder` and `Slot` each use `T` once, as typescript-eslint reports.
    assert_eq!(reported_lines(PROPERTY, "Property.vue"), vec![4, 8]);
}
