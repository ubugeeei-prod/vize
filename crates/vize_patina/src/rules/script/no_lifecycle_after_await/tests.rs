use crate::linter::Linter;

const RULE: &str = "script/no-lifecycle-after-await";

fn findings(source: &str) -> Vec<&str> {
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    let result = linter.lint_sfc(source, "Test.vue");
    let mut found = Vec::new();
    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.rule_name, RULE);
        let snippet = source.get(diagnostic.start as usize..diagnostic.end as usize);
        found.push(snippet.unwrap_or_default());
    }
    found
}

fn options(setup: &str) -> String {
    format!(
        "<script>\nimport {{ onMounted, onUnmounted, defineComponent }} from 'vue'\nexport default {{\n  async setup() {{\n{setup}\n  }},\n}}\n</script>\n"
    )
}

#[test]
fn hooks_before_the_first_await_are_allowed() {
    assert!(findings(&options("onMounted(() => {})\nawait load()")).is_empty());
}

#[test]
fn hooks_after_await_are_reported() {
    let source = options("await load()\nonMounted(() => {})\nif (ok) { onUnmounted(stop) }");
    assert_eq!(
        findings(&source),
        ["onMounted(() => {})", "onUnmounted(stop)"]
    );
}

#[test]
fn awaits_and_hooks_in_nested_functions_do_not_count() {
    let source =
        options("const run = async () => { await load(); onMounted(go) }\nrun()\nonMounted(go)");
    assert!(findings(&source).is_empty());
}

#[test]
fn an_explicit_target_instance_is_allowed() {
    assert!(findings(&options("await load()\nonMounted(go, instance)")).is_empty());
}

#[test]
fn hook_arguments_evaluated_before_await_are_allowed() {
    // The call starts before its awaited argument, as upstream sees it.
    assert!(findings(&options("onMounted(await load())")).is_empty());
}

#[test]
fn define_component_and_namespace_imports_are_checked() {
    let source = "<script lang=\"ts\">\nimport * as Vue from 'vue'\nexport default Vue.defineComponent({})\nconst Inner = defineComponent({\n  async setup() {\n    await load()\n    Vue.onMounted(go)\n  },\n})\n</script>\n";
    assert_eq!(findings(source), ["Vue.onMounted(go)"]);
}

#[test]
fn script_setup_and_unimported_hooks_are_ignored() {
    let setup =
        "<script setup>\nimport { onMounted } from 'vue'\nawait load()\nonMounted(go)\n</script>\n";
    assert!(findings(setup).is_empty());
    let local = "<script>\nexport default {\n  async setup() {\n    await load()\n    onMounted(go)\n  },\n}\n</script>\n";
    assert!(findings(local).is_empty());
}
