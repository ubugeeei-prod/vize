use crate::resolution::{BindingId, Usage};
use vize_l0::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerScopeId(pub(super) u32);
impl HandlerScopeId {
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerLocalId(pub(super) u32);
impl HandlerLocalId {
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Handler-local and enclosing File identities cannot collide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerBindingRef {
    Local(HandlerLocalId),
    Outer(BindingId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerDeclarationKind {
    EventParameter,
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerScope {
    pub id: HandlerScopeId,
    pub parent: Option<HandlerScopeId>,
    /// Decoded-relative block span; the generated handler root has no authored span.
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerBinding<'a> {
    pub id: HandlerLocalId,
    pub scope: HandlerScopeId,
    pub name: &'a str,
    pub kind: HandlerDeclarationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerDeclaration<'a> {
    pub binding: HandlerLocalId,
    /// Actual declaration site, distinct from the scope receiving a hoisted var.
    pub scope: HandlerScopeId,
    pub name: &'a str,
    pub kind: HandlerDeclarationKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerReference<'a> {
    pub scope: HandlerScopeId,
    pub span: Span,
    pub name: &'a str,
    pub binding: HandlerBindingRef,
    pub usage: Usage,
    pub shorthand: bool,
    pub constructor: bool,
}
