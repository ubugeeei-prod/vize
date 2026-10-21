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
fn tsconfig_path_aliased_vue_exports_and_slots_remain_typed() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/vue-alias-types-9864/src/pages");
    let mut failures = Vec::new();
    for name in ["page-view.vue", "list-view.vue"] {
        let path = fixtures.join(name);
        let source = std::fs::read_to_string(&path).expect("issue-authored alias fixture");
        let result = lint_sfc_with_corsa(&linter, &source, &path.to_string_lossy());
        let diagnostics: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                    || diagnostic.rule_name == "type/corsa-runtime"
            })
            .cloned()
            .collect();
        if !diagnostics.is_empty() {
            failures.push((name, diagnostics));
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
                diagnostic.rule_name,
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
    let filename = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding/UnsafeComponentControl.vue");
    for component in ["UnsafeComponent", "UnsafeSplitComponent"] {
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\">\nimport Unsafe from './controls/{component}.vue'\n</script>\n<template><component :is=\"Unsafe\" /></template>"
        );
        let result = lint_sfc_with_corsa(&linter, &source, &filename.to_string_lossy());
        let start = source.rfind("Unsafe").unwrap() as u32;
        let actual: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.rule_name,
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

#[test]
fn nested_and_else_if_conditions_keep_their_own_inferred_type() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let source = r#"<script setup lang="ts">
const isA = (o: { kind: string }) => o.kind === "a";
function getUnsafe(): any { return 'unsafe' }
const show = true as boolean;
</script>
<template>
  <div v-if="show">
    <p v-if="isA({ kind: 'a' })">nested</p>
    <p v-if="!isA({ kind: 'a' } as const)">nested</p>
    <p v-if="getUnsafe({ kind: 'a' })">nested unsafe</p>
  </div>
  <p v-if="show">shown</p>
  <p v-else-if="isA({ kind: 'a' })">else-if</p>
  <p v-else-if="getUnsafe({ kind: 'b' })">else-if unsafe</p>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "NestedConditions.vue");
    let actual: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                || diagnostic.rule_name == "type/corsa-runtime"
        })
        .map(|diagnostic| (diagnostic.start, diagnostic.end))
        .collect();
    let range = |needle: &str| {
        let start = source.find(needle).expect("condition") as u32;
        (start, start + needle.len() as u32)
    };
    assert_eq!(
        actual,
        vec![
            range("getUnsafe({ kind: 'a' })"),
            range("getUnsafe({ kind: 'b' })")
        ],
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn slot_outlet_calls_and_object_literals_use_their_inferred_types() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let source = r#"<script setup lang="ts">
defineSlots<{
  default?(props: { items: string[]; attrs: Record<string, string>; unsafe: unknown }): unknown;
}>();

const getItems = (): string[] => ["a", "b"];
const unsafeValue: any = "unsafe";
</script>

<template>
  <div :class="getItems()" :data-x="{ id: 'x' }.id" />
  <slot :items="getItems()" :attrs="{ id: 'x' }" :unsafe="unsafeValue" />
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "SlotOutletBindings.vue");
    let actual: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                || diagnostic.rule_name == "type/corsa-runtime"
        })
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect();
    let start = source.rfind("unsafeValue").expect("unsafe binding") as u32;
    assert_eq!(
        actual,
        vec![(
            RULE_NO_UNSAFE_TEMPLATE_BINDING,
            start,
            start + "unsafeValue".len() as u32,
            "Template binding resolves to an unsafe `any` or `unknown` type"
        )]
    );
}

#[test]
fn binding_comments_do_not_shift_callee_types() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let source = r#"<script setup lang="ts">
const MyInput = {} as { new(): { $props: { rules?: (true | string)[] } } };
const selected: number[] = [1];
function check(all: number[], current: number[]): true | string {
  return all.length > 0 || current.length > 0 || "empty";
}
const unsafeCheck: any = check;
</script>
<template>
  <MyInput :rules="[
    // checked against the other rows
    check(selected, [2])
  ]" />
  <MyInput :rules="[/* checked */ check(selected, [2])]" />
  <MyInput :rules="[
    /* checked against the other rows */
    check(selected, [2])
  ]" />
  <MyInput :rules="[/* unsafe control */ unsafeCheck(selected, [2])]" />
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "CommentedBinding.vue");
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING)
        .collect();
    assert_eq!(warnings.len(), 1, "{warnings:#?}");
    let unsafe_start = source.rfind("unsafeCheck").unwrap() as u32;
    assert_eq!(warnings[0].start, unsafe_start);
    assert_eq!(warnings[0].end, unsafe_start + "unsafeCheck".len() as u32);
}

#[test]
fn component_object_spreads_use_their_inferred_types() {
    if !runtime_available() {
        return;
    }
    let linter = Linter::new().with_rule(Box::new(NoUnsafeTemplateBinding::new()));
    let source = r#"<script setup lang="ts">
const MyCard = {} as { new(): { $props: { title?: string; topConfig?: { distance: number } } } };
const title = "hello";
const cardProps = { topConfig: { distance: 1 } };
const all = { title: "hi", topConfig: { distance: 2 } };
const unsafeProps: any = { title: "unsafe" };
</script>
<template>
  <MyCard v-bind="{ title }" />
  <MyCard v-bind="{ ...cardProps }" />
  <MyCard v-bind="all" />
  <MyCard :title="title" />
  <MyCard v-bind="unsafeProps" />
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "ComponentObjectSpread.vue");
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING)
        .collect();
    assert_eq!(warnings.len(), 1, "{warnings:#?}");
    let unsafe_start = source.rfind("unsafeProps").unwrap() as u32;
    assert_eq!(warnings[0].start, unsafe_start);
    assert_eq!(warnings[0].end, unsafe_start + "unsafeProps".len() as u32);
}
