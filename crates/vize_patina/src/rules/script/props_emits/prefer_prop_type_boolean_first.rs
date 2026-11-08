//! vue/prefer-prop-type-boolean-first
//!
//! Vue treats a bare boolean attribute as `true` only when `Boolean` is the
//! first constructor in a prop's type list. A later `Boolean`, as in
//! `[String, Boolean]` or `{ type: [String, Boolean] }`, is reported on an
//! Options API `props` object and on runtime `defineProps`. Array-form prop
//! names and a lone `Boolean` stay valid. The check is opt-in.

use oxc_ast::ast::{ArrayExpression, Expression, ObjectExpression, ObjectPropertyKind, Program};
use oxc_span::Span;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::props_source::{collect_all_runtime_props, property_key_name};
use crate::diagnostic::{LintDiagnostic, Severity};

const RULE: &str = "vue/prefer-prop-type-boolean-first";
const MESSAGE: &str = "Type `Boolean` should be at first in prop types.";

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: RULE,
    description: "Enforce `Boolean` comes first in component prop types",
    default_severity: Severity::Warning,
};

/// Report a prop type list whose `Boolean` constructor is not first.
pub struct PreferPropTypeBooleanFirst;

impl ScriptRule for PreferPropTypeBooleanFirst {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        for source in collect_all_runtime_props(program) {
            for prop in source.object_props() {
                if let Some(array) = type_array(prop.value) {
                    report_boolean_not_first(array, offset, result);
                }
            }
        }
    }
}

/// The type-constructor array for a prop value.
///
/// Array shorthand (`foo: [String, Boolean]`) is the value itself. Object
/// shorthand (`foo: { type: [String, Boolean] }`) carries the array on `type`.
fn type_array<'a>(value: &'a Expression<'a>) -> Option<&'a ArrayExpression<'a>> {
    match value {
        Expression::ArrayExpression(array) => Some(array),
        Expression::ObjectExpression(object) => descriptor_type_array(object),
        _ => None,
    }
}

fn descriptor_type_array<'a>(object: &'a ObjectExpression<'a>) -> Option<&'a ArrayExpression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property_key_name(&property.key) != Some("type") {
            continue;
        }
        return match &property.value {
            Expression::ArrayExpression(array) => Some(array),
            _ => None,
        };
    }
    None
}

fn report_boolean_not_first(
    array: &ArrayExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let Some(span) = boolean_not_first(array) else {
        return;
    };
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(LintDiagnostic::warn(RULE, MESSAGE, start, end).with_help(
        "Move `Boolean` to the front of the prop type list, for example `[Boolean, String]`.",
    ));
}

/// Span of the first `Boolean` identifier when it is not the first element.
fn boolean_not_first(array: &ArrayExpression<'_>) -> Option<Span> {
    for (index, element) in array.elements.iter().enumerate() {
        let Some(Expression::Identifier(identifier)) = element.as_expression() else {
            continue;
        };
        if identifier.name.as_str() != "Boolean" {
            continue;
        }
        if index > 0 {
            return Some(identifier.span);
        }
        return None;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{MESSAGE, PreferPropTypeBooleanFirst};
    use crate::diagnostic::Severity;
    use crate::rules::script::ScriptLinter;

    const RULE: &str = super::RULE;

    fn messages(source: &str) -> Vec<std::string::String> {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferPropTypeBooleanFirst));
        linter
            .lint(source, 0)
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect()
    }

    #[test]
    fn reports_boolean_after_another_constructor() {
        let bad = vec![MESSAGE.to_owned()];
        assert_eq!(
            messages("export default { props: { value: [String, Boolean] } }"),
            bad
        );
        assert_eq!(
            messages(
                "export default { props: { value: { type: [String, Boolean], required: true } } }"
            ),
            bad
        );
        assert_eq!(
            messages("export default defineComponent({ props: { value: [Number, Boolean] } })"),
            bad
        );
        assert_eq!(messages("defineProps({ value: [String, Boolean] })"), bad);
        assert_eq!(
            messages("const props = defineProps({ value: { type: [String, Boolean] } })"),
            bad
        );
        assert_eq!(
            messages("export default { props: { value: [String, Boolean, Number] } }"),
            bad
        );
        assert_eq!(
            messages(
                "export default { props: { a: [String, Boolean], b: { type: [Number, Boolean] } } }"
            ),
            vec![MESSAGE.to_owned(), MESSAGE.to_owned()]
        );
        assert_eq!(
            messages(
                "const Child = defineComponent({ props: { value: { type: [String, Boolean] } } })"
            ),
            bad
        );
    }

    #[test]
    fn allows_boolean_first_and_props_without_a_union() {
        for source in [
            "export default { props: { value: [Boolean, String] } }",
            "export default { props: { value: { type: [Boolean, String] } } }",
            "export default { props: { value: Boolean } }",
            "export default { props: { value: { type: Boolean } } }",
            "export default { props: { value: [String, Number] } }",
            "export default { props: { names: ['value'] } }",
            "defineProps({ value: [Boolean, String] })",
            "defineProps({ value: { type: [Boolean, Number] } })",
            "defineProps({ value: Boolean })",
            "defineProps<{ value: string | boolean }>()",
            "const options = { props: { value: [String, Boolean] } }",
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
        assert_eq!(meta.default_severity, Severity::Warning);
        assert!(meta.presets.is_empty());
    }
}
