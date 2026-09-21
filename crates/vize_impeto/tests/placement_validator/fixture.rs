//! A hand-built S3 program exercising every placement alternative.
//!
//! ```text
//! op0 insert-node <div>                     r0  static
//! op1   set-event @click="save"             r0  fx1   -> cache
//! op2   set-prop :title="msg"               r0  fx2
//! r1  children of op0
//! op3   set-text {{ msg }}                  r1  fx3   -> group(op2)
//! op4   if ok                               r1  fx4
//! r2    branch of op4
//! op5     insert-node <p>                   r2  fx5   -> hoist
//! r3      children of op5
//! op6       set-text "hi"                   r3  fx6
//! op7   set-text {{ msg }}                  r1  fx7   (keyed pred is op4)
//! ```

use vize_impeto::op::{
    EdgeKind, EffectId, EffectScope, Op, OpId, OpKind, Phase, Program, Region, RegionId, StateEdge,
};
use vize_impeto::operand::{Operand, OperandRole, OperandValue, ValueKind};
use vize_impeto::placement::{Placement, PlacementRecord, PlacementSet};
use vize_impeto::verify::verify;
use vize_s0::{Allocator, Span, String, cstr};

pub const JS: ValueKind = ValueKind::Js;
pub const LIT: ValueKind = ValueKind::Literal;

pub struct Build<'a> {
    pub program: Program<'a>,
    last_effect: Option<u32>,
}

impl<'a> Build<'a> {
    pub fn new(allocator: &'a Allocator, span: (u32, u32)) -> Self {
        let mut program = Program::new(allocator, Phase::Built);
        program.push_region(Region::root(Span::new(span.0, span.1)));
        Self {
            program,
            last_effect: None,
        }
    }

    pub fn region(&mut self, id: u32, parent: u32, owner: u32, span: (u32, u32)) {
        self.program.push_region(Region::child(
            RegionId::new(id),
            RegionId::new(parent),
            OpId::new(owner),
            Span::new(span.0, span.1),
        ));
    }

    /// Push an op; dynamic ops get their own scope and extend the effect chain.
    pub fn op(&mut self, id: u32, kind: OpKind, region: u32, span: (u32, u32), dynamic: bool) {
        let span = Span::new(span.0, span.1);
        let mut op = Op::new(OpId::new(id), kind, RegionId::new(region), span);
        if dynamic {
            op = op.with_effect(EffectId::new(id));
            self.program.push_effect(EffectScope {
                id: EffectId::new(id),
                owner: OpId::new(id),
                region: RegionId::new(region),
                span,
            });
            if let Some(previous) = self.last_effect.replace(id) {
                self.program.push_edge(StateEdge::new(
                    OpId::new(previous),
                    OpId::new(id),
                    EdgeKind::EffectOrder,
                ));
            }
        }
        self.program.push_op(op);
    }

    pub fn operand(
        &mut self,
        op: u32,
        role: OperandRole,
        target: Option<u32>,
        kind: ValueKind,
        text: &'a str,
    ) {
        let span = self.span_of(op);
        self.program.operands.push(Operand {
            op: OpId::new(op),
            role,
            target: target.map(OpId::new),
            region: None,
            name: None,
            value: OperandValue {
                kind,
                text,
                qualifier: if kind == ValueKind::Opaque {
                    "unparsed"
                } else {
                    ""
                },
                span,
            },
        });
    }

    pub fn attribute(&mut self, op: u32, name: &'a str, value: &'a str) {
        self.operand(op, OperandRole::Attribute, None, LIT, value);
        self.program.operands.last_mut().expect("just pushed").name = Some(name);
    }

    pub fn condition(&mut self, op: u32, region: u32, text: &'a str) {
        self.operand(op, OperandRole::Condition, None, JS, text);
        self.program
            .operands
            .last_mut()
            .expect("just pushed")
            .region = Some(RegionId::new(region));
    }

    pub fn element(&mut self, id: u32, tag: &'a str, region: u32, span: (u32, u32), dynamic: bool) {
        self.op(id, OpKind::InsertNode, region, span, dynamic);
        self.operand(id, OperandRole::Tag, None, LIT, tag);
        self.operand(id, OperandRole::Namespace, None, LIT, "html");
    }

    /// Text is dynamic when it interpolates or when a control region owns it.
    pub fn text(
        &mut self,
        id: u32,
        region: u32,
        span: (u32, u32),
        value: Value<'a>,
        controlled: bool,
    ) {
        let (kind, text) = value;
        self.op(id, OpKind::SetText, region, span, controlled || kind != LIT);
        self.operand(id, OperandRole::Text, None, kind, text);
    }

    pub fn binding(
        &mut self,
        id: u32,
        kind: OpKind,
        target: u32,
        span: (u32, u32),
        parts: Parts<'a>,
    ) {
        let region = self.op_of(target).region.index();
        self.op(id, kind, region, span, true);
        self.operand(id, OperandRole::BindingKind, Some(target), LIT, parts.0);
        self.operand(id, OperandRole::Name, Some(target), LIT, parts.1);
        self.operand(id, OperandRole::Value, Some(target), parts.2, parts.3);
    }

    fn op_of(&self, id: u32) -> Op {
        *self
            .program
            .ops
            .iter()
            .find(|op| op.id == OpId::new(id))
            .expect("op exists")
    }

    fn span_of(&self, id: u32) -> Span {
        self.op_of(id).span
    }

    pub fn record(&mut self, op: u32, alternatives: &[Placement], leader: Option<u32>) {
        let set = alternatives
            .iter()
            .fold(PlacementSet::EMPTY, |set, placement| set.with(*placement));
        self.program.placements.push(PlacementRecord::new(
            OpId::new(op),
            set,
            leader.map(OpId::new),
        ));
    }

    pub fn choose(&mut self, op: u32, chosen: Placement) {
        let record = self
            .program
            .placements
            .iter_mut()
            .find(|record| record.op == OpId::new(op))
            .expect("record exists");
        record.chosen = chosen;
    }
}

/// `(binding kind, name, value kind, value text)`.
pub type Parts<'a> = (&'a str, &'a str, ValueKind, &'a str);
/// `(value kind, text)`.
pub type Value<'a> = (ValueKind, &'a str);

/// The documented fixture, with `text6` as the literal inside the branch.
pub fn fixture<'a>(allocator: &'a Allocator, text6: Value<'a>) -> Build<'a> {
    let mut build = Build::new(allocator, (0, 100));
    build.element(0, "div", 0, (0, 100), false);
    build.binding(1, OpKind::SetEvent, 0, (5, 15), ("on", "click", JS, "save"));
    build.binding(
        2,
        OpKind::SetProp,
        0,
        (16, 25),
        ("bind", "title", JS, "msg"),
    );
    build.region(1, 0, 0, (30, 95));
    build.text(3, 1, (30, 40), (JS, "msg"), false);
    build.op(4, OpKind::If, 1, (40, 80), true);
    build.region(2, 1, 4, (45, 75));
    build.condition(4, 2, "ok");
    build.element(5, "p", 2, (45, 75), true);
    build.region(3, 2, 5, (50, 70));
    build.text(6, 3, (50, 70), text6, true);
    build.text(7, 1, (80, 90), (JS, "msg"), false);
    build
}

pub fn messages(program: &Program<'_>) -> Vec<String> {
    verify(program)
        .into_iter()
        .map(|violation| cstr!("{violation}"))
        .collect()
}
