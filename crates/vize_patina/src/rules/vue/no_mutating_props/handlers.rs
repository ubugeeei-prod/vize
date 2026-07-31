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
//!   a destructuring target (`[props.msg] = list`) and a mutation performed
//!   through a call (`@click="props.items.push(1)"`, which upstream reports
//!   and this does not). All three leave a real bug unreported, which is the
//!   tolerable direction.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, SimpleAssignmentTarget, UpdateExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_assignment_expression, walk_update_expression},
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

/// Call `visit` with the source slice of every assignment / update target in
/// `source`, a `v-on` inline handler body.
///
/// A handler body is a statement list rather than a single expression
/// (`@click="a = 1; b = 2"` is valid), so it is parsed as a program. Bodies
/// oxc rejects are skipped: a report invented from a mis-parse would be a
/// false positive, and a template that does not compile has larger problems.
pub(super) fn for_each_mutation_target(source: &str, mut visit: impl FnMut(&str)) {
    // Cheap reject for the overwhelmingly common handler (`@click="submit"`,
    // `@click="emit('x')"`): no mutation operator, nothing to parse.
    if !source.contains('=') && !source.contains("++") && !source.contains("--") {
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
    let mut collector = MutationTargetCollector { spans: Vec::new() };
    collector.visit_program(&parsed.program);
    for span in collector.spans {
        if let Some(text) = source.get(span.start as usize..span.end as usize) {
            visit(text);
        }
    }
}

struct MutationTargetCollector {
    spans: Vec<Span>,
}

impl<'a> Visit<'a> for MutationTargetCollector {
    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let Some(span) = assignment_target_span(&it.left) {
            self.spans.push(span);
        }
        // Keep walking: the right-hand side can hold further assignments
        // (`a = (props.msg = 'x')`).
        walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if let Some(span) = simple_assignment_target_span(&it.argument) {
            self.spans.push(span);
        }
        walk_update_expression(self, it);
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

fn simple_assignment_target_span(target: &SimpleAssignmentTarget<'_>) -> Option<Span> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => Some(identifier.span),
        other => other.as_member_expression().map(GetSpan::span),
    }
}
