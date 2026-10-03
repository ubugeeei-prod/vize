//! Borrowed facts let the genuine File retain its normal input across callbacks.

use alloc::boxed::Box;
use oxc_ast::ast::{Expression, FormalParameter};
use vize_l0::Span;
use vize_l1::embed::{EmbedSource, Grammar, syntax::ForHeadPart};

use super::{Facts, ForResolution, ForResolutionError, ForResolutionErrorKind, resolve};
use crate::lang::js::NativeForInput;
use crate::resolution::{BindingLookup, ResolutionErrorKind};

/// Private and move-only: caller facts/root tuples cannot construct this packet.
/// Its consuming join independently checks the whole original source and roots.
#[derive(Debug)]
pub(crate) struct ResolvedForFacts<'a> {
    aliases: &'a [FormalParameter<'a>],
    collection: &'a Expression<'a>,
    sources: [EmbedSource<'a>; 3],
    grammar: Grammar,
    name_span: Span,
    parts: Facts<'a>,
}

impl<'a> ResolvedForFacts<'a> {
    pub(crate) fn join(
        self,
        input: NativeForInput<'a>,
    ) -> Result<ForResolution<'a>, Box<NativeForInput<'a>>> {
        let Some(sources) = sources(&input) else {
            return Err(Box::new(input));
        };
        if !core::ptr::eq(self.aliases, input.aliases())
            || !core::ptr::eq(self.collection, input.collection())
            || self.grammar != input.operand().syntax().grammar()
            || self.name_span != input.operand().name_span()
            || !self
                .sources
                .into_iter()
                .zip(sources)
                .all(|(left, right)| same_source(left, right))
        {
            return Err(Box::new(input));
        }
        let (collection, collection_authored, value, key) = self.parts;
        Ok(ForResolution {
            input,
            collection,
            collection_authored,
            value: value.fact,
            key: key.as_ref().map(|key| key.fact),
            value_parameter: value.parameter,
            key_parameter: key.map(|key| key.parameter),
        })
    }
}

pub(crate) fn resolve_for_facts<'a>(
    input: &NativeForInput<'a>,
    enclosing: &impl BindingLookup,
) -> Result<ResolvedForFacts<'a>, ForResolutionError> {
    let parts = resolve(input, enclosing)?;
    Ok(ResolvedForFacts {
        aliases: input.aliases(),
        collection: input.collection(),
        sources: sources(input).ok_or_else(invalid_source)?,
        grammar: input.operand().syntax().grammar(),
        name_span: input.operand().name_span(),
        parts,
    })
}

fn sources<'a>(input: &NativeForInput<'a>) -> Option<[EmbedSource<'a>; 3]> {
    let syntax = input.operand().syntax();
    Some([
        syntax.source(),
        syntax.aliases()?.ok()?.source(),
        syntax.collection()?.ok()?.source(),
    ])
}

fn same_source(left: EmbedSource<'_>, right: EmbedSource<'_>) -> bool {
    core::ptr::eq(left.authored_root(), right.authored_root())
        && core::ptr::eq(left.text(), right.text())
        && left.span() == right.span()
        && match (left.decode_map(), right.decode_map()) {
            (None, None) => true,
            (Some(left), Some(right)) => core::ptr::eq(left.segments(), right.segments()),
            _ => false,
        }
}

pub(super) fn invalid_source() -> ForResolutionError {
    ForResolutionError {
        part: ForHeadPart::Collection,
        span: Span::new(0, 0),
        kind: ForResolutionErrorKind::Reference(ResolutionErrorKind::InvalidSpan),
    }
}
