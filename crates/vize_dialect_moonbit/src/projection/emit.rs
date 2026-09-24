//! The projection emitter: one walk over the S2 op tree writing the
//! virtual file, its span links and its positions (see [`super`]).

use vize_s0::{Allocator, SourceBlock, Span, append};
use vize_s1::SurfaceChild;
use vize_s2::expr::capability::ExprDialect;
use vize_s2::expr::{ExprRef, ForeignExpr, OpaqueReason};
use vize_s2::op::{BindingOp, DynamicName, ForOp, IfOp, Op};

use super::{HELPERS, Position, PositionKind, Projection, Role, SpanLink, Unsupported};
use crate::dialect::{MoonBitDialect, is_handler_path};
use crate::sfc::DIALECT;

pub(super) fn generated_offset(text: &str) -> u32 {
    // A projection is a few times its SFC, far inside the u32 offset space;
    // saturate rather than wrap if that ever stopped holding.
    u32::try_from(text.len()).unwrap_or(u32::MAX)
}

/// An expression's authored text and file-absolute span.
type Piece<'a> = (&'a str, Span);

fn piece(expr: ExprRef<'_>) -> Piece<'_> {
    (expr.source(), expr.span())
}

/// Every S1 interpolation's trimmed content, file-absolute, in document
/// order.
pub(super) fn interpolation_parts<'a>(
    children: &[SurfaceChild<'a>],
    block: SourceBlock<'a>,
    out: &mut Vec<Piece<'a>>,
) {
    for child in children {
        match child {
            SurfaceChild::Element(element) => interpolation_parts(&element.children, block, out),
            SurfaceChild::Interpolation(node) => {
                let text = node.content.text.trim();
                if let Some(span) = block.span_of(text) {
                    out.push((text, span));
                }
            }
            _ => {}
        }
    }
}

pub(super) struct Emitter<'a> {
    pub(super) allocator: &'a Allocator,
    pub(super) parts: Vec<Piece<'a>>,
    pub(super) projection: Projection<'a>,
    pub(super) used: [bool; HELPERS.len()],
    pub(super) depth: usize,
}

impl<'a> Emitter<'a> {
    pub(super) fn region(&mut self, ops: &[Op<'a>]) {
        for op in ops {
            self.op(op);
        }
    }

    fn op(&mut self, op: &Op<'a>) {
        match op {
            Op::Element(element) => {
                self.bindings(&element.bindings);
                self.region(&element.children.ops);
            }
            Op::Component(component) => {
                self.bindings(&component.bindings);
                self.region(&component.children.ops);
            }
            Op::Text(_) | Op::Comment(_) => {}
            Op::Interpolation(interpolation) => match interpolation.expression {
                // A merged `{{ a }} text` run: the lowering kept only the
                // rebuilt text, so the parts come from S1 (P6-5 finding).
                ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
                    let span = interpolation.span;
                    let parts = self.parts.iter().copied();
                    let inside: Vec<_> = parts
                        .filter(|(_, at)| span.start <= at.start && at.end <= span.end)
                        .collect();
                    for part in inside {
                        self.demand("__vize_show", PositionKind::Interpolation, part);
                    }
                }
                expression => {
                    self.demand(
                        "__vize_show",
                        PositionKind::Interpolation,
                        piece(expression),
                    );
                }
            },
            Op::If(if_op) => self.if_chain(if_op),
            Op::For(for_op) => self.for_loop(for_op),
            Op::Slot(slot) => {
                if let DynamicName::Dynamic(name) = &slot.name {
                    self.demand("__vize_name", PositionKind::Argument, piece(*name));
                }
                self.bindings(&slot.bindings);
                self.region(&slot.fallback.ops);
            }
        }
    }

    fn bindings(&mut self, bindings: &[BindingOp<'a>]) {
        for binding in bindings {
            match binding {
                BindingOp::Bind(bind) => {
                    self.argument(bind.name.as_ref());
                    if let Some(value) = bind.value {
                        self.demand("__vize_bind", PositionKind::Bind, piece(value));
                    }
                }
                BindingOp::On(on) => {
                    self.argument(on.name.as_ref());
                    if let Some(handler) = on.handler {
                        self.handler(handler);
                    }
                }
                BindingOp::VueShow(show) => {
                    self.demand("__vize_cond", PositionKind::Show, piece(show.value));
                }
                BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => {}
                other => self.projection.unsupported.push(Unsupported {
                    what: other.mnemonic(),
                    span: binding_span(other),
                }),
            }
        }
    }

    fn argument(&mut self, name: Option<&DynamicName<'a>>) {
        if let Some(DynamicName::Dynamic(name)) = name {
            self.demand("__vize_name", PositionKind::Argument, piece(*name));
        }
    }

    fn handler(&mut self, handler: ExprRef<'a>) {
        if is_handler_path(handler.source()) {
            self.demand("__vize_on", PositionKind::Handler, piece(handler));
            return;
        }
        let start = self.line_start();
        let first = self.projection.positions.len();
        self.use_helper("__vize_on");
        self.projection.text.push_str("__vize_on(fn() { ");
        self.expr(PositionKind::HandlerStatement, piece(handler), start);
        self.projection.text.push_str(" })\n");
        self.close_statement(first);
    }

    fn if_chain(&mut self, if_op: &IfOp<'a>) {
        for (index, branch) in if_op.branches.iter().enumerate() {
            let start = self.line_start();
            let first = self.projection.positions.len();
            if index > 0 {
                self.projection.text.push_str("} else ");
            }
            match branch.condition {
                Some(condition) => {
                    self.projection.text.push_str("if ");
                    self.expr(PositionKind::Condition, piece(condition), start);
                    self.projection.text.push_str(" {\n");
                }
                None => self.projection.text.push_str("{\n"),
            }
            self.close_statement(first);
            self.nested(|this| this.region(&branch.region.ops));
        }
        self.line_start();
        self.projection.text.push_str("}\n");
    }

    fn for_loop(&mut self, for_op: &ForOp<'a>) {
        let binding = for_op.binding;
        if let Some(index) = binding.index {
            self.projection.unsupported.push(Unsupported {
                what: "ui.for (third alias)",
                span: index.span(),
            });
        }
        let start = self.line_start();
        let first = self.projection.positions.len();
        self.projection.text.push_str("for ");
        if let Some(key) = binding.key {
            self.expr(PositionKind::IterKey, piece(key), start);
            self.projection.text.push_str(", ");
        }
        self.expr(PositionKind::IterValue, piece(binding.value), start);
        self.projection.text.push_str(" in ");
        self.expr(PositionKind::IterSource, piece(binding.source), start);
        self.projection.text.push_str(" {\n");
        self.close_statement(first);
        self.nested(|this| this.region(&for_op.region.ops));
        self.line_start();
        self.projection.text.push_str("}\n");
    }

    /// One `helper(expr)` statement line.
    fn demand(&mut self, helper: &'static str, kind: PositionKind, expr: Piece<'a>) {
        let start = self.line_start();
        let first = self.projection.positions.len();
        self.use_helper(helper);
        append!(self.projection.text, "{helper}(");
        self.expr(kind, expr, start);
        self.projection.text.push_str(")\n");
        self.close_statement(first);
    }

    /// Emit one expression through the dialect, recording its position and
    /// link; `statement_start` opens the position's statement range, which
    /// [`Self::close_statement`] closes.
    fn expr(&mut self, kind: PositionKind, (source, span): Piece<'a>, statement_start: u32) {
        let foreign: &'a ForeignExpr<'a> = self.allocator.alloc(ForeignExpr {
            dialect: DIALECT,
            source,
            span,
            facts: vize_s0::Vec::new_in(&self.allocator),
        });
        let start = generated_offset(&self.projection.text);
        // The dialect emits its own payload verbatim into a `String`, which
        // cannot fail.
        let _ = MoonBitDialect.emit(ExprRef::Foreign(foreign), &mut self.projection.text);
        let end = generated_offset(&self.projection.text);
        let index = self.projection.positions.len();
        self.projection.links.push(SpanLink {
            generated: Span::new(start, end),
            source: foreign.span,
            role: Role::Expression(index),
        });
        self.projection.positions.push(Position {
            kind,
            expr: foreign,
            statement: Span::new(statement_start, end),
        });
    }

    /// Close the statement ranges of the positions from `first` on at the
    /// current end of the text.
    fn close_statement(&mut self, first: usize) {
        let end = generated_offset(&self.projection.text);
        for position in self.projection.positions.iter_mut().skip(first) {
            position.statement.end = end;
        }
    }

    fn line_start(&mut self) -> u32 {
        for _ in 0..self.depth {
            self.projection.text.push_str("  ");
        }
        generated_offset(&self.projection.text)
    }

    fn nested(&mut self, body: impl FnOnce(&mut Self)) {
        self.depth += 1;
        body(self);
        self.depth -= 1;
    }

    fn use_helper(&mut self, helper: &str) {
        if let Some(at) = HELPERS.iter().position(|(name, _)| *name == helper)
            && let Some(used) = self.used.get_mut(at)
        {
            *used = true;
        }
    }
}

fn binding_span(binding: &BindingOp<'_>) -> Span {
    match binding {
        BindingOp::Bind(op) => op.span,
        BindingOp::On(op) => op.span,
        BindingOp::Model(op) => op.span,
        BindingOp::SlotContent(op) => op.span,
        BindingOp::VueDirective(op) => op.span,
        BindingOp::VueCssBind(op) => op.span,
        BindingOp::VueSync(op) => op.span,
        BindingOp::VueSlotScope(op) => op.span,
        BindingOp::VueOnce(op) => op.span,
        BindingOp::VueMemo(op) => op.span,
        BindingOp::VueShow(op) => op.span,
        BindingOp::VueHtml(op) => op.span,
        BindingOp::VueText(op) => op.span,
        BindingOp::VueCloak(op) => op.span,
    }
}
