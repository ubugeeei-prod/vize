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
