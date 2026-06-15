//! script/no-unstable-nested-components
//!
//! Disallow component definition calls inside Options API `setup` and `render`.
//! A component definition created there gets a fresh identity for each setup or
//! render execution; define child components at module scope instead.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    CallExpression, Expression, FunctionBody, ObjectProperty, Program, PropertyKey,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_object_property},
};
use oxc_span::{GetSpan, Span};
use vize_croquis::script_parser::collect_options_descriptor;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-unstable-nested-components",
    description: "Disallow component definitions inside setup or render functions",
    default_severity: Severity::Warning,
};

pub struct NoUnstableNestedComponents;

impl ScriptRule for NoUnstableNestedComponents {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
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
        let Some(descriptor) = collect_options_descriptor(program) else {
            return;
        };
        let target_keys: Vec<(u32, u32)> = descriptor
            .option_keys
            .iter()
            .filter(|key| matches!(key.name.as_str(), "setup" | "render"))
            .map(|key| (key.start, key.end))
            .collect();
        if target_keys.is_empty() {
            return;
        }
        OptionsMethodVisitor {
            target_keys: &target_keys,
            offset,
            result,
        }
        .visit_program(program);
    }
}

struct OptionsMethodVisitor<'rule> {
    target_keys: &'rule [(u32, u32)],
    offset: usize,
    result: &'rule mut ScriptLintResult,
}

impl<'a> Visit<'a> for OptionsMethodVisitor<'_> {
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        if key_matches(it, self.target_keys)
            && let Some(body) = property_function_body(it)
        {
            NestedComponentVisitor {
                offset: self.offset,
                result: self.result,
            }
            .visit_function_body(body);
        }
        walk_object_property(self, it);
    }
}

struct NestedComponentVisitor<'rule> {
    offset: usize,
    result: &'rule mut ScriptLintResult,
}

impl<'a> Visit<'a> for NestedComponentVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_component_definition_callee(&it.callee) {
            self.report(it.span);
        }
        walk_call_expression(self, it);
    }
}

impl NestedComponentVisitor<'_> {
    fn report(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result.add_diagnostic(
            LintDiagnostic::warn(
                META.name,
                "Do not define a component inside setup or render.",
                start,
                end,
            )
            .with_label("nested component definition", start, end)
            .with_help(
                "Move the component definition to module scope so the child component keeps a \
                 stable identity across setup/render executions.",
            ),
        );
    }
}

fn key_matches(property: &ObjectProperty<'_>, targets: &[(u32, u32)]) -> bool {
    if property.computed || !matches!(property_key_name(&property.key), Some("setup" | "render")) {
        return false;
    }
    let span = property.key.span();
    targets
        .iter()
        .any(|(start, end)| span.start == *start && span.end == *end)
}

fn property_function_body<'a>(property: &'a ObjectProperty<'a>) -> Option<&'a FunctionBody<'a>> {
    match &property.value {
        Expression::FunctionExpression(function) => function.body.as_deref(),
        Expression::ArrowFunctionExpression(arrow) => Some(&arrow.body),
        _ => None,
    }
}

fn is_component_definition_callee(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(identifier) => {
            matches!(
                identifier.name.as_str(),
                "defineComponent" | "defineAsyncComponent"
            )
        }
        Expression::StaticMemberExpression(member) => {
            matches!(
                member.property.name.as_str(),
                "defineComponent" | "defineAsyncComponent"
            )
        }
        Expression::ParenthesizedExpression(paren) => {
            is_component_definition_callee(&paren.expression)
        }
        Expression::TSAsExpression(ts_as) => is_component_definition_callee(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts) => is_component_definition_callee(&ts.expression),
        Expression::TSNonNullExpression(ts) => is_component_definition_callee(&ts.expression),
        _ => false,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::NoUnstableNestedComponents;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoUnstableNestedComponents));
        linter.lint(source, 0)
    }

    #[test]
    fn reports_define_component_inside_setup() {
        let result = lint(
            r#"
export default {
  setup() {
    const Child = defineComponent({ name: 'Child' })
    return { Child }
  }
}
"#,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_member_define_component_inside_render() {
        let result = lint(
            r#"
export default {
  render() {
    const Child = Vue.defineComponent({ name: 'Child' })
    return h(Child)
  }
}
"#,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_arrow_expression_body_inside_setup() {
        let result = lint(
            r#"
export default {
  setup: () => defineComponent({ name: 'Child' })
}
"#,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_define_async_component_inside_setup() {
        let result = lint(
            r#"
export default {
  setup() {
    const Child = defineAsyncComponent(() => import('./Child.vue'))
    return { Child }
  }
}
"#,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_module_scope_child_component() {
        let result = lint(
            r#"
const Child = defineComponent({ name: 'Child' })
export default {
  setup() {
    return { Child }
  }
}
"#,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_factory_outside_component_execution() {
        let result = lint(
            r#"
function makeChild() {
  return defineComponent({ name: 'Child' })
}
export default {
  setup() {
    return {}
  }
}
"#,
        );
        assert_eq!(result.warning_count, 0);
    }
}
