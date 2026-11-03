use super::BindingId;
use vize_l0::Span;

/// Dense identity local to one genuine For resolution, never a File BindingId.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForAliasId(pub(in crate::resolution) u8);
impl ForAliasId {
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0 as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForAliasRole {
    Value,
    Key,
}

/// Only the owning original Params walk mints a declaration fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForAlias<'a> {
    pub(in crate::resolution) id: ForAliasId,
    pub(in crate::resolution) name: &'a str,
    pub(in crate::resolution) role: ForAliasRole,
    pub(in crate::resolution) decoded: Span,
    pub(in crate::resolution) authored: Span,
}
impl<'a> ForAlias<'a> {
    #[must_use]
    pub const fn id(self) -> ForAliasId {
        self.id
    }
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.name
    }
    #[must_use]
    pub const fn role(self) -> ForAliasRole {
        self.role
    }
    #[must_use]
    pub const fn decoded_span(self) -> Span {
        self.decoded
    }
    #[must_use]
    pub const fn authored_span(self) -> Span {
        self.authored
    }
}

/// Local and enclosing IDs cannot accidentally alias one another numerically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForResolvedBinding {
    Local(ForAliasId),
    Enclosing(BindingId),
}
