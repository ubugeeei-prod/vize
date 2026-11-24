//! P3-4 small-step trace semantics: each step executes the next op of the
//! program and records one backend observation.

#[cfg(creusot)]
use creusot_std::prelude::Clone;
#[cfg(creusot)]
use creusot_std::prelude::*;

use super::program::{Op, OpKind, Program};

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub enum Backend {
    Vdom,
    Vapor,
}

impl Backend {
    pub const ALL: [Self; 2] = [Self::Vdom, Self::Vapor];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Vdom => "vdom",
            Self::Vapor => "vapor",
        }
    }
}

/// One observation. The label is a function of backend and op kind, so the
/// event keeps the kind and compares without string contents.
#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub struct TraceEvent {
    pub backend: Backend,
    pub op_id: u32,
    pub kind: OpKind,
    pub effect: Option<u32>,
}

impl TraceEvent {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self.backend {
            Backend::Vdom => vdom(self.kind),
            Backend::Vapor => vapor(self.kind),
        }
    }
}

fn vdom(kind: OpKind) -> &'static str {
    match kind {
        OpKind::SetProp => "patch-prop",
        OpKind::SetDynamicProps => "patch-dynamic-props",
        OpKind::SetText => "set-text",
        OpKind::SetEvent => "patch-event",
        OpKind::SetHtml => "set-html",
        OpKind::SetTemplateRef => "set-template-ref",
        OpKind::InsertNode => "create-element",
        OpKind::PrependNode => "prepend-node",
        OpKind::Directive => "apply-directive",
        OpKind::Branch => "branch",
        OpKind::Loop => "iterate",
        OpKind::CreateComponent => "create-component",
        OpKind::SlotOutlet => "render-slot",
        OpKind::GetTextChild => "get-text-child",
        OpKind::ChildRef => "child-ref",
        OpKind::NextRef => "next-ref",
        OpKind::SetChildValue => "normalize-child-value",
    }
}

fn vapor(kind: OpKind) -> &'static str {
    match kind {
        OpKind::SetProp => "assign-prop",
        OpKind::SetDynamicProps => "assign-dynamic-props",
        OpKind::SetText => "text-effect",
        OpKind::SetEvent => "listen",
        OpKind::SetHtml => "html-effect",
        OpKind::SetTemplateRef => "template-ref",
        OpKind::InsertNode => "create-node",
        OpKind::PrependNode => "prepend-node",
        OpKind::Directive => "directive-effect",
        OpKind::Branch => "conditional-effect",
        OpKind::Loop => "list-effect",
        OpKind::CreateComponent => "component-effect",
        OpKind::SlotOutlet => "slot-effect",
        OpKind::GetTextChild => "get-text-child",
        OpKind::ChildRef => "child-ref",
        OpKind::NextRef => "next-ref",
        OpKind::SetChildValue => "child-value-effect",
    }
}

/// The observation of `op` under `backend` (Lean `interpret`): it names the op.
#[cfg(creusot)]
#[logic(open)]
pub fn observe(backend: Backend, op: Op) -> TraceEvent {
    TraceEvent {
        backend,
        op_id: op.id,
        kind: op.kind,
        effect: op.effect,
    }
}

#[cfg_attr(creusot, ensures(result == observe(backend, *op)))]
#[must_use]
pub fn interpret(backend: Backend, op: &Op) -> TraceEvent {
    TraceEvent {
        backend,
        op_id: op.id,
        kind: op.kind,
        effect: op.effect,
    }
}

/// Steps from `trace` until no op remains. Each iteration is one small
/// step: it executes the next op and records its observation, so the machine
/// appends one event per op, in list order (Lean `runOps_eq`).
#[cfg_attr(creusot, ensures(result@.len() == trace@.len() + ops@.len()))]
#[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < trace@.len() ==> result@[i] == trace@[i]))]
#[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < ops@.len()
    ==> result@[trace@.len() + i] == observe(backend, ops@[i])))]
#[must_use]
pub fn run_ops(backend: Backend, ops: &[Op], trace: Vec<TraceEvent>) -> Vec<TraceEvent> {
    let mut trace = trace;
    #[cfg(creusot)]
    let earlier = snapshot!(trace);
    #[cfg_attr(creusot, invariant(trace@.len() == (*earlier)@.len() + produced.len()))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < (*earlier)@.len()
        ==> trace@[i] == (*earlier)@[i]))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < produced.len()
        ==> trace@[(*earlier)@.len() + i] == observe(backend, ops@[i])))]
    for op in ops {
        trace.push(interpret(backend, op));
    }
    trace
}

/// The trace of the whole program: position `i` is op `i` (Lean `run_opId`).
#[cfg_attr(creusot, ensures(result@.len() == program.ops@.len()))]
#[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < result@.len()
    ==> result@[i] == observe(backend, program.ops@[i])))]
#[must_use]
pub fn run(backend: Backend, program: &Program) -> Vec<TraceEvent> {
    run_ops(backend, &program.ops, Vec::new())
}
