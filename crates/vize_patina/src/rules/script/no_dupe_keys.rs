//! script/no-dupe-keys
//!
//! Disallow duplicate keys across Options API groups.
//!
//! A key declared as a `prop` must not be re-declared in `data`, `computed`,
//! `methods`, `setup`, or `inject` (and vice versa). All of these groups expose
//! their members on the component instance via `this`, so a duplicate name
//! silently shadows one declaration with another and is almost always a bug.
//!
//! This is a port of [`vue/no-dupe-keys`](https://eslint.vuejs.org/rules/no-dupe-keys.html)
//! from eslint-plugin-vue, covering the `props`, `data`, `computed`, `methods`,
//! `setup`, and `inject` groups.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: ['foo'],
//!   data() {
//!     return { foo: 1 } // duplicate of prop `foo`
//!   },
//!   computed: {
//!     bar() { return 2 }
//!   },
//!   methods: {
//!     bar() {} // duplicate of computed `bar`
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: ['foo'],
//!   data() {
//!     return { bar: 1 }
//!   },
//!   computed: {
//!     baz() { return 2 }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, ExportDefaultDeclarationKind, Expression, Function, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_span::GetSpan;
use vize_carton::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-dupe-keys",
    description: "Disallow duplicate keys across Options API props/data/computed/methods/setup/inject",
    default_severity: Severity::Error,
};

/// Disallow duplicate keys across Options API groups.
pub struct NoDupeKeys;

impl ScriptRule for NoDupeKeys {
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
        let Some(options) = find_component_options(program) else {
            return;
        };
        check_options_object(options, offset, result);
    }
}

/// The Options API group a key was declared in. Stored alongside the first span
/// so a later duplicate can point back at the original declaration.
#[derive(Clone, Copy)]
struct KeyOrigin {
    group: &'static str,
    start: u32,
    end: u32,
}

fn check_options_object(
    object: &ObjectExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    // First declaration wins; subsequent declarations of the same key in any
    // group are reported as duplicates pointing back at the original.
    let mut seen: FxHashMap<CompactString, KeyOrigin> = FxHashMap::default();

    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(group) = property_key_name(&property.key).and_then(option_group) else {
            continue;
        };

        for member in collect_group_member_keys(group, &property.value) {
            record_key(&mut seen, group.label, member, offset, result);
        }
    }
}

fn record_key(
    seen: &mut FxHashMap<CompactString, KeyOrigin>,
    group: &'static str,
    member: MemberKey,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let start = offset as u32 + member.start;
    let end = offset as u32 + member.end;

    if let Some(origin) = seen.get(&member.name) {
        let mut message = CompactString::with_capacity(member.name.len() + 48);
        message.push_str("Duplicated key '");
        message.push_str(&member.name);
        message.push('\'');

        let diagnostic = LintDiagnostic::error(META.name, message, start, end)
            .with_label(group_label(group), start, end)
            .with_label(
                first_declaration_label(origin.group),
                origin.start,
                origin.end,
            )
            .with_help(
                "props, data, computed, methods, setup, and inject share the component \
                 instance namespace; give each member a unique name.",
            );
        result.add_diagnostic(diagnostic);
        return;
    }

    seen.insert(member.name, KeyOrigin { group, start, end });
}

fn group_label(group: &'static str) -> CompactString {
    let mut label = CompactString::with_capacity(group.len() + 24);
    label.push_str("declared again in ");
    label.push_str(group);
    label
}

fn first_declaration_label(group: &'static str) -> CompactString {
    let mut label = CompactString::with_capacity(group.len() + 24);
    label.push_str("first declared in ");
    label.push_str(group);
    label
}

/// A single collected member name and the span it should be reported at.
struct MemberKey {
    name: CompactString,
    start: u32,
    end: u32,
}

#[derive(Clone, Copy)]
struct OptionGroup {
    label: &'static str,
    kind: GroupKind,
}

#[derive(Clone, Copy)]
enum GroupKind {
    /// `props`: array of strings or object of declarations.
    Props,
    /// `inject`: array of strings or object of declarations.
    Inject,
    /// `computed` / `methods`: object of declarations.
    Object,
    /// `data` / `setup`: a function returning an object literal.
    ReturnObject,
}

fn option_group(name: &str) -> Option<OptionGroup> {
    let (label, kind) = match name {
        "props" => ("props", GroupKind::Props),
        "inject" => ("inject", GroupKind::Inject),
        "computed" => ("computed", GroupKind::Object),
        "methods" => ("methods", GroupKind::Object),
        "data" => ("data", GroupKind::ReturnObject),
        "setup" => ("setup", GroupKind::ReturnObject),
        _ => return None,
    };
    Some(OptionGroup { label, kind })
}

fn collect_group_member_keys(group: OptionGroup, value: &Expression<'_>) -> Vec<MemberKey> {
    match group.kind {
        GroupKind::Props | GroupKind::Inject => collect_array_or_object_keys(value),
        GroupKind::Object => collect_object_keys(value),
        GroupKind::ReturnObject => collect_returned_object_keys(value),
    }
}

/// `props`/`inject` accept either `['a', 'b']` or `{ a: ..., b: ... }`.
fn collect_array_or_object_keys(value: &Expression<'_>) -> Vec<MemberKey> {
    match value {
        Expression::ArrayExpression(array) => {
            let mut keys = Vec::new();
            for element in &array.elements {
                if let Some(string) = element.as_expression().and_then(string_literal) {
                    keys.push(MemberKey {
                        name: string.value.as_str().into(),
                        start: string.span.start,
                        end: string.span.end,
                    });
                }
            }
            keys
        }
        _ => collect_object_keys(value),
    }
}

fn collect_object_keys(value: &Expression<'_>) -> Vec<MemberKey> {
    let Expression::ObjectExpression(object) = value else {
        return Vec::new();
    };
    object_property_keys(object)
}

/// `data`/`setup` are functions whose returned object literal declares members.
fn collect_returned_object_keys<'a>(value: &'a Expression<'a>) -> Vec<MemberKey> {
    let object = match value {
        Expression::FunctionExpression(function) => function_return_object(function),
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                // `data: () => ({ ... })`
                arrow
                    .body
                    .statements
                    .first()
                    .and_then(|statement| match statement {
                        Statement::ExpressionStatement(expr) => {
                            as_object(unwrap_parenthesized(&expr.expression))
                        }
                        _ => None,
                    })
            } else {
                find_returned_object(&arrow.body.statements)
            }
        }
        _ => None,
    };
    object.map(object_property_keys).unwrap_or_default()
}

fn function_return_object<'a>(function: &'a Function<'a>) -> Option<&'a ObjectExpression<'a>> {
    let body = function.body.as_ref()?;
    find_returned_object(&body.statements)
}

fn find_returned_object<'a>(statements: &'a [Statement<'a>]) -> Option<&'a ObjectExpression<'a>> {
    for statement in statements {
        if let Statement::ReturnStatement(ret) = statement
            && let Some(argument) = ret.argument.as_ref()
        {
            return as_object(unwrap_parenthesized(argument));
        }
    }
    None
}

fn object_property_keys(object: &ObjectExpression<'_>) -> Vec<MemberKey> {
    let mut keys = Vec::new();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed {
                    continue;
                }
                if let Some(name) = property_key_name(&property.key) {
                    keys.push(MemberKey {
                        name: name.into(),
                        start: property.key.span().start,
                        end: property.key.span().end,
                    });
                }
            }
            ObjectPropertyKind::SpreadProperty(_) => {}
        }
    }
    keys
}

fn as_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn unwrap_parenthesized<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(paren) => unwrap_parenthesized(&paren.expression),
        other => other,
    }
}

fn string_literal<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::StringLiteral<'a>> {
    match expression {
        Expression::StringLiteral(string) => Some(string),
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Component options resolution (export default / defineComponent).
// ---------------------------------------------------------------------------

fn find_component_options<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    let mut bindings: FxHashMap<&'a str, &'a ObjectExpression<'a>> = FxHashMap::default();

    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(object) = options_from_expression(init, &bindings)
            {
                bindings.insert(id.name.as_str(), object);
            }
        }
    }

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        if let Some(object) = options_from_export(&export.declaration, &bindings) {
            return Some(object);
        }
    }

    None
}

fn options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => options_from_call(call, bindings),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            options_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => options_from_expression(&ts_as.expression, bindings),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_call<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| options_from_expression(expression, bindings)),
    }
}

#[cfg(test)]
mod tests {
    use super::NoDupeKeys;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDupeKeys));
        linter
    }

    #[test]
    fn test_valid_unique_keys() {
        let source = r#"
export default {
  props: ['foo'],
  data() {
    return { bar: 1 }
  },
  computed: {
    baz() { return 2 }
  },
  methods: {
    qux() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_options_object() {
        let source = r#"
import { ref } from 'vue'
const foo = ref(0)
const foo2 = ref(1)
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_prop_duplicated_in_data() {
        let source = r#"
export default {
  props: ['foo'],
  data() {
    return { foo: 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_computed_duplicated_in_methods() {
        let source = r#"
export default {
  computed: {
    bar() { return 2 }
  },
  methods: {
    bar() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_prop_object_form_duplicated_in_computed() {
        let source = r#"
export default {
  props: {
    count: Number
  },
  computed: {
    count() { return 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_define_component_duplicate() {
        let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  data() {
    return { value: 1 }
  },
  methods: {
    value() {}
  }
})
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_identifier_export_duplicate() {
        let source = r#"
const component = {
  props: ['name'],
  methods: {
    name() {}
  }
}

export default component
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_arrow_data_concise_body_duplicate() {
        let source = r#"
export default {
  inject: ['theme'],
  data: () => ({ theme: 'dark' })
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_setup_return_duplicated_in_data() {
        let source = r#"
export default {
  data() {
    return { open: false }
  },
  setup() {
    return { open: true }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_multiple_duplicates_reported() {
        let source = r#"
export default {
  props: ['a', 'b'],
  computed: {
    a() { return 1 }
  },
  methods: {
    b() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_inject_object_form_duplicate() {
        let source = r#"
export default {
  inject: {
    foo: { from: 'bar' }
  },
  computed: {
    foo() { return 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_spread_in_object_is_ignored() {
        let source = r#"
export default {
  props: ['foo'],
  computed: {
    ...mapGetters(['count'])
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }
}
