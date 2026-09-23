//! Validate the supported backend shape after the generic graph verifier.
//! One pass builds dense tables and the ordering obligations; source order
//! must agree with explicit S3 edges.

mod attach;
mod component;
mod control;
mod ident;
mod model;
mod operands;
mod order;
mod slots;
mod spread;
mod tree;

use std::borrow::Cow;

use vize_carton::{Allocator, Vec};
use vize_s3::{
    op::{EdgeKind, OpId, OpKind, Phase, Program, RegionId},
    operand::{Operand, OperandRole as Role},
};

use super::{Content, NativeArtifact, Node};
use crate::s3::{AdmissionFailure, LegacyReason, retained::Retained, templates::TemplateLoop};

pub(in crate::s3) use ident::{admitted_void, reference};

type Result<T> = core::result::Result<T, AdmissionFailure>;

/// The native node index and region of each op admitted as a node, by op id.
type Slots = [Option<(usize, RegionId)>];

pub(super) fn admit<'a>(
    program: &Program<'a>,
    retained: &Retained<'_, 'a>,
    loops: &[TemplateLoop<'a>],
) -> Result<NativeArtifact<'a>> {
    if program.phase != Phase::Built {
        return Err(LegacyReason::Operation.into());
    }
    // The S3 lowering mints op ids densely in op order, so ids index tables.
    if (program.ops.iter().enumerate()).any(|(index, op)| op.id.index() as usize != index) {
        return Err(AdmissionFailure::Invalid("op ids are not dense"));
    }
    // Admission tables and the payload live in the output arena: one compile
    // owns them, and bump allocation keeps admission off the global heap.
    let alloc = retained.allocator();
    let (operands, starts) = by_op(program, alloc)?;
    let controlled = control::controlled_regions(program, alloc)?;
    let count = program.ops.len();
    let mut nodes = Vec::with_capacity_in(count, &alloc);
    let mut slots = filled(alloc, None, count);
    // Node indexes of each region, in source order.
    let mut regions: Vec<'a, Vec<'a, usize>> = Vec::from_iter_in(
        (0..program.regions.len()).map(|_| Vec::new_in(&alloc)),
        &alloc,
    );
    let mut bindings = Vec::new_in(&alloc);
    let mut edges = Vec::with_capacity_in(program.edges.len(), &alloc);
    let mut last_in_region: Vec<'a, Option<OpId>> = filled(alloc, None, program.regions.len());
    let mut last_binding: Vec<'a, Option<OpId>> = filled(alloc, None, count);
    let mut last_effect = None;
    for (index, op) in program.ops.iter().enumerate() {
        if op.effect.is_some()
            && let Some(previous) = last_effect.replace(op.id)
        {
            edges.push(order::edge(previous, op.id, EdgeKind::EffectOrder));
        }
        let values = (starts.get(index..=index + 1))
            .and_then(|range| match range {
                [start, end] => operands.get(*start..*end),
                _ => None,
            })
            .ok_or(AdmissionFailure::Invalid(DANGLING))?;
        let content = match op.kind {
            OpKind::InsertNode => operands::element(values, alloc)?,
            OpKind::SetText if values.iter().all(|value| value.role == Role::Text) => {
                operands::text(values, retained)?
            }
            // Slot content shares the outlet op kind; it binds to its template or
            // component like any other binding.
            OpKind::SetProp
            | OpKind::SetDynamicProps
            | OpKind::SetEvent
            | OpKind::SetText
            | OpKind::SetHtml
            | OpKind::Directive
            | OpKind::SlotOutlet
                if op.kind != OpKind::SlotOutlet
                    || values.iter().any(|value| value.role == Role::BindingKind) =>
            {
                let (target, mut binding) = if op.kind == OpKind::SlotOutlet {
                    slots::slot(values, alloc)?
                } else {
                    operands::binding(values, op.kind, retained)?
                };
                binding.position = op.span.start;
                // `v-cloak` is a one-shot DOM operation in a static region.
                // In a dynamic branch it inherits that region's effect scope.
                if op.effect.is_none() && binding.kind != super::BindingKind::Cloak {
                    return Err(AdmissionFailure::Invalid(
                        "binding lacks its dynamic partition",
                    ));
                }
                let last = last_binding
                    .get_mut(target.index() as usize)
                    .ok_or(LegacyReason::Structure)?;
                if let Some(previous) = last.replace(op.id) {
                    edges.push(order::edge(previous, op.id, EdgeKind::DomOrder));
                }
                bindings.push((target, op.region, op.span.start, binding));
                continue;
            }
            OpKind::If => control::branches(values, retained)?,
            OpKind::For => {
                let carrier = loops
                    .iter()
                    .find(|(id, _)| *id == op.id)
                    .map(|(_, key)| *key);
                control::for_loop(values, retained, carrier, op.span)?
            }
            OpKind::CreateComponent => component::component(values, alloc)?,
            OpKind::SlotOutlet => component::outlet(values, alloc)?,
            _ => return Err(LegacyReason::Operation.into()),
        };
        let region = op.region.index() as usize;
        let (Some(members), Some(last), Some(&controlled)) = (
            regions.get_mut(region),
            last_in_region.get_mut(region),
            controlled.get(region),
        ) else {
            return Err(AdmissionFailure::Invalid("op region does not resolve"));
        };
        // Everything inside a branch or loop body is partitioned as dynamic.
        let dynamic = match content {
            Content::Text { dynamic, .. } => dynamic,
            Content::If { .. }
            | Content::For(_)
            | Content::Component { .. }
            | Content::Outlet { .. } => true,
            Content::Element { .. } => false,
        } || controlled;
        if dynamic != op.effect.is_some() {
            return Err(AdmissionFailure::Invalid(
                "native payload disagrees with its partition",
            ));
        }
        if let Some(previous) = last.replace(op.id) {
            edges.push(order::edge(previous, op.id, EdgeKind::DomOrder));
        }
        members.push(nodes.len());
        *at_mut(&mut slots, index)? = Some((nodes.len(), op.region));
        nodes.push(Node {
            content,
            children: Vec::new_in(&alloc),
            bindings: Vec::new_in(&alloc),
        });
    }
    order::check(program, edges, alloc)?;
    let parents = tree::assemble(program, &mut nodes, &slots, &mut regions, alloc)?;
    attach::bindings(&mut nodes, &slots, &parents, bindings, alloc)?;
    slots::check(&nodes, &parents)?;
    model::check(&nodes)?;
    spread::check(&nodes)?;
    tree::check_nesting(&nodes, &parents, alloc)?;
    // The root fragment may hold several nodes, text included.
    let roots = std::mem::replace(
        at_mut(&mut regions, RegionId::ROOT.index() as usize)?,
        Vec::new_in(&alloc),
    );
    if roots.is_empty() {
        return Err(LegacyReason::Structure.into());
    }
    Ok(NativeArtifact { nodes, roots })
}

/// Operands grouped by op in authored order: the program's own storage when
/// the producer already emitted them by op, else a counting sort.
fn by_op<'p, 'a>(
    program: &'p Program<'a>,
    alloc: &'a Allocator,
) -> Result<(Cow<'p, [Operand<'a>]>, Vec<'a, usize>)> {
    let mut starts = filled(alloc, 0_usize, program.ops.len() + 1);
    for operand in &program.operands {
        *starts
            .get_mut(operand.op.index() as usize + 1)
            .ok_or(AdmissionFailure::Invalid("operand op does not resolve"))? += 1;
    }
    let mut total = 0;
    for start in starts.iter_mut() {
        total += *start;
        *start = total;
    }
    let operands = if program.operands.is_sorted_by_key(|operand| operand.op) {
        Cow::Borrowed(program.operands.as_slice())
    } else {
        let mut cursor = starts.to_vec();
        let mut placed = program.operands.to_vec();
        for operand in &program.operands {
            let slot = at_mut(&mut cursor, operand.op.index() as usize)?;
            *at_mut(&mut placed, *slot)? = *operand;
            *slot += 1;
        }
        Cow::Owned(placed)
    };
    Ok((operands, starts))
}

/// `len` copies of `value` in the arena (`vec![value; len]`).
/// `items[index]`, or the payload is inconsistent.
fn at<T>(items: &[T], index: usize) -> Result<&T> {
    items.get(index).ok_or(AdmissionFailure::Invalid(DANGLING))
}

/// [`at`] for a mutable slot.
fn at_mut<T>(items: &mut [T], index: usize) -> Result<&mut T> {
    items
        .get_mut(index)
        .ok_or(AdmissionFailure::Invalid(DANGLING))
}

const DANGLING: &str = "native payload index does not resolve";

fn filled<'a, T: Clone>(alloc: &'a Allocator, value: T, len: usize) -> Vec<'a, T> {
    let mut table = Vec::with_capacity_in(len, &alloc);
    table.resize(len, value);
    table
}
