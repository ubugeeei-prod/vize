use crate::linter::Linter;

const RULE: &str = "script/no-template-ref-outside-instance";

fn lint(source: &str, filename: &str) -> Vec<(String, String)> {
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    let result = if filename.ends_with(".vue") {
        linter.lint_sfc(source, filename)
    } else {
        linter.lint_script(source, filename)
    };
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, RULE);
            let snippet = source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default();
            let label = diagnostic
                .labels
                .first()
                .map(|label| label.message.to_string())
                .unwrap_or_default();
            (snippet.to_string(), label)
        })
        .collect()
}

fn snippets(source: &str) -> Vec<String> {
    lint(source, "Test.vue")
        .into_iter()
        .map(|(snippet, _)| snippet)
        .collect()
}

fn setup(script: &str, template: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nimport {{ onMounted, useTemplateRef, watch, nextTick }} from 'vue'\n{script}\n</script>\n<template>\n{template}\n</template>\n"
    )
}

#[test]
fn setup_body_and_lifecycle_hooks_are_allowed() {
    let source = setup(
        "const input = useTemplateRef('input')\nonMounted(() => { const late = useTemplateRef('late') })\nconst list = computed(() => useTemplateRef('list'))\nif (ok) { const branch = useTemplateRef('branch') }",
        "<input ref=\"input\" />",
    );
    assert!(snippets(&source).is_empty());
}

#[test]
fn module_level_calls_are_reported() {
    let source = "<script lang=\"ts\">\nimport { useTemplateRef } from 'vue'\nexport const shared = useTemplateRef('shared')\nexport function useInput() { return useTemplateRef('input') }\n</script>\n<script setup lang=\"ts\">\nconst input = useTemplateRef('input')\n</script>\n";
    assert_eq!(
        lint(source, "Test.vue"),
        [(
            "useTemplateRef('shared')".to_string(),
            "called at the module level".to_string()
        )]
    );
    let module = "import { useTemplateRef as utr } from 'vue'\nconst el = utr('el')\nexport function useEl() { return utr('el') }\n";
    assert_eq!(
        lint(module, "use-el.ts"),
        [(
            "utr('el')".to_string(),
            "called at the module level".to_string()
        )]
    );
}

#[test]
fn deferred_callbacks_are_reported() {
    let source = setup(
        "setTimeout(() => useTemplateRef('timer'))\nwindow.requestAnimationFrame(function () { useTemplateRef('frame') })\nwatch(source, () => { useTemplateRef('watched') })\nnextTick(() => useTemplateRef('tick'))\nload().then(() => useTemplateRef('then'))\nonMounted(() => { document.addEventListener('keydown', () => useTemplateRef('key')) })",
        "<div />",
    );
    let found = lint(&source, "Test.vue");
    let labels: Vec<&str> = found.iter().map(|(_, label)| label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "called in a `setTimeout` callback",
            "called in a `requestAnimationFrame` callback",
            "called in a `watch` callback",
            "called in a `nextTick` callback",
            "called in a `then` callback",
            "called in a `addEventListener` callback",
        ]
    );
}

#[test]
fn an_immediate_watch_callback_is_not_reported() {
    let source = setup(
        "watch(source, () => useTemplateRef('a'), { immediate: true })\nwatch(source, () => useTemplateRef('b'), options)\nwatch(source, () => useTemplateRef('c'), { deep: true })",
        "<div />",
    );
    assert_eq!(snippets(&source), ["useTemplateRef('c')"]);
}

#[test]
fn template_event_handlers_are_reported() {
    let source = setup(
        "function focus() {\n  useTemplateRef('input').value?.focus()\n}\nconst open = () => useTemplateRef('dialog')",
        "<input ref=\"input\" />\n<button @click=\"focus\" />\n<button @click=\"() => open()\" />",
    );
    let found = lint(&source, "Test.vue");
    assert_eq!(
        found,
        [
            (
                "useTemplateRef('input')".to_string(),
                "called in `focus`, which setup never calls".to_string()
            ),
            (
                "useTemplateRef('dialog')".to_string(),
                "called in `open`, which setup never calls".to_string()
            ),
        ]
    );
}

#[test]
fn local_functions_setup_calls_are_allowed() {
    let source = setup(
        "function init() { return useTemplateRef('input') }\nconst input = init()\nfunction nested() { return useTemplateRef('nested') }\nfunction outer() { return nested() }\nonMounted(() => outer())",
        "<input ref=\"input\" />\n<button @click=\"init\" />",
    );
    assert!(snippets(&source).is_empty());
}

#[test]
fn local_functions_with_unknown_callers_are_not_reported() {
    let source = setup(
        "function passed() { useTemplateRef('a') }\nuseFoo(passed)\nfunction exposed() { useTemplateRef('b') }\ndefineExpose({ exposed })\nfunction rendered() { useTemplateRef('c') }\nfunction unused() { useTemplateRef('d') }",
        "<p>{{ rendered() }}</p>\n<button @click=\"rendered\" />",
    );
    assert!(snippets(&source).is_empty());
}

#[test]
fn locals_reached_only_from_deferred_code_are_reported() {
    let source = setup(
        "function later() { return useTemplateRef('later') }\nsetTimeout(later, 100)\nfunction inner() { return useTemplateRef('inner') }\nfunction handler() { inner() }",
        "<button @click=\"handler\" />",
    );
    assert_eq!(
        snippets(&source),
        ["useTemplateRef('later')", "useTemplateRef('inner')"]
    );
}

#[test]
fn options_api_setup_is_an_instance_owner() {
    let source = "<script>\nimport { defineComponent, useTemplateRef } from 'vue'\nexport default defineComponent({\n  setup() {\n    const el = useTemplateRef('el')\n    function open() { useTemplateRef('dialog') }\n    function later() { useTemplateRef('later') }\n    setTimeout(later)\n    return { el, open }\n  },\n  methods: {\n    focus() { useTemplateRef('method') },\n  },\n})\n</script>\n";
    assert_eq!(snippets(source), ["useTemplateRef('later')"]);
    let function_form = "import { defineComponent, useTemplateRef } from 'vue'\nexport default defineComponent(() => {\n  const el = useTemplateRef('el')\n  return () => null\n})\n";
    assert!(lint(function_form, "Comp.ts").is_empty());
}

#[test]
fn only_vue_template_ref_is_checked() {
    let local = "<script>\nconst useTemplateRef = (key) => key\nconst el = useTemplateRef('el')\n</script>\n";
    assert!(snippets(local).is_empty());
    let other = "<script>\nimport { useTemplateRef } from './refs'\nconst el = useTemplateRef('el')\n</script>\n";
    assert!(snippets(other).is_empty());
    let global = "<script>\nconst el = useTemplateRef('el')\n</script>\n";
    assert_eq!(snippets(global), ["useTemplateRef('el')"]);
    let shadowed_timer = setup(
        "const setTimeout = (run) => run()\nsetTimeout(() => useTemplateRef('sync'))",
        "<div />",
    );
    assert!(snippets(&shadowed_timer).is_empty());
}

#[test]
fn a_sibling_block_can_hide_the_auto_imported_global() {
    let source = "<script>\nimport { useTemplateRef } from './refs'\n</script>\n<script setup>\nsetTimeout(() => useTemplateRef('el'))\n</script>\n";
    assert!(snippets(source).is_empty());
    let vue = "<script>\nimport { useTemplateRef } from 'vue'\n</script>\n<script setup>\nsetTimeout(() => useTemplateRef('el'))\n</script>\n";
    assert_eq!(snippets(vue), ["useTemplateRef('el')"]);
}

#[test]
fn calls_after_await_are_left_to_the_lifecycle_rule() {
    let source = "<script>\nimport { useTemplateRef } from 'vue'\nexport default {\n  async setup() {\n    await load()\n    const el = useTemplateRef('el')\n    return { el }\n  },\n}\n</script>\n";
    assert!(snippets(source).is_empty());
}

#[test]
fn pinned_differential_fixtures() {
    let module = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/template-ref-outside-instance/ModuleLevel.vue.txt"
    );
    assert_eq!(snippets(module), ["useTemplateRef<HTMLDivElement>('el')"]);
    let deferred = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/template-ref-outside-instance/DeferredCallbacks.vue.txt"
    );
    assert_eq!(
        lint(deferred, "DeferredCallbacks.vue")
            .into_iter()
            .map(|(_, label)| label)
            .collect::<Vec<_>>(),
        [
            "called in `focus`, which setup never calls",
            "called in a `setTimeout` callback",
            "called in a `watch` callback",
            "called in a `then` callback",
        ]
    );
    let control = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/template-ref-outside-instance/LifecycleHookControl.vue.txt"
    );
    assert!(snippets(control).is_empty());
}
