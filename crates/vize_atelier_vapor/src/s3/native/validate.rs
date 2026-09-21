//! Validate the supported backend shape after the generic graph verifier.
//! One pass builds dense tables and the ordering obligations; source order
//! must agree with explicit S3 edges.

mod attach;
mod component;
mod control;
mod operands;
mod order;
mod slots;
mod tree;

use std::borrow::Cow;

use vize_s3::{
    op::{EdgeKind, OpId, OpKind, Phase, Program, RegionId},
    operand::{Operand, OperandRole as Role},
};

use super::{Content, NativeArtifact, Node};
use crate::s3::{AdmissionFailure, LegacyReason, retained::Retained, templates::TemplateLoop};

pub(in crate::s3) use operands::reference;

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
    let (operands, starts) = by_op(program)?;
    let controlled = control::controlled_regions(program)?;
    let count = program.ops.len();
    let mut nodes = std::vec::Vec::with_capacity(count);
    let mut slots = std::vec![None; count];
    // Node indexes of each region, in source order.
    let mut regions: std::vec::Vec<std::vec::Vec<usize>> =
        std::vec![std::vec::Vec::new(); program.regions.len()];
    let mut bindings = std::vec::Vec::new();
    let mut edges = std::vec::Vec::with_capacity(program.edges.len());
    let mut last_in_region: std::vec::Vec<Option<OpId>> = std::vec![None; program.regions.len()];
    let mut last_binding: std::vec::Vec<Option<OpId>> = std::vec![None; count];
    let mut last_effect = None;
    for (index, op) in program.ops.iter().enumerate() {
        if op.effect.is_some()
            && let Some(previous) = last_effect.replace(op.id)
        {
            edges.push(order::edge(previous, op.id, EdgeKind::EffectOrder));
        }
        let values = &operands[starts[index]..starts[index + 1]];
        let content = match op.kind {
            OpKind::InsertNode => operands::element(values)?,
            OpKind::SetText if values.iter().all(|value| value.role == Role::Text) => {
                operands::text(values, retained)?
            }
            // Slot content shares the outlet op kind; it binds to its template or
            // component like any other binding.
            OpKind::SetProp
            | OpKind::SetEvent
            | OpKind::SetText
            | OpKind::SetHtml
            | OpKind::Directive
            | OpKind::SlotOutlet
                if op.kind != OpKind::SlotOutlet
                    || values.iter().any(|value| value.role == Role::BindingKind) =>
            {
                let (target, binding) = if op.kind == OpKind::SlotOutlet {
                    slots::slot(values)?
                } else {
                    operands::binding(values, op.kind, retained)?
                };
                if op.effect.is_none() {
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
                control::for_loop(values, retained, carrier)?
            }
            OpKind::CreateComponent => component::component(values)?,
            OpKind::SlotOutlet => component::outlet(values)?,
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
        slots[index] = Some((nodes.len(), op.region));
        nodes.push(Node {
            content,
            children: std::vec::Vec::new(),
            bindings: std::vec::Vec::new(),
        });
    }
    order::check(program, edges)?;
    let parents = tree::assemble(program, &mut nodes, &slots, &mut regions)?;
    attach::bindings(&mut nodes, &slots, &parents, bindings)?;
    slots::check(&nodes, &parents)?;
    tree::check_nesting(&nodes, &parents)?;
    // The root fragment may hold several nodes, text included.
    let roots = std::mem::take(&mut regions[RegionId::ROOT.index() as usize]);
    if roots.is_empty() {
        return Err(LegacyReason::Structure.into());
    }
    Ok(NativeArtifact { nodes, roots })
}

/// Operands grouped by op in authored order: the program's own storage when
/// the producer already emitted them by op, else a counting sort.
fn by_op<'p, 'a>(
    program: &'p Program<'a>,
) -> Result<(Cow<'p, [Operand<'a>]>, std::vec::Vec<usize>)> {
    let mut starts = std::vec![0_usize; program.ops.len() + 1];
    for operand in &program.operands {
        *starts
            .get_mut(operand.op.index() as usize + 1)
            .ok_or(AdmissionFailure::Invalid("operand op does not resolve"))? += 1;
    }
    for index in 1..starts.len() {
        starts[index] += starts[index - 1];
    }
    let operands = if program.operands.is_sorted_by_key(|operand| operand.op) {
        Cow::Borrowed(&program.operands[..])
    } else {
        let mut cursor = starts.clone();
        let mut placed = program.operands.to_vec();
        for operand in &program.operands {
            let slot = &mut cursor[operand.op.index() as usize];
            placed[*slot] = *operand;
            *slot += 1;
        }
        Cow::Owned(placed)
    };
    Ok((operands, starts))
}
