//! vue/valid-model-definition
//!
//! Vue 2's `model` option only accepts the keys `prop` and `event`. Any other
//! static key is reported. The values of those keys are not checked: an empty
//! `prop` string is still a valid key. The rule is Vue 2-only and stays opt-in.

use super::component_options::{component_definitions, property_key_name};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program};
use oxc_span::Span;

use vize_carton::{String, cstr};

const RULE: &str = "vue/valid-model-definition";

pub(super) fn message(name: &str) -> String {
    cstr!("Invalid key '{name}' in model option.")
}

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: RULE,
    description: "Require valid keys in the model option",
    default_severity: Severity::Error,
};

/// Report keys other than `prop` and `event` on a component `model` option.
pub struct ValidModelDefinition;

impl ScriptRule for ValidModelDefinition {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !source.contains("model") {
            return;
        }
        for options in component_definitions(program) {
            check_options(options, offset, result);
        }
    }
}

fn check_options(options: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property_key_name(&property.key) != Some("model") {
            continue;
        }
        let Expression::ObjectExpression(model) = property.value.get_inner_expression() else {
            continue;
        };
        check_model(model, offset, result);
    }
}

fn check_model(model: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &model.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(name) = property_key_name(&property.key) else {
            continue;
        };
        if name == "prop" || name == "event" {
            continue;
        }
        report(name, property.span, offset, result);
    }
}

fn report(name: &str, span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(LintDiagnostic::error(RULE, message(name), start, end));
}

#[cfg(test)]
mod tests {
    use super::{ValidModelDefinition, message};
    use crate::diagnostic::Severity;
    use crate::rules::script::ScriptLinter;

    const RULE: &str = super::RULE;

    fn messages(source: &str) -> Vec<std::string::String> {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidModelDefinition));
        linter
            .lint(source, 0)
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect()
    }

    #[test]
    fn reports_keys_other_than_prop_and_event() {
        assert_eq!(
            messages("export default { model: { prop: 'value', events: 'input' } }"),
            vec![message("events")]
        );
        assert_eq!(
            messages("export default { model: { props: 'checked', event: 'change' } }"),
            vec![message("props")]
        );
        assert_eq!(
            messages("defineOptions({ model: { value: 'checked' } })"),
            vec![message("value")]
        );
        assert_eq!(
            messages(
                "export default defineComponent({ model: { prop: 'value', event: 'input', extra: true } })"
            ),
            vec![message("extra")]
        );
        assert_eq!(
            messages("Vue.component('name', { model: { foo: 1, bar: 2 } })"),
            vec![message("foo"), message("bar")]
        );
        assert_eq!(
            messages(r#"export default { model: { "events": "input" } }"#),
            vec![message("events")]
        );
    }

    #[test]
    fn allows_prop_and_event_keys() {
        for source in [
            "export default { model: { prop: 'value', event: 'input' } }",
            "export default { model: { prop: '' } }",
            "export default { model: { prop: 'not-an-identifier', event: 'also not' } }",
            "export default { model: { event: 'change' } }",
            "export default { model: modelOption }",
            "export default { props: { model: { foo: 1 } } }",
            "export default { mixins: [{ model: { foo: 1 } }] }",
            "const options = { model: { foo: 1 } }",
            "export default { props: ['modelValue'], emits: ['update:modelValue'] }",
        ] {
            assert!(messages(source).is_empty(), "{source}");
        }
    }

    #[test]
    fn stays_opt_in() {
        let meta = crate::linter::script_rules::builtin_script_rules()
            .into_iter()
            .find(|rule| rule.name == RULE)
            .expect(RULE);
        assert_eq!(meta.default_severity, Severity::Error);
        assert!(meta.presets.is_empty());
    }
}
