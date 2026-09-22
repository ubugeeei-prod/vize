//! Backend admission is a projection, not a boolean promise about a generic S3
//! graph. Only this module can construct the payload consumed by `into_ir`.

mod emit;
pub(super) mod validate;

use vize_atelier_core::JsExpression;
use vize_carton::Allocator;
use vize_s2_to_s3::Lowered;

use super::{AdmissionFailure, retained::Retained};

#[derive(Debug)]
pub(super) struct NativeArtifact<'a> {
    nodes: std::vec::Vec<Node<'a>>,
    /// The template root fragment, in authored order.
    roots: std::vec::Vec<usize>,
}

#[derive(Debug)]
struct Node<'a> {
    content: Content<'a>,
    children: std::vec::Vec<usize>,
    bindings: std::vec::Vec<Binding<'a>>,
}

#[derive(Debug)]
enum Content<'a> {
    Element {
        tag: &'a str,
        /// Static attributes with their authored positions.
        attributes: std::vec::Vec<(&'a str, Option<&'a str>, u32)>,
    },
    Text {
        parts: std::vec::Vec<TextPart<'a>>,
        dynamic: bool,
    },
    /// Authored branch order; each branch body is its own block.
    If { branches: std::vec::Vec<Branch<'a>> },
    /// One loop; `children` is its body (one element, or a template fragment).
    For(Loop<'a>),
    /// A resolved component; `children` is its slot content. `is` is the
    /// `:is` expression of a `<component>`, which is created dynamically.
    Component {
        tag: &'a str,
        props: std::vec::Vec<Prop<'a>>,
        is: Option<Expr<'a>>,
    },
    /// A `<slot>` outlet; `children` is its fallback content.
    Outlet {
        name: &'a str,
        props: std::vec::Vec<Prop<'a>>,
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
    region: vize_s3::op::RegionId,
    /// The branch body in authored order: one element, or a template fragment.
    roots: std::vec::Vec<usize>,
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
}

#[derive(Debug)]
struct Binding<'a> {
    kind: BindingKind,
    /// Prop or event name; empty for the unnamed element directives.
    name: &'a str,
    value: Expr<'a>,
    modifiers: std::vec::Vec<&'a str>,
    /// A static `class` merged ahead of this dynamic `:class`.
    merge: Option<&'a str>,
    /// Authored position, which orders an element's spread sources.
    position: u32,
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
    pub(super) fn into_ir(
        self,
        allocator: &'a Allocator,
        source: &'a str,
        scope_id: Option<&str>,
    ) -> crate::ir::RootIRNode<'a> {
        emit::emit(self, allocator, source, scope_id)
    }
}
