use super::{MESSAGE, RequireExpose};
use crate::diagnostic::Severity;
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
    assert_hits("export default { methods: { foo() {}, bar() {} } }", 1);
    assert_hits(
        "export default { setup(_, ctx) { return { foo() {}, bar() {} } } }",
        1,
    );
    assert_hits(
        "export default { props: ['x'], methods: { foo() {}, bar() {} } }",
        1,
    );
    assert_hits("export default {\n}", 1);
    assert_hits("export default {}", 1);
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
