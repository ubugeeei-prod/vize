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
    Argument, AssignmentExpression, AssignmentTarget, CallExpression, Expression,
    SimpleAssignmentTarget, UnaryExpression, UpdateExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_call_expression, walk_unary_expression,
        walk_update_expression,
    },
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::operator::UnaryOperator;

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
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let mut collector = MutationTargetCollector {
        targets: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    for target in collector.targets {
        if let Some(text) = source.get(target.target.start as usize..target.target.end as usize) {
            visit(HandlerMutationTarget {
                target: text,
                span: target.mutation,
            });
        }
    }
}

struct MutationTargetCollector {
    targets: Vec<MutationTargetSpan>,
}

struct MutationTargetSpan {
    target: Span,
    mutation: Span,
}

impl<'a> Visit<'a> for MutationTargetCollector {
    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let Some(span) = assignment_target_span(&it.left) {
            self.push(span, it.span);
        }
        // Keep walking: the right-hand side can hold further assignments
        // (`a = (props.msg = 'x')`).
        walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if let Some(span) = simple_assignment_target_span(&it.argument) {
            self.push(span, it.span);
        }
        walk_update_expression(self, it);
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        if it.operator == UnaryOperator::Delete {
            self.push(it.argument.get_inner_expression().span(), it.span);
        }
        walk_unary_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(target) = mutating_call_target(it) {
            self.push(target.get_inner_expression().span(), it.span);
        }
        walk_call_expression(self, it);
    }
}

impl MutationTargetCollector {
    fn push(&mut self, target: Span, mutation: Span) {
        self.targets.push(MutationTargetSpan { target, mutation });
    }
}

/// The span of an assignment target, when it is a plain identifier or a member
/// access.
///
/// Destructuring targets are deliberately not resolved; see the module-level
/// note on under-matching.
fn assignment_target_span(target: &AssignmentTarget<'_>) -> Option<Span> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => Some(identifier.span),
        other => other.as_member_expression().map(GetSpan::span),
    }
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
    match callee.get_inner_expression() {
        Expression::StaticMemberExpression(member) => {
            Some((&member.object, member.property.name.as_str()))
        }
        Expression::ComputedMemberExpression(member) => match member
            .expression
            .get_inner_expression()
        {
            Expression::StringLiteral(property) => Some((&member.object, property.value.as_str())),
            _ => None,
        },
        _ => None,
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

fn simple_assignment_target_span(target: &SimpleAssignmentTarget<'_>) -> Option<Span> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => Some(identifier.span),
        other => other.as_member_expression().map(GetSpan::span),
    }
}
