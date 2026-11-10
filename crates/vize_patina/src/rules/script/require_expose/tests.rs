use super::{MESSAGE, RequireExpose};
use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule};

const RULE: &str = super::RULE;

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequireExpose));
    linter.lint(source, 0)
}

fn messages(source: &str) -> Vec<std::string::String> {
    lint(source)
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

fn assert_hits(source: &str, count: usize) {
    assert_eq!(messages(source), vec![MESSAGE.to_owned(); count]);
}

fn assert_clean(source: &str) {
    assert!(messages(source).is_empty(), "{:?}", messages(source));
}

#[test]
fn reports_components_without_expose() {
    assert_hits(
        r#"
        defineComponent({ methods: { foo() {} } })
        createApp({})
        defineNuxtComponent({})
        Vue.component('page', {})
        Vue.extend({})
        new Vue({})
        app.component('page', {})
        defineComponent({ setup(_, [expose]) { expose() } })
        defineComponent({ setup(_, { expose }) { function inner(expose) { expose() } } })
        defineComponent({ setup(_, ctx) { function inner(ctx) { ctx.expose() } } })
        defineComponent({ setup() { function inner() { return function () {} } return 1 } })
        defineComponent({ setup(render) { return render } })
        "#,
        12,
    );
}

#[test]
fn bare_default_objects_are_components_only_in_sfc_and_jsx_hosts() {
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    let source = "export default { methods: { foo() {}, bar() {} } }";

    for filename in ["messages.js", "messages.ts"] {
        let result = linter.lint_script(source, filename);
        assert!(
            result.diagnostics.is_empty(),
            "{filename}: {:?}",
            result.diagnostics
        );
    }
    let router_options = r#"
        import type { RouterConfig } from "@nuxt/schema"
        export default {
            scrollBehavior(_to, _from, savedPosition) {
                return savedPosition ?? { top: 0 }
            },
        } satisfies RouterConfig
    "#;
    let result = linter.lint_script(router_options, "router.options.ts");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

    for filename in ["Component.jsx", "Component.tsx"] {
        let result = linter.lint_script(source, filename);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "{filename}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.diagnostics[0].message, MESSAGE);
    }

    let result = linter.lint_sfc("<script>export default {}</script>", "Component.vue");
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].message, MESSAGE);
}

#[test]
fn create_app_checks_the_component_not_root_props() {
    assert_clean(
        r##"
        import { createApp } from "vue"
        createApp(App, { id: "main" }).mount("#app")
        "##,
    );
    assert_hits(
        r#"
        import { createApp } from "vue"
        createApp({ render() {} }, { id: "main" })
        "#,
        1,
    );
}

#[test]
fn allows_expose_render_and_mixins() {
    assert_clean(
        r#"
        export default {
          expose: ['foo'],
          methods: { foo() {}, bar() {} }
        }
        "#,
    );
    assert_clean("export default { expose: [], methods: { foo() {}, bar() {} } }");
    assert_clean("export default { expose() { return 1 } }");
    assert_clean(
        r#"
        defineComponent({
          setup(_, { expose }) {
            expose({ foo() {} })
            return { bar() {} }
          }
        })
        defineComponent({
          setup(_, { expose }) {
            expose()
            return { foo() {}, bar() {} }
          }
        })
        defineComponent({
          setup(_, { expose: open }) {
            open()
          }
        })
        defineComponent({
          setup(_, ctx) {
            ctx.expose({ foo() {} })
            return { bar() {} }
          }
        })
        defineComponent({
          setup(_, ctx) {
            ctx?.expose()
          }
        })
        defineComponent({ setup() { return () => h('div') } })
        defineComponent({ setup() { function foo() {} return foo } })
        defineComponent({ setup() { const foo = function () {} return foo } })
        defineComponent({ setup() { const foo = () => {} return foo } })
        defineComponent({ setup: () => function () {} })
        defineComponent({
          setup() {
            if (false) { return () => h('div') }
          }
        })
        Vue.mixin({ methods: { foo() {} } })
        app.mixin({ methods: { foo() {} } })
        app.extend({})
        var noVue = { methods: { foo() {} } }
        "#,
    );
    assert_clean(
        r#"
        import { defineComponent } from "react"
        defineComponent({})
        "#,
    );
    assert_clean(
        r#"
        function defineComponent(options) { return options }
        defineComponent({})
        "#,
    );
    assert_clean(
        r#"
        const { defineComponent } = require("react")
        defineComponent({})
        "#,
    );
}

#[test]
fn stays_opt_in() {
    let meta = crate::linter::script_rules::builtin_script_rules()
        .into_iter()
        .find(|rule| rule.name == RULE)
        .expect(RULE);
    assert_eq!(meta.default_severity, Severity::Warning);
    assert!(meta.presets.is_empty());
    assert!(!RequireExpose.runs_on_script_setup());
}

#[test]
fn plain_script_default_export_is_ignored_when_script_setup_is_present() {
    let result = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(
            r#"<script lang="ts">export default {}</script>
<script setup lang="ts">const value = 1</script>
<template>{{ value }}</template>"#,
            "Component.vue",
        );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}
