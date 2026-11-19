//! Intrinsic source preparation and a short two-owner Vue For observation.

use alloc::boxed::Box;
use oxc_parser::{AdmittedExpression, AdmittedParameters};
use vize_l0::{Allocator, SourceBlock, Span};

use super::{ForHead, ForHeadHole, parse_vue_for_head_once};
use crate::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape, SourceError, prepare_attribute_value,
};

mod family;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForInputError {
    ForeignValue,
    Source(SourceError),
}

/// An original representable input that could not enter source preparation.
#[derive(Debug)]
pub struct RejectedForInput<'a> {
    block: SourceBlock<'a>,
    value: &'a str,
    lang: Lang,
    error: ForInputError,
}

impl<'a> RejectedForInput<'a> {
    pub const fn source_block(&self) -> SourceBlock<'a> {
        self.block
    }
    pub const fn authored_value(&self) -> &'a str {
        self.value
    }
    pub const fn lang(&self) -> Lang {
        self.lang
    }
    pub const fn error(&self) -> ForInputError {
        self.error
    }
}

/// This proves physical source/profile origin, not a File or Element receipt.
/// Public roots, Lang selectors and byte equality do not establish File identity.
/// A valid contained subslice can be prepared; a real upper Attribute receiver
/// must compare its complete original value pointer, length and span.
pub struct ForInput<'a> {
    allocator: &'a Allocator,
    origin: ForOrigin<'a>,
    lang: Lang,
    source: EmbedSource<'a>,
}

pub(super) struct ForOrigin<'a> {
    block: SourceBlock<'a>,
    value: &'a str,
    span: Span,
}

impl<'a> ForInput<'a> {
    /// Derive coordinates from the original physical slice, then decode once.
    /// This cannot identify the complete extent of an Attribute from a public
    /// SourceBlock alone. It receives no caller span, decoded map or parse status.
    pub fn attribute_in(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
        value: &'a str,
        lang: Lang,
    ) -> Result<Self, Box<RejectedForInput<'a>>> {
        let rejected = |error| {
            Box::new(RejectedForInput {
                block,
                value,
                lang,
                error,
            })
        };
        let span = block.span_of(value).filter(|span| {
            block.contains_block_span(*span)
                && block
                    .root_source()
                    .get(span.start as usize..span.end as usize)
                    .is_some_and(|raw| same_text(raw, value))
        });
        let span = span.ok_or_else(|| rejected(ForInputError::ForeignValue))?;
        let source = prepare_attribute_value(allocator, block.root_source(), span)
            .map_err(|error| rejected(ForInputError::Source(error)))?;
        Ok(Self {
            allocator,
            origin: ForOrigin { block, value, span },
            lang,
            source,
        })
    }

    /// The existing whole admission/split/two observed parses run exactly once.
    /// Consumption and each projection use only the originally retained arena.
    pub fn observe(self) -> ForHead<'a> {
        let mut head = parse_vue_for_head_once(
            self.allocator,
            Embed {
                grammar: Grammar {
                    lang: self.lang,
                    shape: Shape::ForHead,
                },
                source: self.source,
            },
        );
        head.origin = Some(self.origin);
        head.for_refusal = family::refused(&head);
        head
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForRefusal {
    UnpreparedSource,
    Head(ForHeadHole),
    AliasCount(usize),
    AliasShape,
    CollectionShape,
    Comment,
    EscapedSpelling,
    OuterWhitespace,
    EntityOutput,
    RenderContextName,
    StockProfile,
}

/// Neither a public tuple of projections nor raw roots can construct this borrow.
pub struct AdmittedDenseForHead<'p, 'a> {
    owner: &'p ForHead<'a>,
    origin: &'p ForOrigin<'a>,
    aliases: AdmittedParameters<'p, 'a>,
    collection: AdmittedExpression<'p, 'a>,
    alias_source: EmbedSource<'a>,
    collection_source: EmbedSource<'a>,
}

impl<'a> ForHead<'a> {
    pub fn for_refusal(&self) -> Option<ForRefusal> {
        if self.origin.is_none() {
            Some(ForRefusal::UnpreparedSource)
        } else if let Some(hole) = self.hole {
            Some(ForRefusal::Head(hole))
        } else {
            self.for_refusal
        }
    }

    pub fn admitted_dense(&self) -> Option<AdmittedDenseForHead<'_, 'a>> {
        if self.for_refusal().is_some() {
            return None;
        }
        let origin = self.origin.as_ref()?;
        let aliases = self.aliases()?.ok()?;
        let collection = self.collection()?.ok()?;
        let alias_proof = aliases.admitted_parameters()?;
        let collection_proof = collection.admitted_expression()?;
        if !family::stock_matches(self, &alias_proof, &collection_proof) {
            return None;
        }
        Some(AdmittedDenseForHead {
            owner: self,
            origin,
            aliases: alias_proof,
            collection: collection_proof,
            alias_source: aliases.source(),
            collection_source: collection.source(),
        })
    }
}

impl<'p, 'a> AdmittedDenseForHead<'p, 'a> {
    pub const fn source_block(&self) -> SourceBlock<'a> {
        self.origin.block
    }
    pub const fn authored_value(&self) -> &'a str {
        self.origin.value
    }
    pub const fn authored_value_span(&self) -> Span {
        self.origin.span
    }
    pub const fn source(&self) -> EmbedSource<'a> {
        self.owner.source
    }
    pub const fn aliases(&self) -> &AdmittedParameters<'p, 'a> {
        &self.aliases
    }
    pub const fn collection(&self) -> &AdmittedExpression<'p, 'a> {
        &self.collection
    }
    pub const fn alias_source(&self) -> EmbedSource<'a> {
        self.alias_source
    }
    pub const fn collection_source(&self) -> EmbedSource<'a> {
        self.collection_source
    }
}

fn same_text(first: &str, second: &str) -> bool {
    first.as_ptr() == second.as_ptr() && first.len() == second.len()
}

#[cfg(test)]
mod tests;
