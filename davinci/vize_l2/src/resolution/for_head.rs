//! Bounded original For semantics; File and canonical body attachment are separate.

use alloc::boxed::Box;
use vize_l0::Span;
use vize_l1::embed::syntax::ForHeadPart;

use super::{BindingId, BindingLookup, Occurrence, ResolutionErrorKind, walk};
use crate::lang::js::NativeForInput;

mod facts;
mod sink;
pub use facts::{ForAlias, ForAliasId, ForAliasRole, ForResolvedBinding};

/// A decoded-relative error in one of the two original source namespaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForResolutionError {
    pub part: ForHeadPart,
    pub span: Span,
    pub kind: ForResolutionErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForResolutionErrorKind {
    Reference(ResolutionErrorKind),
    UnsupportedAlias,
    DuplicateAlias,
    ReservedAlias,
}

/// No partial facts escape; the whole original Attribute and syntax stay owned.
#[derive(Debug)]
pub struct RejectedForResolution<'a> {
    input: NativeForInput<'a>,
    pub error: ForResolutionError,
}

impl<'a> RejectedForResolution<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeForInput<'a> {
        &self.input
    }
    #[must_use]
    pub fn into_input(self) -> NativeForInput<'a> {
        self.input
    }
}

/// Complete local For facts tied to the genuine whole original input.
/// Local aliases and enclosing binding identities stay in separate namespaces.
/// This establishes neither a File child scope nor a canonical For/body.
///
/// Caller-written facts cannot replace the original source owner:
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l2::lang::js::NativeForInput;
/// use vize_l2::resolution::{ForAlias, ForResolution, Occurrence};
/// fn substitute<'a>(input: NativeForInput<'a>, collection: Occurrence<'a>,
///     collection_authored: Span, value: ForAlias<'a>, key: Option<ForAlias<'a>>) {
///     let _ = ForResolution { input, collection, collection_authored, value, key };
/// }
/// ```
/// The whole normal owner is not cloneable:
/// ```compile_fail
/// use vize_l2::resolution::ForResolution;
/// fn duplicate(table: ForResolution<'_>) { let _ = table.clone(); }
/// ```
#[derive(Debug)]
pub struct ForResolution<'a> {
    input: NativeForInput<'a>,
    collection: Occurrence<'a>,
    collection_authored: Span,
    value: ForAlias<'a>,
    key: Option<ForAlias<'a>>,
}

impl<'a> ForResolution<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeForInput<'a> {
        &self.input
    }
    #[must_use]
    pub const fn collection(&self) -> Occurrence<'a> {
        self.collection
    }
    #[must_use]
    pub const fn collection_authored_span(&self) -> Span {
        self.collection_authored
    }
    #[must_use]
    pub const fn value(&self) -> ForAlias<'a> {
        self.value
    }
    #[must_use]
    pub const fn key(&self) -> Option<ForAlias<'a>> {
        self.key
    }
    /// The initial local environment shadows its parent only after collection resolution.
    /// Enclosing identities remain the caller's contract, without File admission.
    pub fn lookup_body(
        &self,
        name: &str,
        enclosing: &impl BindingLookup,
    ) -> Option<ForResolvedBinding> {
        if self.value.name() == name {
            return Some(ForResolvedBinding::Local(self.value.id()));
        }
        if let Some(key) = self.key.filter(|key| key.name() == name) {
            return Some(ForResolvedBinding::Local(key.id()));
        }
        enclosing.lookup(name).map(ForResolvedBinding::Enclosing)
    }
    #[must_use]
    pub fn into_input(self) -> NativeForInput<'a> {
        self.input
    }
}

/// Resolve the original collection in the enclosing environment, then derive
/// one/two actual alias declarations in the original Params observation.
/// One retained-tree resolver budget covers this whole bounded head; no source
/// decode, parse, AST pre-scan or body walk is introduced.
pub fn resolve_for_head<'a>(
    input: NativeForInput<'a>,
    enclosing: &impl BindingLookup,
) -> Result<ForResolution<'a>, Box<RejectedForResolution<'a>>> {
    let result = resolve(&input, enclosing);
    match result {
        Ok((collection, collection_authored, value, key)) => Ok(ForResolution {
            input,
            collection,
            collection_authored,
            value,
            key,
        }),
        Err(error) => Err(Box::new(RejectedForResolution { input, error })),
    }
}

type Facts<'a> = (Occurrence<'a>, Span, ForAlias<'a>, Option<ForAlias<'a>>);

fn resolve<'a>(
    input: &NativeForInput<'a>,
    enclosing: &impl BindingLookup,
) -> Result<Facts<'a>, ForResolutionError> {
    let source = input.references().ok_or(ForResolutionError {
        part: ForHeadPart::Collection,
        span: Span::new(0, 0),
        kind: ForResolutionErrorKind::Reference(ResolutionErrorKind::InvalidSpan),
    })?;
    let mut sink = sink::Collection::new(enclosing);
    let (value, key) = walk::original_for_head(&source, &mut sink)?;
    let collection = sink.finish()?;
    let collection_authored = source
        .collection_authored_span(source.collection().span())
        .map_err(|_| ForResolutionError {
            part: ForHeadPart::Collection,
            span: collection.span,
            kind: ForResolutionErrorKind::Reference(ResolutionErrorKind::InvalidSpan),
        })?;
    Ok((collection, collection_authored, value, key))
}

use oxc_span::GetSpan;

#[cfg(test)]
mod tests;
