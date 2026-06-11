//! script/no-side-effects-in-computed-properties
//!
//! Disallow side effects in Options API `computed` getters.
//!
//! A computed getter must be a pure function of reactive state: it derives a
//! value and returns it. Mutating component state from inside a getter
//! (assigning to `this.*`, incrementing it, or calling a mutating array method
//! such as `push`/`splice` on `this.*`) makes the dependency graph
//! unpredictable — the getter both reads and writes reactive data, so the
//! recompute can loop or fire in surprising orders. Such mutations almost
//! always belong in a `method` or `watch` handler instead.
//!
//! This is a port of
//! [`vue/no-side-effects-in-computed-properties`](https://eslint.vuejs.org/rules/no-side-effects-in-computed-properties.html)
//! from eslint-plugin-vue, scoped to the Options API `computed` option. Nested
//! functions inside a getter are not traversed: they rebind `this`, so writes
//! there do not mutate the component instance.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   data() {
//!     return { count: 0, items: [] }
//!   },
//!   computed: {
//!     doubled() {
//!       this.count = this.count * 2 // side effect: assigns to data
//!       return this.count
//!     },
//!     reversed() {
//!       return this.items.reverse() // side effect: mutates the array
//!     }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   data() {
//!     return { count: 0, items: [] }
//!   },
//!   computed: {
//!     doubled() {
//!       return this.count * 2
//!     },
//!     reversed() {
//!       return [...this.items].reverse() // operate on a copy
//!     }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, AssignmentTarget, BindingPattern, CallExpression, ExportDefaultDeclarationKind,
    Expression, Function, ObjectExpression, ObjectProperty, ObjectPropertyKind, Program,
    PropertyKey, PropertyKind, SimpleAssignmentTarget, Statement,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_assignment_expression, walk_call_expression, walk_update_expression},
};
use oxc_span::Span;
use vize_carton::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-side-effects-in-computed-properties",
    description: "Disallow side effects in Options API computed getters",
    default_severity: Severity::Error,
};

/// Array methods that mutate their receiver in place.
const MUTATING_ARRAY_METHODS: &[&str] = &[
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "sort",
    "reverse",
    "fill",
    "copyWithin",
];

/// Disallow side effects in Options API computed getters.
pub struct NoSideEffectsInComputed;

impl ScriptRule for NoSideEffectsInComputed {
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
        let Some(computed) = find_computed_object(options) else {
            return;
        };
        check_computed_object(computed, offset, result);
    }
}

fn check_computed_object(
    computed: &ObjectExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for property in &computed.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if let Some(getter) = computed_getter_body(property) {
            check_getter(getter, offset, result);
        }
    }
}

/// The body expression of a computed getter, if `property` declares one.
///
/// Accepts the shorthand `foo() {}` / `foo: () => {}` getter forms and the
/// accessor-object form `foo: { get() {} }`.
fn computed_getter_body<'a>(property: &'a ObjectProperty<'a>) -> Option<GetterBody<'a>> {
    match &property.value {
        Expression::FunctionExpression(function) => Some(GetterBody::Function(function)),
        Expression::ArrowFunctionExpression(arrow) => {
            Some(GetterBody::ArrowBody(&arrow.body.statements))
        }
        Expression::ObjectExpression(object) => accessor_object_getter(object),
        _ => None,
    }
}

/// Locate the `get` accessor inside `{ get() {}, set() {} }`.
fn accessor_object_getter<'a>(object: &'a ObjectExpression<'a>) -> Option<GetterBody<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let is_get = property.kind == PropertyKind::Get
            || matches!(property_key_name(&property.key), Some("get"));
        if !is_get {
            continue;
        }
        match &property.value {
            Expression::FunctionExpression(function) => {
                return Some(GetterBody::Function(function));
            }
            Expression::ArrowFunctionExpression(arrow) => {
                return Some(GetterBody::ArrowBody(&arrow.body.statements));
            }
            _ => {}
        }
    }
    None
}

enum GetterBody<'a> {
    Function(&'a Function<'a>),
    ArrowBody(&'a [Statement<'a>]),
}

fn check_getter(getter: GetterBody<'_>, offset: usize, result: &mut ScriptLintResult) {
    let mut visitor = SideEffectVisitor { offset, result };
    match getter {
        GetterBody::Function(function) => {
            if let Some(body) = function.body.as_ref() {
                visitor.visit_function_body(body);
            }
        }
        GetterBody::ArrowBody(statements) => {
            for statement in statements {
                visitor.visit_statement(statement);
            }
        }
    }
}

/// Walks a single computed getter body and reports writes to `this.*`.
///
/// Nested function/arrow scopes are deliberately not traversed: a non-arrow
/// nested function rebinds `this`, and even an arrow callback inside the getter
/// is the caller's responsibility to schedule, so flagging it here would
/// produce false positives matching eslint-plugin-vue's behavior.
struct SideEffectVisitor<'rule> {
    offset: usize,
    result: &'rule mut ScriptLintResult,
}

impl<'a> Visit<'a> for SideEffectVisitor<'_> {
    // Do not descend into nested functions: they get their own `this`.
    fn visit_function(&mut self, _it: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
    }

    fn visit_assignment_expression(&mut self, it: &oxc_ast::ast::AssignmentExpression<'a>) {
        if assignment_target_is_this_member(&it.left) {
            self.report(it.span, "an assignment to `this`");
        }
        walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &oxc_ast::ast::UpdateExpression<'a>) {
        if simple_target_is_this_member(&it.argument) {
            self.report(it.span, "an update of `this`");
        }
        walk_update_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = &it.callee
            && MUTATING_ARRAY_METHODS.contains(&member.property.name.as_str())
            && expression_root_is_this(&member.object)
        {
            self.report(it.span, "a mutating method call on `this`");
        }
        walk_call_expression(self, it);
    }
}

impl SideEffectVisitor<'_> {
    fn report(&mut self, span: Span, what: &str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        let mut message = CompactString::with_capacity(what.len() + 56);
        message.push_str("Unexpected side effect in a computed property: ");
        message.push_str(what);
        message.push('.');
        let diagnostic = LintDiagnostic::error(META.name, message, start, end)
            .with_label("side effect inside computed getter", start, end)
            .with_help(
                "A computed getter must be pure: derive and return a value without mutating \
                 component state. Move the mutation to a method or a watch handler.",
            );
        self.result.add_diagnostic(diagnostic);
    }
}

/// Whether an assignment target is a member access rooted at `this`
/// (e.g. `this.foo`, `this.foo.bar`, `this['foo']`).
fn assignment_target_is_this_member(target: &AssignmentTarget<'_>) -> bool {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => expression_root_is_this(&member.object),
        AssignmentTarget::ComputedMemberExpression(member) => {
            expression_root_is_this(&member.object)
        }
        _ => false,
    }
}

fn simple_target_is_this_member(target: &SimpleAssignmentTarget<'_>) -> bool {
    match target {
        SimpleAssignmentTarget::StaticMemberExpression(member) => {
            expression_root_is_this(&member.object)
        }
        SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            expression_root_is_this(&member.object)
        }
        _ => false,
    }
}

/// Whether a member-access chain bottoms out at `this`
/// (`this`, `this.a`, `this.a.b`, `this.a[i]`, `this!.a`, `(this).a`, ...).
fn expression_root_is_this(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ThisExpression(_) => true,
        Expression::StaticMemberExpression(member) => expression_root_is_this(&member.object),
        Expression::ComputedMemberExpression(member) => expression_root_is_this(&member.object),
        Expression::PrivateFieldExpression(member) => expression_root_is_this(&member.object),
        Expression::ParenthesizedExpression(paren) => expression_root_is_this(&paren.expression),
        Expression::TSNonNullExpression(non_null) => expression_root_is_this(&non_null.expression),
        Expression::TSAsExpression(ts_as) => expression_root_is_this(&ts_as.expression),
        Expression::TSSatisfiesExpression(satisfies) => {
            expression_root_is_this(&satisfies.expression)
        }
        _ => false,
    }
}

fn find_computed_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("computed"))
            && let Expression::ObjectExpression(object) = &property.value
        {
            return Some(object);
        }
    }
    None
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
//
// Mirrors the resolution in `no_dupe_keys`: a plain object, an identifier bound
// to one, or a `defineComponent(...)` wrapper, optionally through TS
// expression wrappers.
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
    call: &'a CallExpression<'a>,
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
    use super::NoSideEffectsInComputed;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoSideEffectsInComputed));
        linter
    }

    #[test]
    fn test_valid_pure_getter() {
        let source = r#"
export default {
  data() {
    return { count: 0 }
  },
  computed: {
    doubled() {
      return this.count * 2
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_options_object() {
        let source = r#"
import { computed } from 'vue'
const doubled = computed(() => count.value * 2)
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_computed() {
        let source = r#"
export default {
  data() {
    return { count: 0 }
  },
  methods: {
    bump() {
      this.count++
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_assignment_to_this() {
        let source = r#"
export default {
  data() {
    return { count: 0 }
  },
  computed: {
    doubled() {
      this.count = this.count * 2
      return this.count
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_compound_assignment_to_this() {
        let source = r#"
export default {
  computed: {
    total() {
      this.sum += 1
      return this.sum
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_update_expression_on_this() {
        let source = r#"
export default {
  computed: {
    next() {
      this.count++
      return this.count
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_nested_member_assignment() {
        let source = r#"
export default {
  computed: {
    label() {
      this.state.label = 'x'
      return this.state.label
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_computed_member_assignment() {
        let source = r#"
export default {
  computed: {
    first() {
      this.items[0] = 1
      return this.items[0]
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_mutating_array_method() {
        let source = r#"
export default {
  computed: {
    reversed() {
      return this.items.reverse()
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_push_on_nested_this() {
        let source = r#"
export default {
  computed: {
    grow() {
      this.state.list.push(1)
      return this.state.list
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_mutating_method_on_local_copy() {
        // `reverse` on a fresh array literal is fine — no `this` receiver.
        let source = r#"
export default {
  computed: {
    reversed() {
      return [...this.items].reverse()
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_assignment_to_local_variable() {
        let source = r#"
export default {
  computed: {
    sum() {
      let total = 0
      for (const n of this.items) {
        total += n
      }
      return total
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_assignment_in_nested_function() {
        // A non-arrow nested function rebinds `this`, so writing to its `this`
        // does not mutate the component instance.
        let source = r#"
export default {
  computed: {
    handler() {
      return function () {
        this.count = 1
      }
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_accessor_object_getter() {
        let source = r#"
export default {
  computed: {
    value: {
      get() {
        this.count = 1
        return this.count
      },
      set(v) {
        this.count = v
      }
    }
  }
}
"#;
        // Only the getter is checked; the setter's assignment is allowed.
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_define_component() {
        let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  computed: {
    doubled() {
      this.count *= 2
      return this.count
    }
  }
})
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_multiple_side_effects_reported() {
        let source = r#"
export default {
  computed: {
    a() {
      this.x = 1
      this.y++
      return this.x
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_valid_non_mutating_array_method() {
        let source = r#"
export default {
  computed: {
    mapped() {
      return this.items.map((n) => n * 2)
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }
}
