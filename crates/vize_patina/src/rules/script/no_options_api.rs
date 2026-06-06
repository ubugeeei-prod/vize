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
use oxc_ast::ast::{
    Argument, BindingPattern, ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_span::GetSpan;
use vize_carton::{CompactString, FxHashMap, FxHashSet};

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
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(component_options) = find_component_options(program) else {
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

#[derive(Default)]
struct PetiteVueBindings<'a> {
    create_app_bindings: FxHashSet<&'a str>,
    namespace_bindings: FxHashSet<&'a str>,
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

fn find_component_options<'a>(program: &'a Program<'a>) -> Option<ComponentOptionsMatch> {
    let mut bindings = FxHashMap::default();
    let mut petite_vue = PetiteVueBindings::default();

    for statement in program.body.iter() {
        collect_petite_vue_imports(statement, &mut petite_vue);
    }

    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };

            collect_petite_vue_variable_binding(&declarator.id, init, &mut petite_vue);

            if let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(options) =
                    extract_component_options_from_expression(init, &bindings, &petite_vue)
            {
                bindings.insert(id.name.as_str(), options);
            }
        }
    }

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let Some(options) =
            extract_component_options_from_export(&export.declaration, &bindings, &petite_vue)
        else {
            continue;
        };
        return Some(build_component_options_match(options.object));
    }

    for statement in program.body.iter() {
        let Some(options) =
            extract_create_app_options_from_statement(statement, &bindings, &petite_vue)
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
    petite_vue: &PetiteVueBindings<'a>,
) -> Option<ComponentOptionsRef<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => {
            Some(ComponentOptionsRef { object })
        }
        ExportDefaultDeclarationKind::CallExpression(call) => {
            extract_component_options_from_call(call, bindings, petite_vue)
        }
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            extract_component_options_from_expression(&paren.expression, bindings, petite_vue)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            extract_component_options_from_expression(&ts_as.expression, bindings, petite_vue)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            extract_component_options_from_expression(
                &ts_satisfies.expression,
                bindings,
                petite_vue,
            )
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            extract_component_options_from_expression(&ts_non_null.expression, bindings, petite_vue)
        }
        _ => None,
    }
}

fn extract_component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
    petite_vue: &PetiteVueBindings<'a>,
) -> Option<ComponentOptionsRef<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(ComponentOptionsRef { object }),
        Expression::CallExpression(call) => {
            extract_component_options_from_call(call, bindings, petite_vue)
        }
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            extract_component_options_from_expression(&paren.expression, bindings, petite_vue)
        }
        Expression::TSAsExpression(ts_as) => {
            extract_component_options_from_expression(&ts_as.expression, bindings, petite_vue)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_component_options_from_expression(
                &ts_satisfies.expression,
                bindings,
                petite_vue,
            )
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_component_options_from_expression(&ts_non_null.expression, bindings, petite_vue)
        }
        _ => None,
    }
}

fn extract_component_options_from_call<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
    petite_vue: &PetiteVueBindings<'a>,
) -> Option<ComponentOptionsRef<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }

    let first_arg = call.arguments.first()?;
    extract_component_options_from_argument(first_arg, bindings, petite_vue)
}

fn extract_create_app_options_from_statement<'a>(
    statement: &'a Statement<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
    petite_vue: &PetiteVueBindings<'a>,
) -> Option<ComponentOptionsRef<'a>> {
    match statement {
        Statement::ExpressionStatement(statement) => {
            extract_create_app_options_from_expression(&statement.expression, bindings, petite_vue)
        }
        Statement::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                let Some(init) = declarator.init.as_ref() else {
                    continue;
                };
                if let Some(options) =
                    extract_create_app_options_from_expression(init, bindings, petite_vue)
                {
                    return Some(options);
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_create_app_options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
    petite_vue: &PetiteVueBindings<'a>,
) -> Option<ComponentOptionsRef<'a>> {
    match expression {
        Expression::CallExpression(call) => extract_create_app_options_from_create_app_call(
            call, bindings, petite_vue,
        )
        .or_else(|| extract_create_app_options_from_expression(&call.callee, bindings, petite_vue)),
        Expression::StaticMemberExpression(member) => {
            extract_create_app_options_from_expression(&member.object, bindings, petite_vue)
        }
        Expression::ParenthesizedExpression(paren) => {
            extract_create_app_options_from_expression(&paren.expression, bindings, petite_vue)
        }
        Expression::TSAsExpression(ts_as) => {
            extract_create_app_options_from_expression(&ts_as.expression, bindings, petite_vue)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_create_app_options_from_expression(
                &ts_satisfies.expression,
                bindings,
                petite_vue,
            )
        }
        Expression::TSNonNullExpression(ts_non_null) => extract_create_app_options_from_expression(
            &ts_non_null.expression,
            bindings,
            petite_vue,
        ),
        _ => None,
    }
}

fn extract_create_app_options_from_create_app_call<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
    petite_vue: &PetiteVueBindings<'a>,
) -> Option<ComponentOptionsRef<'a>> {
    if !is_create_app_callee(&call.callee, petite_vue) {
        return None;
    }

    let first_arg = call.arguments.first()?;
    extract_component_options_from_argument(first_arg, bindings, petite_vue)
}

fn is_create_app_callee(callee: &Expression<'_>, petite_vue: &PetiteVueBindings<'_>) -> bool {
    match callee {
        Expression::Identifier(callee) => {
            callee.name.as_str() == "createApp"
                && !petite_vue
                    .create_app_bindings
                    .contains(callee.name.as_str())
        }
        Expression::StaticMemberExpression(member) => {
            member.property.name.as_str() == "createApp"
                && matches!(
                    &member.object,
                    Expression::Identifier(object)
                        if object.name.as_str() == "Vue"
                            && !petite_vue.namespace_bindings.contains(object.name.as_str())
                )
        }
        _ => false,
    }
}

fn extract_component_options_from_argument<'a>(
    argument: &'a Argument<'a>,
    bindings: &FxHashMap<&'a str, ComponentOptionsRef<'a>>,
    petite_vue: &PetiteVueBindings<'a>,
) -> Option<ComponentOptionsRef<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(ComponentOptionsRef { object }),
        Argument::CallExpression(call) => {
            extract_component_options_from_call(call, bindings, petite_vue)
        }
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Argument::ParenthesizedExpression(paren) => {
            extract_component_options_from_expression(&paren.expression, bindings, petite_vue)
        }
        Argument::TSAsExpression(ts_as) => {
            extract_component_options_from_expression(&ts_as.expression, bindings, petite_vue)
        }
        Argument::TSSatisfiesExpression(ts_satisfies) => extract_component_options_from_expression(
            &ts_satisfies.expression,
            bindings,
            petite_vue,
        ),
        Argument::TSNonNullExpression(ts_non_null) => {
            extract_component_options_from_expression(&ts_non_null.expression, bindings, petite_vue)
        }
        _ => None,
    }
}

fn collect_petite_vue_imports<'a>(
    statement: &'a Statement<'a>,
    petite_vue: &mut PetiteVueBindings<'a>,
) {
    let Statement::ImportDeclaration(import) = statement else {
        return;
    };
    if !is_petite_vue_module(import.source.value.as_str()) {
        return;
    }

    let Some(specifiers) = &import.specifiers else {
        return;
    };

    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if specifier.imported.name().as_str() == "createApp" =>
            {
                petite_vue
                    .create_app_bindings
                    .insert(specifier.local.name.as_str());
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                petite_vue
                    .namespace_bindings
                    .insert(specifier.local.name.as_str());
            }
            _ => {}
        }
    }
}

fn collect_petite_vue_variable_binding<'a>(
    pattern: &'a BindingPattern<'a>,
    init: &'a Expression<'a>,
    petite_vue: &mut PetiteVueBindings<'a>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier)
            if is_petite_vue_namespace_expression(init, petite_vue) =>
        {
            petite_vue
                .namespace_bindings
                .insert(identifier.name.as_str());
        }
        BindingPattern::ObjectPattern(pattern)
            if is_petite_vue_namespace_expression(init, petite_vue) =>
        {
            for property in &pattern.properties {
                if property_key_name(&property.key) != Some("createApp") {
                    continue;
                }
                if let Some(local_name) = binding_pattern_name(&property.value) {
                    petite_vue.create_app_bindings.insert(local_name);
                }
            }
        }
        _ => {}
    }
}

fn is_petite_vue_namespace_expression(
    expression: &Expression<'_>,
    petite_vue: &PetiteVueBindings<'_>,
) -> bool {
    match expression {
        Expression::Identifier(identifier) => {
            identifier.name.as_str() == "PetiteVue"
                || petite_vue
                    .namespace_bindings
                    .contains(identifier.name.as_str())
        }
        Expression::ParenthesizedExpression(paren) => {
            is_petite_vue_namespace_expression(&paren.expression, petite_vue)
        }
        Expression::TSAsExpression(ts_as) => {
            is_petite_vue_namespace_expression(&ts_as.expression, petite_vue)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_petite_vue_namespace_expression(&ts_satisfies.expression, petite_vue)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_petite_vue_namespace_expression(&ts_non_null.expression, petite_vue)
        }
        _ => false,
    }
}

fn is_petite_vue_module(value: &str) -> bool {
    value == "petite-vue" || value.starts_with("petite-vue/") || value.contains("/petite-vue")
}

fn binding_pattern_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        BindingPattern::AssignmentPattern(assignment) => binding_pattern_name(&assignment.left),
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
        insta::assert_debug_snapshot!(result.diagnostics);
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
        insta::assert_debug_snapshot!(result.diagnostics);
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
        insta::assert_debug_snapshot!(result.diagnostics);
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
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_cdn_create_app_options() {
        let source = r##"
Vue.createApp({
  data() {
    return { count: 0 }
  }
}).mount("#app")
"##;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_destructured_create_app_options() {
        let source = r##"
const { createApp } = Vue
const options = {
  methods: {
    increment() {}
  }
}

createApp(options).mount("#app")
"##;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_petite_vue_global_create_app_is_not_options_api() {
        let source = r##"
PetiteVue.createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_petite_vue_imported_create_app_is_not_options_api() {
        let source = r##"
import { createApp } from 'petite-vue'

createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_petite_vue_cdn_imported_create_app_is_not_options_api() {
        let source = r##"
import { createApp as createPetiteApp } from 'https://unpkg.com/petite-vue?module'

createPetiteApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_petite_vue_destructured_create_app_is_not_options_api() {
        let source = r##"
const { createApp } = PetiteVue

createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_petite_vue_namespace_named_vue_is_not_options_api() {
        let source = r##"
import * as Vue from 'petite-vue'

Vue.createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
        let rule = NoOptionsApi;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
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
