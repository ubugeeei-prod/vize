//! #13080: restrict-plus-operands reads a generic intersection (`T & {}`,
//! `U & boolean`, `U & Date`) through the checker's base constraint.
use super::super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use serde_json::json;
use vize_l0::config::TypeAwareRuleOptions;

const RULE: &str = "type/restrict-plus-operands";

const NARROWED: &str = r#"<script setup lang="ts">
const truthy = <T extends boolean | undefined>(flag: T) => (flag ? "flag: " + flag : "");
const nullish = <U,>(value: U) => (value ?? "none") + "!";
const byTypeof = <U,>(value: U) => (typeof value === "boolean" ? "b: " + value : "");
const byInstance = <U,>(value: U) => (value instanceof Date ? "d: " + value : "");
const spelled = <T extends boolean | undefined>(flag: NonNullable<T>) => "flag: " + flag;
const bare = <T extends boolean | undefined>(flag: T) => "flag: " + flag;
const text = <U,>(value: U) => (typeof value === "string" ? value + "!" : "");
const union = <U extends boolean>(value: U | "none") => value + "!";
</script>

<template>
  <p>{{ [truthy, nullish, byTypeof, byInstance, spelled, bare, text, union].length }}</p>
</template>"#;

const NARROWED_PROP: &str = r#"<script setup lang="ts" generic="T extends boolean | undefined, U">
import { computed } from "vue";

const { flag, value } = defineProps<{ flag: T; value: U }>();

const truthy = computed(() => (flag ? "flag: " + flag : ""));
const nullish = computed(() => (value ?? "none") + "!");
const byTypeof = computed(() => (typeof value === "boolean" ? "b: " + value : ""));
const byInstance = computed(() => (value instanceof Date ? "d: " + value : ""));
const bare = computed(() => "flag: " + flag);
</script>

<template>
  <p>{{ truthy }} {{ nullish }} {{ byTypeof }} {{ byInstance }} {{ bare }}</p>
</template>"#;

fn linter() -> Linter {
    let options: TypeAwareRuleOptions = serde_json::from_value(json!({
        (RULE): {
            "allowAny": false,
            "allowBoolean": false,
            "allowNullish": false,
            "allowNumberAndString": true,
            "allowRegExp": false
        }
    }))
    .expect("type-aware options");
    Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_type_aware_rule_options(options)
}

/// `(operand text, reported type)` for each diagnostic, in source order.
fn reports<'a>(source: &'a str, filename: &str) -> Vec<(&'a str, String)> {
    let result = lint_sfc_with_corsa(&linter(), source, filename);
    let mut diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .collect::<Vec<_>>();
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            let operand = source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default();
            let got = diagnostic
                .message
                .split("Got `")
                .nth(1)
                .and_then(|rest| rest.split('`').next())
                .unwrap_or_default();
            (operand, got.into())
        })
        .collect()
}

#[test]
fn generic_intersections_are_read_through_their_base_constraint() {
    if !corsa_available() {
        return;
    }
    let expected = |pairs: &[(&'static str, &str)]| {
        pairs
            .iter()
            .map(|(operand, got)| (*operand, String::from(*got)))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        reports(NARROWED, "Narrowed.vue"),
        expected(&[
            ("flag", "boolean"),
            ("value ?? \"none\"", "{}"),
            ("value", "boolean"),
            ("value", "Date"),
            ("flag", "boolean"),
            ("flag", "undefined | boolean"),
            ("value", "string | boolean"),
        ])
    );
    assert_eq!(
        reports(NARROWED_PROP, "NarrowedProp.vue"),
        expected(&[
            ("flag", "boolean"),
            ("value ?? \"none\"", "{}"),
            ("value", "boolean"),
            ("value", "Date"),
            ("flag", "undefined | boolean"),
        ])
    );
}
