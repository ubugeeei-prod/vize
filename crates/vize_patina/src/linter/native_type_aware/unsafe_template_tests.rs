use super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, lint_sfc_with_corsa, tests::corsa_available};
use crate::{Linter, rules::opinionated::type_aware::NoUnsafeTemplateBinding};

fn runtime_available() -> bool {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return false;
    }
    let available = corsa_available();
    if std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some() {
        assert!(available, "the full template-binding corpus requires Corsa");
    }
    available
}

#[test]
fn original_typed_template_binding_corpus_is_safe() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding");
    let mut failures = Vec::new();
    for relative in [
        "calls/src/App.vue",
        "component-values/ParentPanel.vue",
        "fallthrough/MyPanel.vue",
        "handlers/ParentPanel.vue",
    ] {
        let path = fixtures.join(relative);
        let source = std::fs::read_to_string(&path).expect("original fixture");
        let result = lint_sfc_with_corsa(&linter, &source, &path.to_string_lossy());
        let diagnostics: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                    || diagnostic.rule_name == "type/corsa-runtime"
            })
            .collect();
        if !diagnostics.is_empty() {
            failures.push((
                relative,
                diagnostics.into_iter().cloned().collect::<Vec<_>>(),
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn unresolved_component_imports_remain_unsafe() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let source = "<script setup lang=\"ts\">\nimport Missing from './DefinitelyMissing.vue'\n</script>\n<template><component :is=\"Missing\" /></template>";
    let result = lint_sfc_with_corsa(&linter, source, "MissingComponentControl.vue");
    let start = source.rfind("Missing").unwrap() as u32;
    let actual: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name.as_str(),
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        actual,
        vec![(
            RULE_NO_UNSAFE_TEMPLATE_BINDING,
            start,
            start + 7,
            "Template binding resolves to an unsafe `any` or `unknown` type"
        )]
    );
}

#[test]
fn authored_any_component_exports_remain_unsafe() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let source = "<script setup lang=\"ts\">\nimport Unsafe from './controls/UnsafeComponent.vue'\n</script>\n<template><component :is=\"Unsafe\" /></template>";
    let filename = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding/UnsafeComponentControl.vue");
    let result = lint_sfc_with_corsa(&linter, source, &filename.to_string_lossy());
    let start = source.rfind("Unsafe").unwrap() as u32;
    let actual: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name.as_str(),
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        actual,
        vec![(
            RULE_NO_UNSAFE_TEMPLATE_BINDING,
            start,
            start + 6,
            "Template binding resolves to an unsafe `any` or `unknown` type"
        )]
    );
}

#[test]
fn unsafe_template_binding_controls_keep_exact_ranges() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let source = r#"<script setup lang="ts">
const Child = {} as { new(): { $props: { label?: string, onPlay?: () => void } } }
const unsafeHandler: any = () => {}
function getUnsafe(): any { return 'unsafe' }
const unknownValue: unknown = []
let value: any = []
</script>
<template>
  <Child :label="getUnsafe()" />
  <Child @play="() => unsafeHandler()" />
  <button @click="value = unknownValue" />
  <button @click="() => (value = [])" />
  <button :disabled="!getUnsafe()" />
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "UnsafeTemplateControls.vue");
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "type/corsa-runtime"),
        "runtime failure: {:?}",
        result.diagnostics
    );
    let actual: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING)
        .map(|diagnostic| {
            (
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect();
    let range = |needle: &str| {
        let start = source.rfind(needle).expect("control range") as u32;
        (start, start + needle.len() as u32)
    };
    let call_start = source.find(":label=\"getUnsafe()").unwrap() as u32 + ":label=\"".len() as u32;
    let call = (call_start, call_start + "getUnsafe()".len() as u32);
    let callee = range("unsafeHandler()\" />");
    let assignment = range("value = unknownValue");
    assert_eq!(
        actual,
        vec![
            (
                call.0,
                call.0 + "getUnsafe()".len() as u32,
                "Template binding resolves to an unsafe `any` or `unknown` type"
            ),
            (
                callee.0,
                callee.0 + "unsafeHandler".len() as u32,
                "Template event handler calls a value with an unsafe `any` or `unknown` type"
            ),
            (
                assignment.0,
                assignment.1,
                "Template event handler resolves to an unsafe `any` or `unknown` type"
            ),
        ]
    );
}
