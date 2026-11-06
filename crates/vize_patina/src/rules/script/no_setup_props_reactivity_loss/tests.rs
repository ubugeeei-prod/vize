use super::{DESTRUCTURE, GET_SCRIPT_SETUP, GET_SETUP, NoSetupPropsReactivityLoss};
use crate::diagnostic::Severity;
use crate::rules::script::{ScriptLintResult, ScriptLinter};

const RULE: &str = super::RULE;

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoSetupPropsReactivityLoss));
    linter.lint(source, 0)
}

fn messages(source: &str) -> Vec<std::string::String> {
    lint(source)
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

fn assert_messages(source: &str, expected: &[&str]) {
    assert_eq!(
        messages(source),
        expected
            .iter()
            .map(|message| (*message).to_owned())
            .collect::<Vec<_>>()
    );
}

#[test]
fn reports_destructured_setup_params() {
    assert_messages("export default { setup({ count }) {} }", &[DESTRUCTURE]);
    assert_messages("export default { setup([count]) {} }", &[DESTRUCTURE]);
}

#[test]
fn reports_setup_root_copies() {
    assert_messages(
        "export default { setup(props) { const { count } = props } }",
        &[GET_SETUP],
    );
    assert_messages(
        "export default { setup: (props) => { const { count } = props } }",
        &[GET_SETUP],
    );
    assert_messages(
        "export default { setup(props) { ;({ count } = props) } }",
        &[GET_SETUP],
    );
    assert_messages(
        r#"
        export default { setup(p) { const { count } = p } }
        Vue.component('component', { setup(p) { const { count } = p } })
        defineComponent({
          setup(p) { const { x } = p; const { y } = p },
          _setup(p) { const { count } = p },
        })
        "#,
        &[GET_SETUP, GET_SETUP, GET_SETUP, GET_SETUP],
    );
    assert_messages(
        r#"
        export default {
          setup(p) {
            const foo = p.bar
            const x = p.foo
            const y = p?.bar
            const z = (p?.baz).qux
            const xc = p?.foo?.()
            const yc = (p?.bar)?.()
            const zc = (p?.baz.qux)?.()
            let assigned
            assigned = p.bar
            const { foo: renamed } = p.bar
            member.bar = p.bar
          }
        }
        "#,
        &[GET_SETUP; 10],
    );
    assert_messages(
        r#"
        export default {
          setup: (props) => {
            const count = ref(props.count)
            count = fn(props.count)
          }
        }
        "#,
        &[GET_SETUP, GET_SETUP],
    );
}

#[test]
fn reports_script_setup_copies() {
    assert_messages(
        r#"
        const { count } = defineProps({ count: Number })
        const foo = count
        "#,
        &[GET_SCRIPT_SETUP],
    );
    assert_messages(
        "const [count] = defineProps({ count: Number })",
        &[DESTRUCTURE],
    );
    assert_messages(
        "const { count: { nested } } = defineProps({ count: Object })",
        &[DESTRUCTURE],
    );
    assert_messages(
        r#"
        const props = defineProps({ count: Number })
        const { count } = props
        ;({ count } = props)
        let seen = props.count
        seen = props.count
        const wrapped = ref(props.count)
        wrapped = fn(props.count)
        const newProps = ref({ count: props.count })
        const counts = [props.count]
        const counter = { count: props.count }
        const counters = [{ count: [props.count] }]
        const buildCounter = (count) => ({ count })
        buildCounter(props.count)
        const tern = props.count ? 1 : undefined
        const text = `${props.x}`
        const both = `bar${props.x}bar${props.y}`
        "#,
        &[GET_SCRIPT_SETUP; 14],
    );
    assert_messages(
        r#"
        const props = withDefaults(defineProps({ count: Number }), { count: 1 })
        const { c1 } = props
        const { c2 } = withDefaults(defineProps({ count: Number }), { count: 1 })
        const foo = c2
        "#,
        &[GET_SCRIPT_SETUP, GET_SCRIPT_SETUP],
    );
}

#[test]
fn allows_nested_reads_and_non_vue_objects() {
    let source = r#"
        export default {
          setup(props) {
            watch(() => {
              console.log(props.count)
              const { count } = props
              console.log(count)
            })
            return () => {
              const { count } = props
              return h('div', count)
            }
            const count = computed(() => props.count)
            const props2 = props
            ;({ props } = x)
            const { x } = noProps
            ;({ y } = noProps)
            const z = noProps.z
            const foo = `${noProp.foo}`
            if (props.count) {
              const { count } = props
              const copy = props.count
            }
          },
          _setup({ count }) {},
          watch: { setup({ val }) {} },
        }
        defineComponent({ setup() {} })
        defineComponent({ setup(...args) {} })
        defineComponent({
          setup: (props) => { const count = computed(() => props.count) }
        })
        var noVue = { setup(props) { const { count } = props } }
        Vue.component('test', { el: a = b })
        new Vue({ setup(props) { watch(() => props.count) } })
        const props = defineProps({ count: Number })
        watch(() => { const { count } = props })
        watch(() => { const count = props.count })
        watch(() => { ;({ count } = props) })
        watch(() => { count = props.count })
        const { count } = defineProps({ count: Number })
        const { foo = 1, bar = 'ok' } = defineProps({ foo: Number, bar: String })
        const mirrored = computed(() => props.count)
    "#;
    assert!(messages(source).is_empty(), "{:?}", messages(source));
}

#[test]
fn belongs_to_the_default_presets() {
    let meta = crate::linter::script_rules::builtin_script_rules()
        .into_iter()
        .find(|rule| rule.name == RULE)
        .expect(RULE);
    assert_eq!(meta.default_severity, Severity::Warning);
    assert_eq!(
        meta.presets,
        ["happy-path", "ecosystem", "nuxt", "opinionated"]
    );
}
