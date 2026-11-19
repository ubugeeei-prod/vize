//! One bounded collection/alias walk over the same whole original source.

use oxc_ast::ast::{BindingPattern, FormalParameter};
use oxc_span::GetSpan;
use vize_l0::Span;
use vize_l1::embed::syntax::ForHeadPart;

use super::{ReferenceSink, ResolutionErrorKind, Resolver};
use crate::resolution::for_head::{
    ForAlias, ForAliasId, ForAliasRole, ForResolutionError, ForResolutionErrorKind,
    OriginalForAlias,
};
use crate::resolution::source::ForReferenceSource;

pub(in crate::resolution) fn original<'a>(
    source: &ForReferenceSource<'_, 'a>,
    sink: &mut impl ReferenceSink<'a>,
) -> Result<(OriginalForAlias<'a>, Option<OriginalForAlias<'a>>), ForResolutionError> {
    let checkpoint = sink.checkpoint();
    let collection_source = source.collection_source().ok_or(ForResolutionError {
        part: ForHeadPart::Collection,
        span: Span::new(0, 0),
        kind: ForResolutionErrorKind::Reference(ResolutionErrorKind::InvalidSpan),
    })?;
    let mut resolver = Resolver::new(collection_source, sink);
    let result = (|| {
        resolver
            .expression(source.collection(), 0)
            .map_err(|error| ForResolutionError {
                part: ForHeadPart::Collection,
                span: error.span,
                kind: ForResolutionErrorKind::Reference(error.kind),
            })?;
        let mut aliases = source.aliases().iter();
        let value = alias(
            &mut resolver,
            source,
            aliases.next().ok_or(ForResolutionError {
                part: ForHeadPart::Aliases,
                span: Span::new(0, 0),
                kind: ForResolutionErrorKind::UnsupportedAlias,
            })?,
            0,
        )?;
        let key = aliases
            .next()
            .map(|parameter| alias(&mut resolver, source, parameter, 1))
            .transpose()?;
        if let Some(extra) = aliases.next() {
            return Err(error(
                source,
                extra.span(),
                ForResolutionErrorKind::UnsupportedAlias,
            ));
        }
        if let Some(key) = key
            .as_ref()
            .filter(|key| key.fact.name() == value.fact.name())
        {
            return Err(ForResolutionError {
                part: ForHeadPart::Aliases,
                span: key.fact.decoded_span(),
                kind: ForResolutionErrorKind::DuplicateAlias,
            });
        }
        Ok((value, key))
    })();
    if result.is_err() {
        resolver.sink.rollback(checkpoint);
    }
    result
}

fn error(
    source: &ForReferenceSource<'_, '_>,
    span: oxc_span::Span,
    kind: ForResolutionErrorKind,
) -> ForResolutionError {
    ForResolutionError {
        part: ForHeadPart::Aliases,
        span: source.alias_decoded_span(span).unwrap_or(Span::new(0, 0)),
        kind,
    }
}

fn alias<'a, S: ReferenceSink<'a>>(
    resolver: &mut Resolver<'a, '_, S>,
    source: &ForReferenceSource<'_, 'a>,
    parameter: &'a FormalParameter<'a>,
    index: u8,
) -> Result<OriginalForAlias<'a>, ForResolutionError> {
    if !resolver.advance(0) {
        return Err(error(
            source,
            parameter.span(),
            ForResolutionErrorKind::Reference(ResolutionErrorKind::TraversalLimit),
        ));
    }
    let BindingPattern::BindingIdentifier(binding) = &parameter.pattern else {
        return Err(error(
            source,
            parameter.span(),
            ForResolutionErrorKind::UnsupportedAlias,
        ));
    };
    let name = binding.name.as_str();
    // The initial semantic family reserves generated helper/context prefixes.
    // Wider authored names need the eventual owning emitter's namespace policy.
    if name.starts_with('_') || name.starts_with('$') {
        return Err(error(
            source,
            binding.span,
            ForResolutionErrorKind::ReservedAlias,
        ));
    }
    let decoded = source.alias_decoded_span(binding.span).map_err(|_| {
        error(
            source,
            binding.span,
            ForResolutionErrorKind::Reference(ResolutionErrorKind::InvalidSpan),
        )
    })?;
    let authored = source.alias_authored_span(binding.span).map_err(|_| {
        error(
            source,
            binding.span,
            ForResolutionErrorKind::Reference(ResolutionErrorKind::InvalidSpan),
        )
    })?;
    Ok(OriginalForAlias {
        parameter,
        fact: ForAlias {
            id: ForAliasId(index),
            name,
            role: if index == 0 {
                ForAliasRole::Value
            } else {
                ForAliasRole::Key
            },
            decoded,
            authored,
        },
    })
}
