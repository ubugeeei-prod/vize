//! Control-flow operand schemas and the controlled-region closure.
//!
//! Branch conditions and loop sources share the direct-reference grammar of
//! the rest of the native slice. Loop aliases must be plain identifiers: S2
//! enumerates no names for destructuring patterns (`ForName::Pending`), so
//! those keep the legacy lane rather than being re-derived from text here.

use vize_atelier_core::steps::expression::is_template_global;
use vize_carton::{FxHashMap, FxHashSet};
use vize_s3::{
    op::{OpId, OpKind, Program, RegionId},
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::super::{Branch, Content, Expr, Loop};
use super::{Result, operands::js, operands::one};
use crate::s3::{AdmissionFailure, LegacyReason, retained::Retained};

pub(super) fn branches<'a>(
    values: &[&Operand<'a>],
    retained: &Retained<'_, 'a>,
) -> Result<Content<'a>> {
    let mut branches: std::vec::Vec<Branch<'a>> = std::vec::Vec::with_capacity(values.len());
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
            region,
            root: None,
        });
    }
    if branches.is_empty() {
        return Err(AdmissionFailure::Invalid("conditional without branches"));
    }
    Ok(Content::If { branches })
}

pub(super) fn for_loop<'a>(
    values: &[&Operand<'a>],
    retained: &Retained<'_, 'a>,
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
    Ok(Content::For(Loop {
        source,
        value,
        key,
        index,
        key_prop: None,
    }))
}

fn alias(kind: ValueKind, text: &str) -> Result<Option<&str>> {
    match kind {
        ValueKind::Absent => Ok(None),
        ValueKind::Js
            if text.starts_with(|c: char| c.is_ascii_alphabetic())
                && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !oxc_syntax::keyword::is_reserved_keyword(text)
                && !is_template_global(text) =>
        {
            Ok(Some(text))
        }
        _ => Err(LegacyReason::ControlFlow.into()),
    }
}

/// Regions inside a conditional branch or loop body. S2 to S3 partitions every
/// op there as dynamic, so native payload classification must agree.
pub(super) fn controlled_regions(
    program: &Program<'_>,
    kinds: &FxHashMap<OpId, OpKind>,
) -> Result<FxHashSet<RegionId>> {
    let meta: FxHashMap<_, _> = program
        .regions
        .iter()
        .map(|region| (region.id, (region.parent, region.owner)))
        .collect();
    let mut memo: FxHashMap<RegionId, bool> = FxHashMap::default();
    let mut path = std::vec::Vec::new();
    for region in &program.regions {
        path.clear();
        let mut cursor = Some(region.id);
        let mut controlled = false;
        while let Some(id) = cursor {
            if let Some(&known) = memo.get(&id) {
                controlled = known;
                break;
            }
            let &(parent, owner) = meta
                .get(&id)
                .ok_or(AdmissionFailure::Invalid("region does not resolve"))?;
            if path.len() > program.regions.len() {
                return Err(AdmissionFailure::Invalid("region parent cycle"));
            }
            path.push(id);
            // Slot content and outlet fallbacks are partitioned as dynamic too.
            if owner.is_some_and(|owner| {
                matches!(
                    kinds.get(&owner),
                    Some(OpKind::If | OpKind::For | OpKind::CreateComponent | OpKind::SlotOutlet)
                )
            }) {
                controlled = true;
                break;
            }
            cursor = parent;
        }
        for id in &path {
            memo.insert(*id, controlled);
        }
    }
    Ok(memo
        .into_iter()
        .filter_map(|(id, controlled)| controlled.then_some(id))
        .collect())
}
