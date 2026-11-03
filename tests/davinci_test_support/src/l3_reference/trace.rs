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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceEvent {
    pub backend: Backend,
    pub op_id: u32,
    pub label: &'static str,
    pub effect: Option<u32>,
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
    let label = match backend {
        Backend::Vdom => vdom(op.kind),
        Backend::Vapor => vapor(op.kind),
    };
    TraceEvent {
        backend,
        op_id: op.id,
        label,
        effect: op.effect,
    }
}

/// Ops still to execute and the observations so far.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine<'p> {
    pub remaining: &'p [Op],
    pub trace: Vec<TraceEvent>,
}

/// One small step. A machine with no op left is halted: `Err` returns it.
pub fn step(backend: Backend, machine: Machine<'_>) -> Result<Machine<'_>, Machine<'_>> {
    let Some((op, remaining)) = machine.remaining.split_first() else {
        return Err(machine);
    };
    let mut trace = machine.trace;
    trace.push(interpret(backend, op));
    Ok(Machine { remaining, trace })
}

/// Steps from `trace` until the machine halts.
#[must_use]
pub fn run_ops(backend: Backend, ops: &[Op], trace: Vec<TraceEvent>) -> Vec<TraceEvent> {
    let mut machine = Machine {
        remaining: ops,
        trace,
    };
    loop {
        match step(backend, machine) {
            Ok(next) => machine = next,
            Err(halted) => return halted.trace,
        }
    }
}

#[must_use]
pub fn run(backend: Backend, program: &Program) -> Vec<TraceEvent> {
    run_ops(backend, &program.ops, Vec::new())
}
