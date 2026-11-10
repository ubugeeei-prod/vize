//! script/no-side-effects-in-computed-properties
//!
//! Disallow side effects in Options and Composition API `computed` getters.
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
//! from eslint-plugin-vue, covering Options getters and imported Vue `computed` calls. Nested
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
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression, Function,
    ObjectExpression, ObjectProperty, ObjectPropertyKind, Program, PropertyKey, PropertyKind,
    Statement, UnaryExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_call_expression, walk_unary_expression,
        walk_update_expression,
    },
};
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;
use vize_l0::{CompactString, FxHashMap};

mod composition;
mod this_receiver;

use this_receiver::{
    assignment_target_is_this_member, expression_root_is_this, simple_target_is_this_member,
};

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

/// Disallow side effects in Options and Composition API computed getters.
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
        if let Some(options) = find_component_options(program)
            && let Some(computed) = find_computed_object(options)
        {
            check_computed_object(computed, offset, result);
        }
        composition::check_program(program, offset, result);
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
pub(super) fn computed_getter_body<'a>(property: &'a ObjectProperty<'a>) -> Option<GetterBody<'a>> {
    match &property.value {
        Expression::FunctionExpression(function) => Some(GetterBody::Function(function)),
        Expression::ArrowFunctionExpression(arrow) => Some(GetterBody::Arrow(arrow)),
        Expression::ObjectExpression(object) => accessor_object_getter(object),
        _ => None,
    }
}

/// Locate the `get` accessor inside `{ get() {}, set() {} }`.
pub(super) fn accessor_object_getter<'a>(
    object: &'a ObjectExpression<'a>,
) -> Option<GetterBody<'a>> {
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
                return Some(GetterBody::Arrow(arrow));
            }
            _ => {}
        }
    }
    None
}

pub(super) enum GetterBody<'a> {
    Function(&'a Function<'a>),
    Arrow(&'a oxc_ast::ast::ArrowFunctionExpression<'a>),
}

fn check_getter(getter: GetterBody<'_>, offset: usize, result: &mut ScriptLintResult) {
    let mut visitor = SideEffectVisitor {
        offset,
        result,
        scoping: None,
        getter_span: Span::default(),
    };
    match getter {
        GetterBody::Function(function) => {
            if let Some(body) = function.body.as_ref() {
                visitor.visit_function_body(body);
            }
        }
        GetterBody::Arrow(arrow) => {
            for statement in &arrow.body.statements {
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
    scoping: Option<&'rule oxc_semantic::Scoping>,
    getter_span: Span,
}

impl<'a> Visit<'a> for SideEffectVisitor<'_> {
    // Do not descend into nested functions: they get their own `this`.
    fn visit_function(&mut self, _it: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
    }

    fn visit_assignment_expression(&mut self, it: &oxc_ast::ast::AssignmentExpression<'a>) {
        if assignment_target_is_this_member(&it.left)
            || composition::assignment_is_external(&it.left, self.scoping, self.getter_span)
        {
            self.report(
                it.span,
                if assignment_target_is_this_member(&it.left) {
                    "an assignment to `this`"
                } else {
                    "an assignment to external state"
                },
            );
        }
        walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &oxc_ast::ast::UpdateExpression<'a>) {
        if simple_target_is_this_member(&it.argument)
            || composition::simple_target_is_external(&it.argument, self.scoping, self.getter_span)
        {
            self.report(
                it.span,
                if simple_target_is_this_member(&it.argument) {
                    "an update of `this`"
                } else {
                    "an update of external state"
                },
            );
        }
        walk_update_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = &it.callee
            && member.property.name == "$set"
            && matches!(
                member.object.get_inner_expression(),
                Expression::ThisExpression(_)
            )
        {
            self.report(member.property.span, "a call to `this.$set`");
        }
        if let Expression::StaticMemberExpression(member) = &it.callee
            && MUTATING_ARRAY_METHODS.contains(&member.property.name.as_str())
            && ((expression_root_is_this(&member.object)
                && matches!(
                    member.object.without_parentheses(),
                    Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_)
                ))
                || composition::root_is_external(&member.object, self.scoping, self.getter_span))
        {
            self.report(
                it.span,
                if expression_root_is_this(&member.object) {
                    "a mutating method call on `this`"
                } else {
                    "a mutating method call on external state"
                },
            );
        }
        if matches!(&it.callee, Expression::StaticMemberExpression(member)
            if matches!(member.object.get_inner_expression(), Expression::Identifier(identifier) if identifier.name == "Object")
                && member.property.name == "assign")
            && let Some(target) = it.arguments.first().and_then(Argument::as_expression)
            && target.as_member_expression().is_some()
            && (expression_root_is_this(target)
                || composition::root_is_external(target, self.scoping, self.getter_span))
        {
            self.report(it.span, "an Object.assign call on external state");
        }
        walk_call_expression(self, it);
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        if it.operator == UnaryOperator::Delete
            && it.argument.as_member_expression().is_some()
            && (expression_root_is_this(&it.argument)
                || composition::root_is_external(&it.argument, self.scoping, self.getter_span))
        {
            self.report(it.span, "a deletion from external state");
        }
        walk_unary_expression(self, it);
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

pub(super) fn find_computed_object<'a>(
    options: &'a ObjectExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
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

pub(super) fn find_component_options<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
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
mod tests;
