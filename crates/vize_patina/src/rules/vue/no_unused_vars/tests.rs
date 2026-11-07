use super::NoUnusedVars;
use crate::linter::Linter;
use crate::rule::{Rule, RuleCategory};

#[test]
fn test_meta() {
    let rule = NoUnusedVars::default();
    assert_eq!(rule.meta().name, "vue/no-unused-vars");
    assert_eq!(rule.meta().category, RuleCategory::Essential);
}

#[test]
fn test_default_does_not_ignore_underscore_prefix() {
    let rule = NoUnusedVars::default();
    assert!(!rule.should_ignore("_item"));
    assert!(!rule.should_ignore("_"));
    assert!(!rule.should_ignore("item"));
    assert!(!rule.should_ignore("index"));
}

#[test]
fn same_element_v_if_counts_as_upstream_usage() {
    let source = r#"<template>
  <img v-for="(url, index) in urls" v-if="index === selected" :src="url" />
</template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
        .lint_sfc(source, "Probe.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/no-unused-vars"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn same_element_slot_prop_directive_counts_as_usage() {
    let source = r#"<template>
  <FormKit v-slot="{ state: { loading }, unused }" :label="loading ? 'Wait' : 'Go'" />
</template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
        .lint_sfc(source, "Probe.vue");
    let messages: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-unused-vars")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        messages,
        ["Slot prop 'unused' from slot 'default' is defined but never used"]
    );
}

#[test]
fn authored_directive_use_survives_a_repaired_subtree() {
    let source = r#"<template>
  <a><a v-for="(file, index) in files" @click="remove(index)">{{ file }}</a></a>
</template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
        .lint_sfc(source, "Probe.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/no-unused-vars"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn vue2_slot_scope_and_unparenthesized_v_for_report_unused_aliases() {
    let source = r#"<template>
  <Child><template slot-scope="{ used, unused }">{{ used }}</template></Child>
  <div v-for="item,index in items">{{ item }}</div>
</template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
        .lint_sfc(source, "Probe.vue");
    let messages: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-unused-vars")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages.len(), 2, "{:?}", result.diagnostics);
    assert_eq!(
        messages,
        [
            "Slot prop 'unused' from slot 'default' is defined but never used",
            "Key variable 'index' is defined by v-for but never used",
        ]
    );
}

#[test]
fn generated_parenthesized_v_for_reports_the_unused_index() {
    let source = r#"<template>
  <view v-for="(item,index) in ('item in 12')" :key="item.index" v-if="type === 'spinner'" />
</template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
        .lint_sfc(source, "Probe.vue");
    let messages: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-unused-vars")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        messages,
        ["Key variable 'index' is defined by v-for but never used"]
    );
}

#[test]
fn generated_parenthesized_v_for_on_explicit_empty_element_reports_the_unused_index() {
    let source = r#"<template>
  <view v-for="(item,index) in ('item in 12')" :key="item.index"></view>
</template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
        .lint_sfc(source, "Probe.vue");
    let messages: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-unused-vars")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        messages,
        ["Key variable 'index' is defined by v-for but never used"]
    );
}
