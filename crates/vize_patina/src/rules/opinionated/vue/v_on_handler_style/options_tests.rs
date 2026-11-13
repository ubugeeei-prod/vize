//! eslint-plugin-vue's documented examples for each `v-on-handler-style` option.

use super::VOnHandlerStyle;
use crate::linter::Linter;
use crate::rule::RuleRegistry;
use crate::{LintPreset, Locale};
use vize_l0::config::ConfigLintRuleOptions;

fn linter(options: serde_json::Value) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(VOnHandlerStyle::from_options(
        serde_json::from_value(options).unwrap(),
    )));
    Linter::with_registry(registry)
}

/// Each report as the handler text and its message.
fn reports(options: serde_json::Value, template: &str) -> Vec<(vize_l0::String, vize_l0::String)> {
    let result = linter(options).lint_template(template, "App.vue");
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                template
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default()
                    .into(),
                diagnostic.message.as_str().into(),
            )
        })
        .collect()
}

fn handlers(options: serde_json::Value, template: &str) -> Vec<vize_l0::String> {
    reports(options, template)
        .into_iter()
        .map(|(handler, _)| handler)
        .collect()
}

#[test]
fn method_or_inline_function_matches_the_upstream_default() {
    let template = r#"<div>
  <button v-on:click="handler" />
  <template v-for="e in list">
    <button v-on:click="e" />
    <button v-on:click="() => handler(e)" />
  </template>
  <button v-on:click="handler()" />
  <button v-on:click="count++" />
  <button v-on:click="() => handler()" />
  <button v-on:click="() => count++" />
  <button v-on:click="(a, b) => handler(a, b)" />
  <template v-for="e in list">
    <button v-on:click="e()" />
    <button v-on:click="() => e()" />
    <button v-on:click="handler(e)" />
  </template>
</div>"#;
    assert_eq!(
        reports(serde_json::json!(["method", "inline-function"]), template),
        [
            ("handler()", "Prefer method handler over inline handler in v-on."),
            (
                "count++",
                "Prefer method handler over inline handler in v-on. Note that you may need to create a new method.",
            ),
            ("() => handler()", "Prefer method handler over inline function in v-on."),
            (
                "() => count++",
                "Prefer method handler over inline function in v-on. Note that you may need to create a new method.",
            ),
            ("(a, b) => handler(a, b)", "Prefer method handler over inline function in v-on."),
            ("e()", "Prefer method handler over inline handler in v-on."),
            ("() => e()", "Prefer method handler over inline function in v-on."),
            ("handler(e)", "Prefer inline function over inline handler in v-on."),
        ]
        .map(|(handler, message)| (handler.into(), message.into()))
    );
}

#[test]
fn method_or_inline_matches_upstream() {
    let template = r#"<div>
  <button v-on:click="handler" />
  <template v-for="e in list">
    <button v-on:click="e" />
    <button v-on:click="handler(e)" />
  </template>
  <button v-on:click="handler()" />
  <button v-on:click="(a, b) => handler(a, b)" />
  <template v-for="e in list">
    <button v-on:click="() => handler(e)" />
  </template>
</div>"#;
    assert_eq!(
        reports(serde_json::json!(["method", "inline"]), template),
        [
            (
                "handler()",
                "Prefer method handler over inline handler in v-on."
            ),
            (
                "(a, b) => handler(a, b)",
                "Prefer method handler over inline function in v-on."
            ),
            (
                "() => handler(e)",
                "Prefer inline handler over inline function in v-on."
            ),
        ]
        .map(|(handler, message)| (handler.into(), message.into()))
    );
}

#[test]
fn a_single_style_reports_the_other_two() {
    let inline_function = r#"<div>
  <button v-on:click="() => handler()" />
  <button v-on:click="() => count++" />
  <button v-on:click="handler" />
  <button v-on:click="handler($event)" />
</div>"#;
    assert_eq!(
        reports(serde_json::json!("inline-function"), inline_function),
        [
            (
                "handler",
                "Prefer inline function over method handler in v-on."
            ),
            (
                "handler($event)",
                "Prefer inline function over inline handler in v-on."
            ),
        ]
        .map(|(handler, message)| (handler.into(), message.into()))
    );
    let inline = r#"<div>
  <button v-on:click="handler()" />
  <button v-on:click="count++" />
  <button v-on:click="handler" />
  <button v-on:click="(foo) => handler(foo)" />
  <button v-on:click="(foo, bar) => handler(foo, bar)" />
</div>"#;
    assert_eq!(
        reports(serde_json::json!("inline"), inline),
        [
            ("handler", "Prefer inline handler over method handler in v-on."),
            ("(foo) => handler(foo)", "Prefer inline handler over inline function in v-on."),
            (
                "(foo, bar) => handler(foo, bar)",
                "Prefer inline handler over inline function in v-on. Note that the custom event must be changed to a single payload.",
            ),
        ]
        .map(|(handler, message)| (handler.into(), message.into()))
    );
}

#[test]
fn ignore_includes_comment_moves_a_commented_handler_to_the_second_style() {
    let template = r#"<div>
  <button v-on:click="handler" />
  <button v-on:click="() => handler() /* comment */" />
  <button v-on:click="handler()" />
  <button v-on:click="() => handler()" />
  <button v-on:click="handler() /* comment */" />
</div>"#;
    let options = serde_json::json!({
        "style": ["method", "inline-function"],
        "ignoreIncludesComment": true,
    });
    assert_eq!(
        reports(options, template)
            .into_iter()
            .map(|(_, message)| message)
            .collect::<Vec<_>>(),
        [
            "Prefer method handler over inline handler in v-on.",
            "Prefer method handler over inline function in v-on.",
            "Prefer inline function over inline handler in v-on.",
        ]
    );
    // Patina's own method-only style honours the flag too.
    assert_eq!(
        handlers(
            serde_json::json!({ "ignoreIncludesComment": true }),
            r#"<div><button @click="save() // keep" /><button @click="save('//')" /></div>"#,
        ),
        ["save('//')"]
    );
}

#[test]
fn the_issue_reproduction_keeps_handlers_a_method_reference_cannot_express() {
    let template = r#"<div>
  <ul>
    <li v-for="item in items" :key="item.id">
      <button type="button" @click="() => pick(item)">pick</button>
    </li>
    <li v-for="(item, index) in items" :key="item.id">
      <button type="button" @click="(event: Event) => open(index, event)">open</button>
    </li>
  </ul>
  <button type="button" @click="send">a</button>
  <button type="button" @click="() => send()">b</button>
  <button type="button" @click="send()">c</button>
  <button type="button" @click="count++">d</button>
  <Picker v-slot="{ pick }"><button @click="(event) => pick(event, 1)" /></Picker>
  <button @click="actions.save" /><button @click="actions?.save" />
</div>"#;
    assert_eq!(
        handlers(serde_json::json!(["method", "inline-function"]), template),
        ["() => send()", "send()", "count++"]
    );
}

#[test]
fn a_shadowing_parameter_is_not_a_v_for_read() {
    assert_eq!(
        handlers(
            serde_json::json!(["method", "inline-function"]),
            r#"<div v-for="column in columns"><button @click="(column) => save(column?.id)" /></div>"#,
        ),
        ["(column) => save(column?.id)"]
    );
}

#[test]
fn config_rule_options_reach_the_rule_and_keep_localized_messages() {
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/v-on-handler-style": ["method", "inline-function"]
    }))
    .unwrap();
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/v-on-handler-style".into()]))
        .with_config_rule_options(&options)
        .with_locale(Locale::Ja)
        .lint_sfc(
            r#"<template><div v-for="item in items"><button @click="() => pick(item)" /><button @click="() => go()" /></div></template>"#,
            "App.vue",
        );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        ["v-on ではインライン関数ではなくメソッドハンドラを使用してください。"]
    );
}
