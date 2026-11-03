//! P3-4 small-step trace semantics: each step executes the next op of the
//! program and records one backend observation.

use super::program::{Op, OpKind, Program};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
/// step: it executes the next op and records its observation. The state stays
/// in locals; CBMC pays heavily for a machine struct or a `&mut` trace.
#[must_use]
pub fn run_ops(backend: Backend, ops: &[Op], trace: Vec<TraceEvent>) -> Vec<TraceEvent> {
    let mut trace = trace;
    let mut remaining = ops;
    while let Some((op, rest)) = remaining.split_first() {
        trace.push(interpret(backend, op));
        remaining = rest;
    }
    trace
}

#[must_use]
pub fn run(backend: Backend, program: &Program) -> Vec<TraceEvent> {
    run_ops(backend, &program.ops, Vec::new())
}
