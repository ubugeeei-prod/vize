use crate::resolution::{BindingId, Usage};
use vize_l0::Span;

/// Statement facts recorded during the same whole original handler walk.
/// They confer no Vue directive or runtime policy independently of that owner.
///
/// Callers cannot manufacture original-body observations:
/// ```compile_fail
/// use vize_l2::resolution::HandlerSyntax;
/// let _ = HandlerSyntax { leading_declaration: true, first_return: None };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerSyntax {
    pub(super) leading_declaration: bool,
    pub(super) first_return: Option<Span>,
}

impl HandlerSyntax {
    #[must_use]
    pub const fn leading_declaration(self) -> bool {
        self.leading_declaration
    }
    /// Decoded-relative original first return statement, including nested blocks.
    #[must_use]
    pub const fn first_return(self) -> Option<Span> {
        self.first_return
    }
}

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
