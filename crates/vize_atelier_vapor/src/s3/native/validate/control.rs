//! Control-flow operand schemas and the controlled-region closure.
//!
//! Branch conditions and loop sources share the direct-reference grammar of
//! the rest of the native slice. Loop aliases must be plain identifiers: S2
//! enumerates no names for destructuring patterns (`ForName::Pending`), so
//! those keep the legacy lane rather than being re-derived from text here.

use vize_atelier_core::steps::expression::is_template_global;
use vize_carton::{Allocator, Span, Vec};
use vize_s3::{
    op::{OpId, OpKind, Program, RegionId},
    operand::{Operand, OperandRole as Role, OperandValue, ValueKind},
};

use super::super::{Branch, Content, Expr, Loop, LoopSpans};
use super::{Result, ident::reference, operands::js, operands::one};
use crate::s3::{AdmissionFailure, LegacyReason, retained::Retained};

pub(super) fn branches<'a>(
    values: &[Operand<'a>],
    retained: &Retained<'_, 'a>,
) -> Result<Content<'a>> {
    let alloc = retained.allocator();
    let mut branches: Vec<'a, Branch<'a>> = Vec::with_capacity_in(values.len(), &alloc);
    for (position, value) in values.iter().enumerate() {
        let Some(region) = value.region else {
            return Err(AdmissionFailure::Invalid("invalid native branch operand"));
        };
        if value.role != Role::Condition
            || value.target.is_some()
            || value.name.is_some()
            || branches.iter().any(|branch| branch.region == region)
        {
            return Err(AdmissionFailure::Invalid("invalid native branch operand"));
        }
        let condition = match value.value.kind {
            // Only a trailing branch after a conditional one is unconditional.
            ValueKind::Absent if position > 0 && position + 1 == values.len() => None,
            ValueKind::Absent => {
                return Err(AdmissionFailure::Invalid(
                    "unconditional branch is not trailing",
                ));
            }
            _ => Some(js(retained, value)?),
        };
        branches.push(Branch {
            condition,
            span: (value.value.span.start, value.value.span.end),
            region,
            roots: Vec::new_in(&alloc),
        });
    }
    if branches.is_empty() {
        return Err(AdmissionFailure::Invalid("conditional without branches"));
    }
    Ok(Content::If { branches })
}

/// `carrier` is `Some` for a `<template v-for>`, holding its wrapper key.
/// `carrier_span` is the carrier element's authored span (its `<`).
pub(super) fn for_loop<'a>(
    values: &[Operand<'a>],
    retained: &Retained<'_, 'a>,
    carrier: Option<Option<(&'a str, Span)>>,
    carrier_span: Span,
) -> Result<Content<'a>> {
    if values.iter().any(|value| {
        !matches!(
            value.role,
            Role::ForSource | Role::ForValue | Role::ForKey | Role::ForIndex
        ) || value.target.is_some()
            || value.region.is_some()
            || value.name.is_some()
    }) {
        return Err(AdmissionFailure::Invalid("invalid native loop operand"));
    }
    let source = one(values, Role::ForSource)?;
    let value = one(values, Role::ForValue)?.value;
    let key = one(values, Role::ForKey)?.value;
    let index = one(values, Role::ForIndex)?.value;
    let span_of = |value: OperandValue<'_>| (value.span.start, value.span.end);
    let mut spans = LoopSpans {
        source: span_of(source.value),
        aliases: [
            Some(span_of(value)),
            (key.kind != ValueKind::Absent).then(|| span_of(key)),
            (index.kind != ValueKind::Absent).then(|| span_of(index)),
        ],
        key_prop: None,
    };
    let text = source.value.text;
    let source = if source.value.kind == ValueKind::Js
        && !text.is_empty()
        && text.bytes().all(|b| b.is_ascii_digit())
    {
        Expr::plain(text)
    } else {
        js(retained, source)?
    };
    let value = alias(value.kind, value.text)?.ok_or(LegacyReason::ControlFlow)?;
    let key = alias(key.kind, key.text)?;
    let index = alias(index.kind, index.text)?;
    // The generator names the index parameter after the key position.
    if index.is_some() && key.is_none()
        || key == Some(value)
        || index.is_some_and(|index| Some(index) == key || index == value)
    {
        return Err(LegacyReason::ControlFlow.into());
    }
    let key_prop = match carrier.flatten() {
        Some((text, key_span)) => {
            spans.key_prop = Some((key_span.start, key_span.end));
            if reference(text) {
                Some(Expr::plain(text))
            } else {
                Some(Expr {
                    text,
                    js: Some(
                        retained
                            .expression(text, key_span)
                            .ok_or(LegacyReason::ExpressionOrEncoding)?,
                    ),
                })
            }
        }
        None => None,
    };
    Ok(Content::For(Loop {
        source,
        value,
        key,
        index,
        key_prop,
        template: carrier.is_some(),
        spans,
        carrier_start: carrier_span.start,
    }))
}

fn alias(kind: ValueKind, text: &str) -> Result<Option<&str>> {
    match kind {
        ValueKind::Absent => Ok(None),
        ValueKind::Js
            if text.starts_with(|c: char| c.is_ascii_alphabetic())
                && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !super::ident::reserved_keyword(text)
                && !is_template_global(text) =>
        {
            Ok(Some(text))
        }
        _ => Err(LegacyReason::ControlFlow.into()),
    }
}

/// Regions inside a conditional branch or loop body, by region index. S2 to
/// S3 partitions every op there as dynamic, so native payload classification
/// must agree. The verifier rejects duplicate ids, so ids below the region
/// count index a dense table.
pub(super) fn controlled_regions<'a>(
    program: &Program<'_>,
    alloc: &'a Allocator,
) -> Result<Vec<'a, bool>> {
    let count = program.regions.len();
    let mut meta = super::filled(alloc, (None, None), count);
    for region in &program.regions {
        *meta
            .get_mut(region.id.index() as usize)
            .ok_or(AdmissionFailure::Invalid("region ids are not dense"))? =
            (region.parent, region.owner);
    }
    let mut memo: Vec<'_, Option<bool>> = super::filled(alloc, None, count);
    let mut path = Vec::new_in(&alloc);
    for region in &program.regions {
        path.clear();
        let mut cursor: Option<RegionId> = Some(region.id);
        let mut controlled = false;
        while let Some(id) = cursor {
            let slot = id.index() as usize;
            let &(parent, owner) = meta
                .get(slot)
                .ok_or(AdmissionFailure::Invalid("region does not resolve"))?;
            if let Some(&Some(known)) = memo.get(slot) {
                controlled = known;
                break;
            }
            if path.len() > count {
                return Err(AdmissionFailure::Invalid("region parent cycle"));
            }
            path.push(slot);
            // Slot content and outlet fallbacks are partitioned as dynamic too.
            if owner.is_some_and(|owner: OpId| {
                matches!(
                    program.ops.get(owner.index() as usize).map(|op| op.kind),
                    Some(OpKind::If | OpKind::For | OpKind::CreateComponent | OpKind::SlotOutlet)
                )
            }) {
                controlled = true;
                break;
            }
            cursor = parent;
        }
        for slot in &path {
            if let Some(known) = memo.get_mut(*slot) {
                *known = Some(controlled);
            }
        }
    }
    Ok(Vec::from_iter_in(
        memo.iter().map(|known| *known == Some(true)),
        &alloc,
    ))
}
