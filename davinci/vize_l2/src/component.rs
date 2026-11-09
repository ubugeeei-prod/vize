//! A component definition: the unit a lowered file presents to every product.
//!
//! An SFC lowers to one definition and a JSX/TSX module to one per outermost
//! render root, in authored order. Products read these without asking which
//! input produced them.

use vize_l0::Span;

use crate::artifact::ArtifactParts;

/// How a component's render code runs. The execution model is semantics, not
/// only a target: it fixes whether a value computed before the render stays
/// fresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exec {
    /// The render function re-runs on every update (VDOM).
    ReRender,
    /// Setup runs once; each dynamic expression is its own effect (Vapor).
    RunOnce,
}

/// Where a free identifier in the render body resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderScope {
    /// The enclosing JavaScript scope, as in a JSX render body.
    Lexical,
    /// The component instance and its setup bindings, as in a template.
    Instance,
}

/// One component and its lowered render body.
#[derive(Debug)]
pub struct ComponentDef<'a> {
    /// The authored component name, when the lowering resolved one.
    pub name: Option<&'a str>,
    pub exec: Exec,
    pub scope: RenderScope,
    /// The authored range of the render root.
    pub span: Span,
    pub body: ArtifactParts<'a>,
    /// Ops and attached bindings minted while building `body`.
    pub op_count: u32,
}
