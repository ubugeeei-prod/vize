//! `<template v-if>` and `<template v-for>` carriers unwrap in S2, so S3 sees
//! only their content; S2 keeps what the wrapper carried as side facts. A
//! plain wrapper needs nothing further. A loop wrapper's `:key` becomes the
//! native loop key; branch keys, cached chains and other wrapper attributes
//! stay legacy.

use vize_carton::{Allocator, Span};
use vize_s1_to_s2::lower::WrapperKey;
use vize_s3::op::{OpId, OpKind, Program};

use super::{LegacyReason, native::validate::reference, retained::Retained};

/// A template-carried loop: its S3 op and its wrapper key text and span.
pub(super) type TemplateLoop<'a> = (OpId, Option<(&'a str, Span)>);

pub(super) fn collect<'a>(
    allocator: &'a Allocator,
    source: &str,
    s2: &vize_s1_to_s2::Lowered<'_>,
    program: &Program<'a>,
    retained: &mut Retained<'_, 'a>,
) -> Result<std::vec::Vec<TemplateLoop<'a>>, LegacyReason> {
    if s2
        .wrappers
        .iter()
        .any(|(_, keys)| keys.once || keys.branches.iter().any(Option::is_some))
        || s2
            .if_facts
            .iter()
            .any(|(_, facts)| facts.branches.iter().any(Option::is_some))
    {
        return Err(LegacyReason::ControlFlow);
    }
    let mut loops = std::vec::Vec::with_capacity(s2.for_wrappers.len());
    for (node, wrapper) in s2.for_wrappers.iter() {
        if !wrapper.attributes.is_empty() || wrapper.class.is_some() {
            return Err(LegacyReason::ControlFlow);
        }
        // The loop's directive value locates its S3 op: the innermost loop
        // spanning it.
        let value = s2
            .provenance
            .iter()
            .find(|record| record.rule == "lower.for" && record.node == Some(node))
            .map(|record| record.span)
            .ok_or(LegacyReason::ControlFlow)?;
        let op = program
            .ops
            .iter()
            .filter(|op| {
                op.kind == OpKind::For && op.span.start <= value.start && value.end <= op.span.end
            })
            .min_by_key(|op| op.span.end - op.span.start)
            .ok_or(LegacyReason::ControlFlow)?;
        let key = match &wrapper.key {
            None => None,
            Some(WrapperKey::Dynamic { source: text, span }) => {
                Some(key(allocator, source, text, *span, retained)?)
            }
            Some(WrapperKey::Static { .. }) => return Err(LegacyReason::ControlFlow),
        };
        loops.push((op.id, key));
    }
    Ok(loops)
}

/// The key expression in the output arena at its value span (the fact records
/// the whole attribute). A non-reference key takes its single parse here, as
/// compound text parts do; S2 kept only the text.
fn key<'a>(
    allocator: &'a Allocator,
    source: &str,
    text: &str,
    attribute: Span,
    retained: &mut Retained<'_, 'a>,
) -> Result<(&'a str, Span), LegacyReason> {
    let authored = source
        .get(attribute.start as usize..attribute.end as usize)
        .ok_or(LegacyReason::ControlFlow)?;
    // A valueless `:key` reads `key` and has no value text to locate.
    let equals = authored.find('=').ok_or(LegacyReason::ControlFlow)?;
    let at = (authored.get(equals..))
        .and_then(|value| value.find(text))
        .ok_or(LegacyReason::ControlFlow)?
        + equals;
    let start = attribute.start + at as u32;
    let span = Span::new(start, start + text.len() as u32);
    let text: &str = allocator.alloc_str(text);
    if !reference(text) && !retained.parse(text, span) {
        return Err(LegacyReason::ExpressionOrEncoding);
    }
    Ok((text, span))
}
