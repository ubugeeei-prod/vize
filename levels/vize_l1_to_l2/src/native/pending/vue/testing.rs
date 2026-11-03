//! Test-only interruption probes delegate every real native factory operation.

extern crate std;
use crate::native::{ConstructionBody, ConstructionFactory};
use core::cell::Cell;
use vize_l0::{Allocator, Span, id::NodeId};
use vize_l2::{
    artifact::ArtifactError,
    expr::JsExpr,
    op::{Attribute, Namespace},
    provenance::ProvenanceRecord,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Point {
    BeforeSource,
    BetweenExpressions,
    AfterLastExpression,
}

std::thread_local! {
    static POINT: Cell<Option<Point>> = const { Cell::new(None) };
    static CONSTRUCTIONS: Cell<usize> = const { Cell::new(0) };
    static RECORDS: Cell<usize> = const { Cell::new(0) };
    static ARENA_AT_PANIC: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(crate) struct Probe;
impl Probe {
    pub fn arm(point: Point) -> Self {
        POINT.set(Some(point));
        CONSTRUCTIONS.set(0);
        RECORDS.set(0);
        ARENA_AT_PANIC.set(None);
        Self
    }
    pub fn arena_at_panic(&self) -> Option<usize> {
        ARENA_AT_PANIC.get()
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        POINT.set(None);
    }
}

fn interrupt(point: Point, allocator: &Allocator) {
    if POINT.get() == Some(point) {
        ARENA_AT_PANIC.set(Some(allocator.allocated_bytes()));
        panic!("actual native callback interruption");
    }
}

pub(super) struct Factory<'s, 'a, R> {
    inner: &'s mut R,
    allocator: &'a Allocator,
}
impl<'s, 'a, R> Factory<'s, 'a, R> {
    pub fn new(inner: &'s mut R, allocator: &'a Allocator) -> Self {
        Self { inner, allocator }
    }
}
impl<'a, R: ConstructionFactory<'a>> ConstructionFactory<'a> for Factory<'_, 'a, R> {
    fn source(&self) -> &'a str {
        interrupt(Point::BeforeSource, self.allocator);
        self.inner.source()
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.inner.text(text, span)
    }
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.inner.comment(text, span)
    }
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        CONSTRUCTIONS.set(CONSTRUCTIONS.get() + 1);
        if CONSTRUCTIONS.get() == 2 {
            interrupt(Point::BetweenExpressions, self.allocator);
        }
        self.inner.interpolation(expression, span)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.bind(name, name_span, expression, span)
    }
    fn element<B: ConstructionBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.element(
            tag,
            namespace,
            attributes,
            span,
            Body {
                body,
                allocator: self.allocator,
            },
        )
    }
    fn component<B: ConstructionBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.inner.component(
            name,
            attributes,
            span,
            Body {
                body,
                allocator: self.allocator,
            },
        )
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        if record.rule.as_str() == "native.interpolation" {
            RECORDS.set(RECORDS.get() + 1);
            if RECORDS.get() == 2 {
                interrupt(Point::AfterLastExpression, self.allocator);
            }
        }
        self.inner.record(record)
    }
}

struct Body<'a, B> {
    body: B,
    allocator: &'a Allocator,
}
impl<'a, B: ConstructionBody<'a>> ConstructionBody<'a> for Body<'a, B> {
    fn run<R: ConstructionFactory<'a>>(self, child: &mut R, node: NodeId) {
        self.body
            .run(&mut Factory::new(child, self.allocator), node);
    }
}
