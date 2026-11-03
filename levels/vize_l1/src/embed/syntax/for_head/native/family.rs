//! Bounded native spelling checks over the already retained roots and comments.

use oxc_ast::ast::{BindingPattern, Expression};
use oxc_parser::{AdmittedExpression, AdmittedParameters, ParseOptions};
use vize_l0::Span;

use super::{NativeForHead, NativeForRefusal, same_text};
use crate::embed::syntax::ProgramOptions;

pub(super) fn refused(head: &NativeForHead<'_>) -> Option<NativeForRefusal> {
    if let Some(hole) = head.hole {
        return Some(NativeForRefusal::Head(hole));
    }
    check(head).err()
}

fn check(head: &NativeForHead<'_>) -> Result<(), NativeForRefusal> {
    let origin = head
        .origin
        .as_ref()
        .ok_or(NativeForRefusal::UnpreparedSource)?;
    let outer_whitespace = |ch: char| ch.is_whitespace() || ch == '\u{feff}';
    if origin.value.chars().next().is_some_and(outer_whitespace)
        || origin
            .value
            .chars()
            .next_back()
            .is_some_and(outer_whitespace)
    {
        return Err(NativeForRefusal::OuterWhitespace);
    }
    if head.source.decode_map().is_some() {
        return Err(NativeForRefusal::EntityOutput);
    }
    let aliases = head
        .aliases()
        .and_then(Result::ok)
        .ok_or(NativeForRefusal::StockProfile)?;
    let collection = head
        .collection()
        .and_then(Result::ok)
        .ok_or(NativeForRefusal::StockProfile)?;
    let parameters = aliases
        .admitted_parameters()
        .ok_or(NativeForRefusal::StockProfile)?;
    let expression = collection
        .admitted_expression()
        .ok_or(NativeForRefusal::StockProfile)?;
    if !stock_matches(head, &parameters, &expression) {
        return Err(NativeForRefusal::StockProfile);
    }
    if aliases.comments().next().is_some() || collection.comments().next().is_some() {
        return Err(NativeForRefusal::Comment);
    }
    let formals = parameters.parameters();
    let count = formals.items.len();
    if !(1..=2).contains(&count) {
        return Err(NativeForRefusal::AliasCount(count));
    }
    if formals.rest.is_some() {
        return Err(NativeForRefusal::AliasShape);
    }
    let first = formals.items.first().ok_or(NativeForRefusal::AliasShape)?;
    let last = formals.items.last().ok_or(NativeForRefusal::AliasShape)?;
    let first_span = aliases
        .decoded_span(first.span)
        .map_err(|_| NativeForRefusal::AliasShape)?;
    let last_span = aliases
        .decoded_span(last.span)
        .map_err(|_| NativeForRefusal::AliasShape)?;
    if first_span.start != 0 || last_span.end as usize != aliases.source().text().len() {
        return Err(NativeForRefusal::AliasShape);
    }
    for formal in &formals.items {
        let BindingPattern::BindingIdentifier(identifier) = &formal.pattern else {
            return Err(NativeForRefusal::AliasShape);
        };
        if formal.span != identifier.span
            || formal.initializer.is_some()
            || formal.type_annotation.is_some()
        {
            return Err(NativeForRefusal::AliasShape);
        }
        let span = aliases
            .decoded_span(identifier.span)
            .map_err(|_| NativeForRefusal::AliasShape)?;
        let spelling = aliases
            .source()
            .text()
            .get(span.start as usize..span.end as usize)
            .ok_or(NativeForRefusal::AliasShape)?;
        if spelling != identifier.name.as_str() {
            return Err(NativeForRefusal::EscapedSpelling);
        }
        if identifier.name.as_str() == "_ctx" {
            return Err(NativeForRefusal::RenderContextName);
        }
    }
    let Expression::Identifier(identifier) = expression.expression() else {
        return Err(NativeForRefusal::CollectionShape);
    };
    let span = collection
        .decoded_span(identifier.span)
        .map_err(|_| NativeForRefusal::CollectionShape)?;
    if span
        != Span::new(
            0,
            u32::try_from(collection.source().text().len())
                .map_err(|_| NativeForRefusal::CollectionShape)?,
        )
    {
        return Err(NativeForRefusal::CollectionShape);
    }
    if collection.source().text() != identifier.name.as_str() {
        return Err(NativeForRefusal::EscapedSpelling);
    }
    if identifier.name.as_str() == "_ctx" {
        return Err(NativeForRefusal::RenderContextName);
    }
    Ok(())
}

pub(super) fn stock_matches(
    head: &NativeForHead<'_>,
    aliases: &AdmittedParameters<'_, '_>,
    collection: &AdmittedExpression<'_, '_>,
) -> bool {
    let Some(Ok(alias_owner)) = head.aliases() else {
        return false;
    };
    let Some(Ok(collection_owner)) = head.collection() else {
        return false;
    };
    let expected = ProgramOptions::module(head.grammar.lang).source_type();
    aliases.source_type() == expected
        && collection.source_type() == expected
        && aliases.options() == ParseOptions::default()
        && collection.options() == ParseOptions::default()
        && same_text(aliases.content(), alias_owner.source().text())
        && same_text(collection.content(), collection_owner.source().text())
        && window_matches(
            aliases.parser_content_span(),
            alias_owner.parser_prefix(),
            aliases.content(),
        )
        && window_matches(
            collection.parser_content_span(),
            collection_owner.parser_prefix(),
            collection.content(),
        )
}

fn window_matches(span: oxc_span::Span, prefix: u32, text: &str) -> bool {
    span.start == prefix
        && u32::try_from(text.len())
            .ok()
            .and_then(|len| prefix.checked_add(len))
            == Some(span.end)
}
