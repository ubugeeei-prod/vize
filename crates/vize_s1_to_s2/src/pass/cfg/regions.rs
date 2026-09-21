//! The template-complexity pass's region recursion: one pre-order visit
//! in page order, carrying the nesting depth down as an inherited
//! attribute. The rules are `complexity-metrics.md`'s; each arm below
//! names the rule it implements.

use vize_davinci::id::NodeId;
use vize_s0::ensure_sufficient_stack;
use vize_s2::op::{BindingOp, DynamicName, IfOp, Op};

use super::expr::score;
use super::{ComplexityFacts, Contribution, DecisionKind};
use crate::pass::walk::PageWalk;

/// Visit one region whose ops stand at nesting `depth`.
pub(super) fn visit_region(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    depth: u32,
    facts: &mut ComplexityFacts,
) {
    ensure_sufficient_stack(|| visit_region_guarded(walk, ops, depth, facts));
}

fn visit_region_guarded(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    depth: u32,
    facts: &mut ComplexityFacts,
) {
    for op in ops {
        let id = walk.mint();
        match op {
            Op::Element(element) => {
                let inner = owner_bindings(walk, &element.bindings, depth, facts);
                visit_nested(walk, &element.children.ops, inner, facts);
            }
            Op::Component(component) => {
                let inner = owner_bindings(walk, &component.bindings, depth, facts);
                visit_nested(walk, &component.children.ops, inner, facts);
            }
            Op::Text(_) | Op::Comment(_) => {}
            Op::Interpolation(interpolation) => {
                score(interpolation.expression, id, depth, facts);
            }
            Op::If(if_op) => visit_if(walk, if_op, id, depth, facts),
            Op::For(for_op) => {
                // Rule `for`: one decision (continue or exit), a structure.
                // The source evaluates once, outside the repeated body.
                facts.push(Contribution {
                    span: for_op.binding.source.span(),
                    kind: DecisionKind::For,
                    op: id,
                    nesting: depth,
                    cyclomatic: 1,
                    cognitive: 1 + depth,
                });
                score(for_op.binding.source, id, depth, facts);
                visit_nested(walk, &for_op.region.ops, depth + 1, facts);
            }
            Op::Slot(slot) => {
                // An outlet's fallback is not a decision of this template
                // (the parent chooses), so it neither counts nor nests.
                if let DynamicName::Dynamic(name) = slot.name {
                    score(name, id, depth, facts);
                }
                let _ = owner_bindings(walk, &slot.bindings, depth, facts);
                visit_nested(walk, &slot.fallback.ops, depth, facts);
            }
        }
    }
}

/// Enter a region at `depth`, recording the deepest depth reached.
fn visit_nested(walk: &mut PageWalk, ops: &[Op<'_>], depth: u32, facts: &mut ComplexityFacts) {
    if !ops.is_empty() {
        facts.max_nesting = facts.max_nesting.max(depth);
    }
    visit_region(walk, ops, depth, facts);
}

/// Rules `if` / `else-if` / `else`. Every condition is one decision, so
/// the `ui.if` contributes exactly "each branch beyond the first, plus one
/// without a `v-else`": the conditions are attributed where they stand.
fn visit_if(
    walk: &mut PageWalk,
    if_op: &IfOp<'_>,
    id: Option<NodeId>,
    depth: u32,
    facts: &mut ComplexityFacts,
) {
    for (index, branch) in if_op.branches.iter().enumerate() {
        match branch.condition {
            Some(condition) => {
                let (kind, cognitive) = if index == 0 {
                    (DecisionKind::If, 1 + depth)
                } else {
                    (DecisionKind::ElseIf, 1)
                };
                facts.push(Contribution {
                    span: condition.span(),
                    kind,
                    op: id,
                    nesting: depth,
                    cyclomatic: 1,
                    cognitive,
                });
                // A condition evaluates before its branch is entered.
                score(condition, id, depth, facts);
            }
            None => facts.push(Contribution {
                span: branch.span,
                kind: DecisionKind::Else,
                op: id,
                nesting: depth,
                cyclomatic: 0,
                cognitive: 1,
            }),
        }
        visit_nested(walk, &branch.region.ops, depth + 1, facts);
    }
}

/// Score an owner's attached bindings (each minted in page order) and
/// return the depth its children stand at: one deeper when the owner
/// carries a scoped slot (rule `scoped-slot`), else `depth`.
fn owner_bindings(
    walk: &mut PageWalk,
    bindings: &[BindingOp<'_>],
    depth: u32,
    facts: &mut ComplexityFacts,
) -> u32 {
    let mut scoped = false;
    for binding in bindings {
        let id = walk.mint();
        match binding {
            BindingOp::Bind(bind) => {
                dynamic_name(bind.name, id, depth, facts);
                if let Some(value) = bind.value {
                    score(value, id, depth, facts);
                }
            }
            BindingOp::On(on) => {
                dynamic_name(on.name, id, depth, facts);
                if let Some(handler) = on.handler {
                    score(handler, id, depth, facts);
                }
            }
            BindingOp::Model(model) => {
                // The write side is the same authored text as the read.
                dynamic_name(model.argument, id, depth, facts);
                score(model.contract.read, id, depth, facts);
            }
            BindingOp::SlotContent(content) => {
                dynamic_name(content.name, id, depth, facts);
                if content.params.is_some() {
                    scoped |= scoped_slot(content.span, id, depth, facts);
                }
            }
            BindingOp::VueSlotScope(scope) => {
                if scope.params.is_some() {
                    scoped |= scoped_slot(scope.span, id, depth, facts);
                }
            }
            BindingOp::VueDirective(directive) => {
                dynamic_name(directive.argument, id, depth, facts);
                if let Some(value) = directive.value {
                    score(value, id, depth, facts);
                }
            }
            BindingOp::VueSync(sync) => score(sync.value, id, depth, facts),
            BindingOp::VueMemo(memo) => score(memo.value, id, depth, facts),
            BindingOp::VueShow(show) => score(show.value, id, depth, facts),
            BindingOp::VueHtml(html) => {
                if let Some(value) = html.value {
                    score(value, id, depth, facts);
                }
            }
            BindingOp::VueText(text) => {
                if let Some(value) = text.value {
                    score(value, id, depth, facts);
                }
            }
            // Style-block binds are not template control flow; the
            // one-shot and cloak markers carry no expression.
            BindingOp::VueCssBind(_) | BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => {}
        }
    }
    if scoped { depth + 1 } else { depth }
}

fn dynamic_name(
    name: Option<DynamicName<'_>>,
    id: Option<NodeId>,
    depth: u32,
    facts: &mut ComplexityFacts,
) {
    if let Some(DynamicName::Dynamic(expr)) = name {
        score(expr, id, depth, facts);
    }
}

/// Rule `scoped-slot`: recorded for the breakdown, no increment.
fn scoped_slot(
    span: vize_s0::Span,
    id: Option<NodeId>,
    depth: u32,
    facts: &mut ComplexityFacts,
) -> bool {
    facts.push(Contribution {
        span,
        kind: DecisionKind::ScopedSlot,
        op: id,
        nesting: depth,
        cyclomatic: 0,
        cognitive: 0,
    });
    true
}
