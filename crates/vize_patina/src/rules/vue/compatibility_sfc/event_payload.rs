//! `vue/prefer-single-event-payload`.
//!
//! A handler written inline, `@move="onMove($event)"`, only receives the first
//! payload value, so an event that carries several values loses the rest.
//! Reports, as eslint-plugin-vue does, every declaration with more than one
//! payload value: a `defineEmits` type (`move: [x: number, y: number]`, a call
//! signature or a method signature), a runtime validator with several
//! parameters, and the same in a Vue object's `emits` option. Also reports
//! every emit call with more than one payload argument: the `defineEmits`
//! binding, a `setup` context's `emit` and `ctx.emit`, any `x.$emit` in a
//! script, and template calls of `$emit` and of the `defineEmits` binding.
//!
//! The report covers the values after the first, where the change is needed.

use super::absolute_range;
use super::events::{Block, for_block_emits, script_blocks, setup_emit_binding, setup_program};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::script::props_emits::require_explicit_emits::setup_context;
use crate::rules::script::template_scan::{DOLLAR_EMIT, for_each_template_call};
use crate::rules::script::vue_object::VueObjects;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, IdentifierReference, MemberExpression, ObjectExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Semantic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::symbol::SymbolId;
use vize_l0::{String, cstr};
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-single-event-payload",
    description: "Enforce passing a single argument to custom event emissions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/prefer-single-event-payload`.
pub struct PreferSingleEventPayload;

/// A finding: the message and its range.
type Finding = (String, ByteRange);

impl Rule for PreferSingleEventPayload {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = script_findings(ctx);
        report(ctx, findings);
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let (Some(surface), Some(_)) = (ctx.template_surface(), ctx.sfc_descriptor()) else {
            return;
        };
        let binding = setup_program(ctx).and_then(setup_emit_binding);
        if binding.is_none() && !ctx.source.contains(DOLLAR_EMIT) {
            return;
        }
        let mut findings = Vec::new();
        for_each_template_call(surface, |call| {
            if call.callee != DOLLAR_EMIT && Some(call.callee) != binding {
                return;
            }
            if let Some((start, end)) = call.arguments_after_second {
                let name = call.first_string_argument.map(|event| event.value);
                findings.push((call_message(name), ByteRange { start, end }));
            }
        });
        report(ctx, findings);
    }
}

fn report(ctx: &mut LintContext<'_>, findings: Vec<Finding>) {
    for (message, range) in findings {
        ctx.warn_at_with_help(
            message,
            range,
            "Wrap the values in one object, such as `{ x, y }`, so every listener receives all of them",
        );
    }
}

fn call_message(name: Option<&str>) -> String {
    let name = name.unwrap_or("unknown");
    cstr!(
        "Pass a single payload object instead of multiple arguments when emitting the \"{name}\" event."
    )
}

fn declaration_message(name: &str) -> String {
    cstr!(
        "Declare a single payload parameter instead of multiple parameters for the \"{name}\" event."
    )
}

fn script_findings(ctx: &LintContext<'_>) -> Vec<Finding> {
    let (script, setup) = script_blocks(ctx);
    let mut findings = Vec::new();
    if let Some(block) = &script {
        block_findings(ctx, block, None, &mut findings);
    }
    if let Some(block) = &setup {
        block_findings(ctx, block, Some(script.as_ref()), &mut findings);
    }
    findings.sort_by_key(|(_, range)| (range.start, range.end));
    findings
}

fn block_findings(
    ctx: &LintContext<'_>,
    block: &Block<'_>,
    sibling: Option<Option<&Block<'_>>>,
    findings: &mut Vec<Finding>,
) {
    // `emit`, `$emit`, `emits` and `defineEmits` all contain it.
    if !block.source.contains("mit") {
        return;
    }
    for_block_emits(ctx, block, sibling, |semantic, emits| {
        for (entry, offset) in &emits.declared {
            if let Some(payload) = entry.extra_payload {
                findings.push((
                    declaration_message(&entry.name),
                    absolute_range(*offset, payload),
                ));
            }
        }
        let mut calls = PayloadCalls {
            semantic,
            objects: &emits.objects,
            setup_emit: emits.setup_emit,
            stack: Vec::new(),
            found: Vec::new(),
        };
        calls.visit_program(block.program);
        for (name, span) in calls.found {
            findings.push((
                call_message(name.as_deref()),
                absolute_range(block.offset, span),
            ));
        }
    });
}

/// Emit calls with more than one payload argument.
struct PayloadCalls<'s, 'a> {
    semantic: &'s Semantic<'a>,
    objects: &'s VueObjects,
    setup_emit: Option<SymbolId>,
    /// `setup(props, { emit })` and `setup(props, ctx)` of each enclosing
    /// Vue object; the innermost one dispatches.
    stack: Vec<(Option<SymbolId>, Option<SymbolId>)>,
    found: Vec<(Option<String>, Span)>,
}

impl<'a> Visit<'a> for PayloadCalls<'_, 'a> {
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        if !self.objects.contains(it) {
            walk::walk_object_expression(self, it);
            return;
        }
        self.stack.push(setup_context(it));
        walk::walk_object_expression(self, it);
        self.stack.pop();
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(third) = it.arguments.get(2)
            && self.dispatches(&it.callee)
        {
            let last = it.arguments.last().map_or(third.span(), GetSpan::span);
            self.found.push((
                event_name(it.arguments.first()),
                Span::new(third.span().start, last.end),
            ));
        }
        walk::walk_call_expression(self, it);
    }
}

impl<'a> PayloadCalls<'_, 'a> {
    /// Upstream's callees: the `defineEmits` binding, the innermost `setup`
    /// context's `emit` or `ctx.emit`, and any `x.$emit`.
    fn dispatches(&self, callee: &Expression<'a>) -> bool {
        let callee = match callee.without_parentheses() {
            Expression::ChainExpression(chain) => match chain.expression.as_member_expression() {
                Some(member) => return self.member_dispatches(member),
                None => return false,
            },
            callee => callee,
        };
        if let Expression::Identifier(identifier) = callee {
            let symbol = self.symbol(identifier);
            return symbol.is_some()
                && (symbol == self.setup_emit
                    || self.stack.last().is_some_and(|(emit, _)| *emit == symbol));
        }
        callee
            .as_member_expression()
            .is_some_and(|member| self.member_dispatches(member))
    }

    fn member_dispatches(&self, member: &MemberExpression<'a>) -> bool {
        match member.static_property_name() {
            Some("$emit") => true,
            Some("emit") => match member.object().without_parentheses() {
                Expression::Identifier(object) => {
                    let symbol = self.symbol(object);
                    symbol.is_some()
                        && self
                            .stack
                            .last()
                            .is_some_and(|(_, context)| *context == symbol)
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn symbol(&self, identifier: &IdentifierReference<'a>) -> Option<SymbolId> {
        let scoping = self.semantic.scoping();
        scoping
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }
}

/// Upstream's `isStringLiteral`: a string, or a template without
/// substitutions.
fn event_name(argument: Option<&Argument<'_>>) -> Option<String> {
    match argument?.as_expression()?.without_parentheses() {
        Expression::StringLiteral(literal) => Some(String::from(literal.value.as_str())),
        Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => literal
            .single_quasi()
            .map(|quasi| String::from(quasi.as_str())),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
