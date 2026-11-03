//! Queries over sealed attached Handler/OriginalFor rows, never bare coordinates.

use super::{BindingRef, FileArtifact, PositionQueryError};
use crate::file::{FileForHead, FileHandler, ScopeId};
use crate::resolution::{HandlerBinding, HandlerDeclaration, HandlerScopeId, Usage};
use vize_l0::Span;

mod sites;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateQueryError {
    Position(PositionQueryError),
    OriginalMembership,
    Projection,
    ForeignSymbol,
}

/// A local identity retains the whole actual handler, not a File BindingId.
/// The implicit `$event` has a binding but no authored declaration site.
///
/// ```compile_fail
/// use vize_l2::file::{FileHandler, HandlerLocalRef};
/// use vize_l2::resolution::HandlerBinding;
/// fn forge<'f, 'a>(handler: FileHandler<'f, 'a>, binding: &'f HandlerBinding<'a>) {
///     let _ = HandlerLocalRef { handler, binding };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct HandlerLocalRef<'f, 'a> {
    handler: FileHandler<'f, 'a>,
    binding: &'f HandlerBinding<'a>,
}

impl<'f, 'a> HandlerLocalRef<'f, 'a> {
    #[must_use]
    pub const fn handler(self) -> FileHandler<'f, 'a> {
        self.handler
    }
    #[must_use]
    pub const fn binding(self) -> &'f HandlerBinding<'a> {
        self.binding
    }
    /// Actual declaration sites; a hoisted var can have several original sites.
    pub fn declarations(self) -> impl Iterator<Item = &'f HandlerDeclaration<'a>> {
        self.handler
            .resolution()
            .into_iter()
            .flat_map(|resolution| resolution.declarations())
            .filter(move |row| row.binding == self.binding.id)
    }
    #[must_use]
    pub fn same_symbol(self, other: HandlerLocalRef<'_, '_>) -> bool {
        self.handler.same_owner(other.handler)
            && self.handler.id() == other.handler.id()
            && core::ptr::eq(self.binding, other.binding)
    }
}

/// File bindings (including real template aliases) and handler locals stay distinct.
#[derive(Clone, Copy)]
pub enum TemplateSymbolRef<'f, 'a> {
    File(BindingRef<'f, 'a>),
    HandlerLocal(HandlerLocalRef<'f, 'a>),
}

impl<'f, 'a> TemplateSymbolRef<'f, 'a> {
    #[must_use]
    pub fn file(self) -> &'f FileArtifact<'a> {
        match self {
            Self::File(binding) => binding.file(),
            Self::HandlerLocal(local) => local.handler.file(),
        }
    }
    #[must_use]
    pub fn same_symbol(self, other: TemplateSymbolRef<'_, '_>) -> bool {
        match (self, other) {
            (Self::File(left), TemplateSymbolRef::File(right)) => {
                left.same_owner(right) && left.id() == right.id()
            }
            (Self::HandlerLocal(left), TemplateSymbolRef::HandlerLocal(right)) => {
                left.same_symbol(right)
            }
            _ => false,
        }
    }
}

/// Recorded site scope; handler local scope numbers never become File ScopeIds.
#[derive(Clone, Copy)]
pub enum TemplateSiteScope<'f, 'a> {
    File(ScopeId),
    Handler(FileHandler<'f, 'a>, HandlerScopeId),
}

#[derive(Clone, Copy)]
enum Origin<'f, 'a> {
    Handler(FileHandler<'f, 'a>),
    For(FileForHead<'f, 'a>),
}

/// One original declaration/use, borrowed with its actual immutable File owner.
///
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// fn discard(file: FileArtifact<'_>, offset: u32) {
///     let site = file.template_symbol_at_offset(offset).unwrap().unwrap();
///     drop(file);
///     let _ = site.symbol();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct TemplateSiteRef<'f, 'a> {
    symbol: TemplateSymbolRef<'f, 'a>,
    span: Span,
    scope: TemplateSiteScope<'f, 'a>,
    usage: Option<Usage>,
    origin: Origin<'f, 'a>,
}

impl<'f, 'a> TemplateSiteRef<'f, 'a> {
    #[must_use]
    pub fn file(self) -> &'f FileArtifact<'a> {
        self.symbol.file()
    }
    #[must_use]
    pub const fn symbol(self) -> TemplateSymbolRef<'f, 'a> {
        self.symbol
    }
    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }
    #[must_use]
    pub const fn scope(self) -> TemplateSiteScope<'f, 'a> {
        self.scope
    }
    /// None means an authored declaration; use classifications are original facts.
    #[must_use]
    pub const fn usage(self) -> Option<Usage> {
        self.usage
    }
    #[must_use]
    pub fn handler(self) -> Option<FileHandler<'f, 'a>> {
        match self.origin {
            Origin::Handler(handler) => Some(handler),
            Origin::For(_) => None,
        }
    }
    #[must_use]
    pub fn for_head(self) -> Option<FileForHead<'f, 'a>> {
        match self.origin {
            Origin::For(head) => Some(head),
            Origin::Handler(_) => None,
        }
    }
}

impl<'a> FileArtifact<'a> {
    /// Query original attached event-body and For collection/alias sites only.
    /// Script and generic expression queries remain their separate actual APIs.
    /// UTF-8 names are half-open. EOF, opaque text and the implicit event
    /// declaration return None; any overlapping observations refuse.
    pub fn template_symbol_at_offset(
        &self,
        offset: u32,
    ) -> Result<Option<TemplateSiteRef<'_, 'a>>, TemplateQueryError> {
        self.check_template_offset(offset)?;
        let mut found = None;
        let mut ambiguous = false;
        sites::visit(self, |site| {
            if site.span.start <= offset && offset < site.span.end {
                ambiguous |= found.is_some();
                found = Some(site);
            }
        })?;
        if ambiguous {
            return Err(TemplateQueryError::Position(
                PositionQueryError::AmbiguousSite,
            ));
        }
        Ok(found)
    }

    /// Visit uses in these attached template families without parsing/allocation.
    /// Order is unspecified. A late refusal invalidates all previously visited
    /// work; callers must discard it before publishing a response.
    pub fn for_each_template_reference_to<'f>(
        &'f self,
        symbol: TemplateSymbolRef<'_, '_>,
        mut visit: impl FnMut(TemplateSiteRef<'f, 'a>),
    ) -> Result<(), TemplateQueryError> {
        if !core::ptr::eq(self, symbol.file()) {
            return Err(TemplateQueryError::ForeignSymbol);
        }
        self.check_template_offset(0)?;
        sites::visit(self, |site| {
            if site.usage.is_some() && site.symbol.same_symbol(symbol) {
                visit(site);
            }
        })
    }

    fn check_template_offset(&self, offset: u32) -> Result<(), TemplateQueryError> {
        let error = if !self.is_complete() {
            Some(PositionQueryError::IncompleteFile)
        } else if offset as usize > self.artifact().source().len() {
            Some(PositionQueryError::OutOfBounds)
        } else if !self.artifact().source().is_char_boundary(offset as usize) {
            Some(PositionQueryError::NotCharBoundary)
        } else {
            None
        };
        error.map_or(Ok(()), |error| Err(TemplateQueryError::Position(error)))
    }
}

#[cfg(test)]
mod tests;
