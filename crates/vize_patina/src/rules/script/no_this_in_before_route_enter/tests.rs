use super::{MESSAGE, NoThisInBeforeRouteEnter};
use crate::diagnostic::Severity;
use crate::rules::script::{ScriptLintResult, ScriptLinter};

const RULE: &str = super::RULE;

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoThisInBeforeRouteEnter));
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

#[test]
fn reports_this_in_before_route_enter() {
    assert_hits("export default { beforeRouteEnter() { this.xxx() } }", 1);
    assert_hits(
        "export default { beforeRouteEnter: function () { this.method() } }",
        1,
    );
    assert_hits(
        "export default { beforeRouteEnter() { this.attr = this.method() } }",
        2,
    );
    assert_hits(
        "export default { beforeRouteEnter() { if (this.method()) {} } }",
        1,
    );
    assert_hits(
        "export default { beforeRouteEnter: function () { if (true) { this.method() } } }",
        1,
    );
    assert_hits(
        r#"
        export default {
          beforeRouteEnter(to, from, next) {
            next(vm => { this.load(vm) })
            const later = () => this.ready
          }
        }
        "#,
        2,
    );
    assert_hits(
        r#"
        Vue.component('page', { beforeRouteEnter() { this.boot() } })
        defineComponent({ beforeRouteEnter() { this.boot() } })
        new Vue({ beforeRouteEnter() { this.boot() } })
        "#,
        3,
    );
}

#[test]
fn allows_other_hooks_and_nested_functions() {
    let source = r#"
        export default {
          beforeRouteEnter(to, from, next) {
            const variable = 42
            someFunction(42)
            next(vm => { vm.load() })
            next(function (vm) { this.load(vm) })
            function inner() { return this.value }
          },
          data() { return this.greeting },
          created() { this.load() },
          beforeRouteUpdate() { this.refresh() },
          beforeRouteLeave() { this.save() },
          watch: { beforeRouteEnter() { this.nested() } },
        }
        defineComponent({
          beforeRouteEnter: () => { this.arrowProperty },
        })
        var noVue = { beforeRouteEnter() { this.outside() } }
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
