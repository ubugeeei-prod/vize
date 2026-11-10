//! Assignment targets inside a `v-on` inline handler body.
//!
//! # Why this needs a real parse
//!
//! `vue/no-mutating-props` already sees the template, but it only inspected
//! `v-model`. An inline handler mutates a prop just as directly
//! (`@click="props.msg = 'x'"`) and was missed entirely.
//!
//! Recovering those call sites *creates* findings from template evidence,
//! which is the dangerous direction: unlike
//! [`crate::rules::script::props_emits::template_emits`], where an
//! over-matching raw scan can only suppress an "unused" report, an over-match
//! here invents a diagnostic where the user did nothing wrong. Concretely, a
//! substring sweep for `<prop> =` over the raw template would fire on all of
//! these, none of which mutate a prop:
//!
//! * `<!-- msg = 'x' -->` — an HTML comment.
//! * `<p>msg = 'x'</p>` — a text node.
//! * `<div title="msg = 'x'">` — a plain attribute, not an expression.
//! * `@click="log('msg = 1')"` — an assignment inside a string literal.
//! * `@click="msgExtra = 1"` — a longer identifier that merely starts with the
//!   prop name.
//! * `<pre v-pre>{{ msg = 1 }}</pre>` — a region Vue never compiles.
//!
//! So the evidence is taken from the *template AST* — only a `v-on`
//! [`vize_relief::DirectiveNode`] carrying an expression, which excludes
//! comments, text nodes, plain attributes and `v-pre` regions — and the
//! handler body is then parsed with oxc so that only a genuine assignment or
//! update *target* counts. An occurrence inside a string literal parses to a
//! `StringLiteral` and can never be one.
//!
//! # Direction of error
//!
//! * **Over-match** would be a false positive, so every step above is exact.
//!   The one deliberate imprecision is shadowing (a `v-for` alias or a slot
//!   variable that reuses a prop name), and it errs towards *not* reporting.
//! * **Under-match** loses a report: a body oxc cannot parse is skipped, as is
//!   a destructuring target (`[props.msg] = list`) or a mutating call whose
//!   method is not statically named (`@click="props.items[method](1)"`). These
//!   leave a real bug unreported, which is the tolerable direction.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, AssignmentExpression, AssignmentTarget, BindingIdentifier,
    CallExpression, Class, Expression, Function, MemberExpression, SimpleAssignmentTarget,
    UnaryExpression, UpdateExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_assignment_expression, walk_call_expression,
        walk_function, walk_unary_expression, walk_update_expression,
    },
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::operator::UnaryOperator;
use oxc_syntax::scope::ScopeFlags;

const MUTATING_ARRAY_METHODS: &[&str] = &[
    "push",
    "pop",
    "shift",
    "unshift",
    "reverse",
    "splice",
    "sort",
    "copyWithin",
    "fill",
];

pub(super) struct HandlerMutationTarget<'a> {
    pub(super) target: &'a str,
    pub(super) span: Span,
    pub(super) deep: bool,
}

/// Call `visit` with the source slice of every assignment / update target in
/// `source`, a `v-on` inline handler body.
///
/// A handler body is a statement list rather than a single expression
/// (`@click="a = 1; b = 2"` is valid), so it is parsed as a program. Bodies
/// oxc rejects are skipped: a report invented from a mis-parse would be a
/// false positive, and a template that does not compile has larger problems.
pub(super) fn for_each_mutation_target(
    source: &str,
    mut visit: impl FnMut(HandlerMutationTarget<'_>),
) {
    // Cheap reject for the overwhelmingly common handler (`@click="submit"`,
    // `@click="emit('x')"`): no mutation operator, nothing to parse.
    if !may_mutate(source) {
        return;
    }
    // `@click="function () { ... }"` is a function expression, which a
    // statement list only accepts inside parentheses.
    let trimmed = source.trim_start();
    let wrapped = (trimmed.starts_with("function") || trimmed.starts_with("async function"))
        .then(|| vize_l0::cstr!("({source})"));
    let shift = u32::from(wrapped.is_some());
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        wrapped.as_deref().unwrap_or(source),
        SourceType::default().with_typescript(true),
    )
    .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let mut collector = MutationTargetCollector {
        targets: Vec::new(),
        shadowed: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    for target in collector.targets {
        let start = target.target.start.saturating_sub(shift) as usize;
        let end = target.target.end.saturating_sub(shift) as usize;
        if let Some(text) = source.get(start..end) {
            visit(HandlerMutationTarget {
                target: text,
                span: Span::new(
                    target.mutation.start.saturating_sub(shift),
                    target.mutation.end.saturating_sub(shift),
                ),
                deep: target.deep,
            });
        }
    }
}

struct MutationTargetCollector<'a> {
    targets: Vec<MutationTargetSpan>,
    /// Names bound by enclosing handler functions (parameters and locals).
    shadowed: Vec<&'a str>,
}

struct MutationTargetSpan {
    target: Span,
    mutation: Span,
    deep: bool,
}

impl<'a> Visit<'a> for MutationTargetCollector<'a> {
    // A function handler (`@click="() => rows.pop()"`) runs the same mutations;
    // its parameters and locals shadow props of the same name.
    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        let depth = self.shadowed.len();
        DeclaredNames(&mut self.shadowed).visit_formal_parameters(&it.params);
        if let Some(body) = &it.body {
            DeclaredNames(&mut self.shadowed).visit_function_body(body);
        }
        walk_function(self, it, flags);
        self.shadowed.truncate(depth);
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        let depth = self.shadowed.len();
        DeclaredNames(&mut self.shadowed).visit_formal_parameters(&it.params);
        DeclaredNames(&mut self.shadowed).visit_function_body(&it.body);
        walk_arrow_function_expression(self, it);
        self.shadowed.truncate(depth);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let Some((root, span)) = assignment_target(&it.left)
            && !self.is_shadowed(root)
        {
            self.push(span, it.span, false);
        }
        // Keep walking: the right-hand side can hold further assignments
        // (`a = (props.msg = 'x')`).
        walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if let Some((root, span)) = simple_assignment_target(&it.argument)
            && !self.is_shadowed(root)
        {
            self.push(span, it.span, false);
        }
        walk_update_expression(self, it);
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        if it.operator == UnaryOperator::Delete
            && let Some(root) = member_chain_root(&it.argument)
            && !self.is_shadowed(root)
        {
            self.push(it.argument.get_inner_expression().span(), it.span, false);
        }
        walk_unary_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(target) = mutating_call_target(it)
            && let Some(root) = member_chain_root(target)
            && !self.is_shadowed(root)
        {
            self.push(target.get_inner_expression().span(), it.span, true);
        }
        walk_call_expression(self, it);
    }
}

impl MutationTargetCollector<'_> {
    fn is_shadowed(&self, root: &str) -> bool {
        self.shadowed.contains(&root)
    }

    fn push(&mut self, target: Span, mutation: Span, deep: bool) {
        self.targets.push(MutationTargetSpan {
            target,
            mutation,
            deep,
        });
    }
}

/// Collects the names a function binds, without entering nested functions.
struct DeclaredNames<'s, 'a>(&'s mut Vec<&'a str>);

impl<'a> Visit<'a> for DeclaredNames<'_, 'a> {
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.0.push(it.name.as_str());
    }

    fn visit_function(&mut self, it: &Function<'a>, _: ScopeFlags) {
        if let Some(id) = &it.id {
            self.0.push(id.name.as_str());
        }
    }

    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, it: &Class<'a>) {
        if let Some(id) = &it.id {
            self.0.push(id.name.as_str());
        }
    }
}

/// The root identifier of a plain member chain (`rows`, `item.list[0]`,
/// `(rows?.shift)`). A call result such as `rows.slice()` is a fresh value.
fn member_chain_root<'t>(expression: &'t Expression<'_>) -> Option<&'t str> {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::ChainExpression(chain) => chain
            .expression
            .as_member_expression()
            .and_then(|member| member_chain_root(member.object())),
        expression => expression
            .as_member_expression()
            .and_then(|member| member_chain_root(member.object())),
    }
}

/// The span of an assignment target, when it is a plain identifier or a member
/// access.
///
/// Destructuring targets are deliberately not resolved; see the module-level
/// note on under-matching.
fn assignment_target<'t>(target: &'t AssignmentTarget<'_>) -> Option<(&'t str, Span)> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some((identifier.name.as_str(), identifier.span))
        }
        other => other.as_member_expression().and_then(member_target),
    }
}

fn member_target<'t>(member: &'t MemberExpression<'_>) -> Option<(&'t str, Span)> {
    Some((member_chain_root(member.object())?, member.span()))
}

fn mutating_call_target<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    let (object, property) = static_call_member(&call.callee)?;

    if MUTATING_ARRAY_METHODS.contains(&property) {
        return Some(object);
    }

    if property == "assign"
        && is_identifier_named(object, "Object")
        && let Some(argument) = call.arguments.first().and_then(Argument::as_expression)
    {
        return Some(argument);
    }

    None
}

fn static_call_member<'a>(callee: &'a Expression<'a>) -> Option<(&'a Expression<'a>, &'a str)> {
    // `(rows?.shift)?.()` wraps the member in a chain.
    let member = match callee.get_inner_expression() {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
        callee => callee.as_member_expression(),
    }?;
    match member {
        MemberExpression::StaticMemberExpression(member) => {
            Some((&member.object, member.property.name.as_str()))
        }
        MemberExpression::ComputedMemberExpression(member) => match member
            .expression
            .get_inner_expression()
        {
            Expression::StringLiteral(property) => Some((&member.object, property.value.as_str())),
            _ => None,
        },
        MemberExpression::PrivateFieldExpression(_) => None,
    }
}

fn is_identifier_named(expression: &Expression<'_>, expected: &str) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::Identifier(identifier) if identifier.name.as_str() == expected
    )
}

fn may_mutate(source: &str) -> bool {
    source.contains('=')
        || source.contains("++")
        || source.contains("--")
        || source.contains("delete")
        || source.contains("Object.assign")
        || source.contains("assign")
        || MUTATING_ARRAY_METHODS
            .iter()
            .any(|method| source.contains(method))
}

fn simple_assignment_target<'t>(target: &'t SimpleAssignmentTarget<'_>) -> Option<(&'t str, Span)> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some((identifier.name.as_str(), identifier.span))
        }
        other => other.as_member_expression().and_then(member_target),
    }
}
