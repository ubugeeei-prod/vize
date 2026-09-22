//! Backend admission is a projection, not a boolean promise about a generic S3
//! graph. Only this module can construct the payload consumed by `into_ir`.

mod emit;
pub(super) mod validate;

use vize_atelier_core::JsExpression;
use vize_carton::{Allocator, Vec};
use vize_s2_to_s3::Lowered;

use super::{AdmissionFailure, retained::Retained};

#[derive(Debug)]
pub(super) struct NativeArtifact<'a> {
    nodes: Vec<'a, Node<'a>>,
    /// The template root fragment, in authored order.
    roots: Vec<'a, usize>,
}

#[derive(Debug)]
struct Node<'a> {
    content: Content<'a>,
    children: Vec<'a, usize>,
    bindings: Vec<'a, Binding<'a>>,
}

/// Authored `[start, end)` byte span of a payload value (P3-9 source maps).
type AuthoredSpan = (u32, u32);

#[derive(Debug)]
enum Content<'a> {
    Element {
        tag: &'a str,
        tag_span: AuthoredSpan,
        /// Name, literal value, and the authored span of that value.
        /// The span start orders a `v-bind` object's sources.
        attributes: Vec<'a, (&'a str, Option<&'a str>, AuthoredSpan)>,
    },
    Text {
        parts: Vec<'a, TextPart<'a>>,
        dynamic: bool,
    },
    /// Authored branch order; each branch body is its own block.
    If { branches: Vec<'a, Branch<'a>> },
    /// One loop; `children` is its body (one element, or a template fragment).
    For(Loop<'a>),
    /// A resolved component; `children` is its slot content. `is` is the
    /// `:is` expression of a `<component>`, which is created dynamically.
    Component {
        tag: &'a str,
        props: Vec<'a, Prop<'a>>,
        is: Option<Expr<'a>>,
    },
    /// A `<slot>` outlet; `children` is its fallback content.
    Outlet {
        name: &'a str,
        props: Vec<'a, Prop<'a>>,
    },
}

/// One component or outlet prop in authored order. Static attributes carry a
/// literal (or no value), bindings an expression, listeners a handler key.
/// A `v-bind`/`v-on` object is the `$` source key (`handler` for `v-on`).
#[derive(Debug, Clone, Copy)]
struct Prop<'a> {
    key: &'a str,
    value: Option<Expr<'a>>,
    dynamic: bool,
    handler: bool,
    position: u32,
}

#[derive(Debug)]
struct Branch<'a> {
    /// `None` only for a trailing unconditional (`v-else`) branch.
    condition: Option<Expr<'a>>,
    /// Authored span of the untrimmed condition value.
    span: AuthoredSpan,
    region: vize_s3::op::RegionId,
    /// The branch body in authored order: one element, or a template fragment.
    roots: Vec<'a, usize>,
}

#[derive(Debug, Clone, Copy)]
struct Loop<'a> {
    source: Expr<'a>,
    value: &'a str,
    key: Option<&'a str>,
    index: Option<&'a str>,
    /// The body element's `:key`, lifted out of its ordinary bindings, or
    /// a template carrier's wrapper key.
    key_prop: Option<Expr<'a>>,
    /// Carried by `<template v-for>`: the body is a fragment.
    template: bool,
    /// Authored spans of the untrimmed source, the value/key/index aliases,
    /// and the `:key` value (P3-9 source maps).
    spans: LoopSpans,
    /// Authored start (`<`) of the carrier element, for source-map units.
    carrier_start: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct LoopSpans {
    source: AuthoredSpan,
    aliases: [Option<AuthoredSpan>; 3],
    key_prop: Option<AuthoredSpan>,
}

/// One admitted operand. A direct reference or static text needs no AST; any
/// other expression carries S2's retained parse for the shared generator.
#[derive(Debug, Clone, Copy)]
struct Expr<'a> {
    text: &'a str,
    js: Option<JsExpression<'a>>,
}

impl<'a> Expr<'a> {
    const fn plain(text: &'a str) -> Self {
        Self { text, js: None }
    }
}

#[derive(Debug, Clone, Copy)]
struct TextPart<'a> {
    value: Expr<'a>,
    dynamic: bool,
    span: AuthoredSpan,
}

#[derive(Debug)]
struct Binding<'a> {
    kind: BindingKind,
    /// Prop or event name; empty for the unnamed element directives.
    name: &'a str,
    value: Expr<'a>,
    modifiers: Vec<'a, &'a str>,
    /// A static `class` merged ahead of this dynamic `:class`.
    merge: Option<&'a str>,
    /// Authored position, which orders an element's spread sources.
    position: u32,
    /// Authored spans of the name and of the untrimmed value.
    spans: [AuthoredSpan; 2],
}

/// The binding families the native projection emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BindingKind {
    Prop,
    Event,
    Show,
    Html,
    Text,
    /// `v-model` on an input: `value` is the model reference, `modifiers` the
    /// `lazy`/`number`/`trim` options.
    Model,
    /// Slot content on a `<template #name>` or its component: `name` is the
    /// slot name, `value` the parameter pattern (empty when there is none).
    Slot,
    /// `v-bind="object"`: the element's props merge in authored order.
    Spread,
    /// `v-on="object"`: listeners bound from an object.
    Handlers,
}

impl<'a> NativeArtifact<'a> {
    pub(super) fn admit(
        s3: &Lowered<'a>,
        retained: &Retained<'_, 'a>,
        loops: &[super::templates::TemplateLoop<'a>],
    ) -> Result<Self, AdmissionFailure> {
        validate::admit(&s3.program, retained, loops)
    }

    /// Consuming the checked projection is the only production generation path
    /// for an accepted artifact. No source parsing or AST lowering occurs here.
    /// With `spans`, payload slices borrowed from `source` keep their authored
    /// spans and the template and control-flow anchors are returned (Davinci
    /// P3-9).
    pub(super) fn into_ir_with_spans(
        self,
        allocator: &'a Allocator,
        source: &'a str,
        scope_id: Option<&str>,
        spans: bool,
    ) -> (
        crate::ir::RootIRNode<'a>,
        Option<crate::generate::spans::VaporSourceSpans>,
    ) {
        emit::emit(self, allocator, source, scope_id, spans)
    }
}
