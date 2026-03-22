//! script/no-options-api
//!
//! Disallow Options API patterns in Vapor mode.
//!
//! Vue Vapor mode only supports Composition API. Options API patterns like
//! `data()`, `computed`, `methods`, `watch` as object properties are not
//! supported.
//!
//! Based on Vue 3.6.0-beta.1 release notes:
//! <https://github.com/vuejs/core/releases/tag/v3.6.0-beta.1>
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   data() { return { count: 0 } },
//!   computed: { doubled() { return this.count * 2 } },
//!   methods: { increment() { this.count++ } },
//!   watch: { count(val) { console.log(val) } }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! import { ref, computed, watch } from 'vue'
//! const count = ref(0)
//! const doubled = computed(() => count.value * 2)
//! const increment = () => count.value++
//! watch(count, (val) => console.log(val))
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, BindingPattern, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-options-api",
    description: "Disallow Options API patterns in Vapor mode",
    default_severity: Severity::Error,
};

/// Disallow Options API patterns
pub struct NoOptionsApi;

impl ScriptRule for NoOptionsApi {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
        let Some(component_options) = find_component_options(source) else {
            return;
        };

        let start = offset + component_options.start as usize;
        let end = offset + component_options.end as usize;
        let mut diagnostic = LintDiagnostic::error(
            META.name,
            "Options API component declarations are not supported",
            start as u32,
            end as u32,
        )
        .with_help(
            "Use <script setup> with Composition API. Move props/emits to defineProps()/defineEmits(), lifecycle options to onMounted()/onUnmounted(), and component metadata to defineOptions() when needed.",
        );

        if component_options.labels.is_empty() {
            diagnostic =
                diagnostic.with_label("Component options object", start as u32, end as u32);
        } else {
            for label in component_options.labels {
                diagnostic = diagnostic.with_label(
                    label.message,
                    offset as u32 + label.start,
                    offset as u32 + label.end,
                );
            }
        }

        result.add_diagnostic(diagnostic);
    }
}

#[derive(Clone, Copy)]
struct ComponentOptionsRef<'a> {
    object: &'a ObjectExpression<'a>,
}

struct ComponentOptionsMatch {
    start: u32,
    end: u32,
    labels: Vec<OptionLabel>,
}

struct OptionLabel {
    message: CompactString,
    start: u32,
    end: u32,
}

fn find_component_options(source: &str) -> Option<ComponentOptionsMatch> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("component.ts").unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return None;
    }

    let mut bindings = FxHashMap::default();
    for statement in parsed.program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let Some(options) = extract_component_options_from_expression(init, &bindings) {
                bindings.insert(id.name.as_str(), options);
            }
        }
    }

    for statement in parsed.program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let Some(options) = extract_component_options_from_export(&export.declaration, &bindings)
        else {
            continue;
        };
        return Some(build_component_options_match(options.object));
    }

    None
}

fn extract_component_options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => {
            Some(ComponentOptionsRef { object })
        }
        ExportDefaultDeclarationKind::CallExpression(call) => {
            extract_component_options_from_call(call, bindings)
        }
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            extract_component_options_from_expression(&paren.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            extract_component_options_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            extract_component_options_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            extract_component_options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn extract_component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(ComponentOptionsRef { object }),
        Expression::CallExpression(call) => extract_component_options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            extract_component_options_from_expression(&paren.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => {
            extract_component_options_from_expression(&ts_as.expression, bindings)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_component_options_from_expression(&ts_satisfies.expression, bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_component_options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn extract_component_options_from_call<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }

    let first_arg = call.arguments.first()?;
    extract_component_options_from_argument(first_arg, bindings)
}

fn extract_component_options_from_argument<'a>(
    argument: &'a Argument<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
) -> Option<ComponentOptionsRef<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(ComponentOptionsRef { object }),
        Argument::CallExpression(call) => extract_component_options_from_call(call, bindings),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Argument::ParenthesizedExpression(paren) => {
            extract_component_options_from_expression(&paren.expression, bindings)
        }
        Argument::TSAsExpression(ts_as) => {
            extract_component_options_from_expression(&ts_as.expression, bindings)
        }
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            extract_component_options_from_expression(&ts_satisfies.expression, bindings)
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            extract_component_options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn build_component_options_match(object: &ObjectExpression<'_>) -> ComponentOptionsMatch {
    let mut labels = Vec::new();

    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(name) = property_key_name(&property.key) else {
            continue;
        };

        labels.push(OptionLabel {
            message: option_label(name),
            start: property.key.span().start,
            end: property.key.span().end,
        });
    }

    ComponentOptionsMatch {
        start: object.span.start,
        end: object.span.end,
        labels,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn option_label(name: &str) -> CompactString {
    match name {
        "data" => "data() option (use ref()/reactive())".into(),
        "computed" => "computed option (use computed())".into(),
        "methods" => "methods option (use plain functions)".into(),
        "watch" => "watch option (use watch())".into(),
        "props" => "props option (use defineProps())".into(),
        "emits" => "emits option (use defineEmits())".into(),
        "setup" => "setup() option (use <script setup>)".into(),
        "created" | "beforeCreate" | "beforeMount" | "mounted" | "beforeUpdate" | "updated"
        | "beforeUnmount" | "unmounted" => "lifecycle option (use Composition API hooks)".into(),
        _ => {
            let mut label = CompactString::with_capacity(name.len() + 19);
            label.push_str("component option '");
            label.push_str(name);
            label.push('\'');
            label
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NoOptionsApi, ScriptLintResult, ScriptRule};

    #[test]
    fn test_valid_composition_api() {
        let source = r#"
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
"#;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_data_option() {
        let source = r#"
export default {
  data() {
    return { count: 0 }
  }
}
"#;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "script/no-options-api");
        assert!(result.diagnostics[0]
            .labels
            .iter()
            .any(|label| label.message.contains("data()")));
    }

    #[test]
    fn test_invalid_define_component_props_option() {
        let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    count: Number
  }
})
"#;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0]
            .labels
            .iter()
            .any(|label| label.message.contains("defineProps")));
    }

    #[test]
    fn test_invalid_identifier_export() {
        let source = r#"
const component = {
  methods: {
    increment() { this.count++ }
  }
}

export default component
"#;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0]
            .labels
            .iter()
            .any(|label| label.message.contains("methods")));
    }

    #[test]
    fn test_component_metadata_only_still_errors() {
        let source = r#"
export default {
  name: 'CounterButton',
  inheritAttrs: false
}
"#;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0]
            .labels
            .iter()
            .any(|label| label.message.contains("component option 'name'")));
    }

    #[test]
    fn test_no_export_default_skip() {
        let source = r#"
const computed = { foo: 'bar' }
"#;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }
}
