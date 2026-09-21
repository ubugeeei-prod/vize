//! Validate the supported backend shape after the generic graph verifier.
//! Indexes are built once; source order must agree with explicit S3 edges.

mod attach;
mod component;
mod control;
mod operands;
mod order;
mod tree;

use vize_carton::FxHashMap;
use vize_s3::{
    op::{OpId, OpKind, Phase, Program, RegionId},
    operand::{Operand, OperandRole as Role},
};

use super::{Content, NativeArtifact, Node};
use crate::s3::{AdmissionFailure, LegacyReason, retained::Retained};

pub(in crate::s3) use operands::reference;

type Result<T> = core::result::Result<T, AdmissionFailure>;

pub(super) fn admit<'a>(
    program: &Program<'a>,
    retained: &Retained<'_, 'a>,
) -> Result<NativeArtifact<'a>> {
    if program.phase != Phase::Built {
        return Err(LegacyReason::Operation.into());
    }
    let mut operands: FxHashMap<OpId, std::vec::Vec<&Operand<'a>>> = FxHashMap::default();
    for operand in &program.operands {
        operands.entry(operand.op).or_default().push(operand);
    }
    let kinds: FxHashMap<_, _> = program.ops.iter().map(|op| (op.id, op.kind)).collect();
    let controlled = control::controlled_regions(program, &kinds)?;
    let mut nodes = std::vec::Vec::new();
    let mut indexes = FxHashMap::default();
    let mut regions: FxHashMap<RegionId, std::vec::Vec<OpId>> = FxHashMap::default();
    let mut binding_order: FxHashMap<OpId, std::vec::Vec<OpId>> = FxHashMap::default();
    let mut bindings = std::vec::Vec::new();
    for op in &program.ops {
        let values = operands.get(&op.id).map_or(&[][..], |v| &v[..]);
        let content = match op.kind {
            OpKind::InsertNode => operands::element(values)?,
            OpKind::SetText if values.iter().all(|value| value.role == Role::Text) => {
                operands::text(values, retained)?
            }
            OpKind::SetProp
            | OpKind::SetEvent
            | OpKind::SetText
            | OpKind::SetHtml
            | OpKind::Directive => {
                let (target, binding) = operands::binding(values, op.kind, retained)?;
                if op.effect.is_none() {
                    return Err(AdmissionFailure::Invalid(
                        "binding lacks its dynamic partition",
                    ));
                }
                binding_order.entry(target).or_default().push(op.id);
                bindings.push((target, op.region, op.span.start, binding));
                continue;
            }
            OpKind::If => control::branches(values, retained)?,
            OpKind::For => control::for_loop(values, retained)?,
            OpKind::CreateComponent => component::component(values)?,
            // Slot-content bindings (named or scoped slots) share the op kind.
            OpKind::SlotOutlet if values.iter().any(|value| value.role == Role::BindingKind) => {
                return Err(LegacyReason::Component.into());
            }
            OpKind::SlotOutlet => component::outlet(values)?,
            _ => return Err(LegacyReason::Operation.into()),
        };
        // Everything inside a branch or loop body is partitioned as dynamic.
        let dynamic = match content {
            Content::Text { dynamic, .. } => dynamic,
            Content::If { .. }
            | Content::For(_)
            | Content::Component { .. }
            | Content::Outlet { .. } => true,
            Content::Element { .. } => false,
        } || controlled.contains(&op.region);
        if dynamic != op.effect.is_some() {
            return Err(AdmissionFailure::Invalid(
                "native payload disagrees with its partition",
            ));
        }
        indexes.insert(op.id, (nodes.len(), op.region));
        regions.entry(op.region).or_default().push(op.id);
        nodes.push(Node {
            content,
            children: std::vec::Vec::new(),
            bindings: std::vec::Vec::new(),
        });
    }
    order::check(program, &regions, &binding_order)?;
    let parents = tree::assemble(program, &mut nodes, &indexes, &regions)?;
    attach::bindings(&mut nodes, &indexes, &parents, bindings)?;
    tree::check_nesting(&nodes, &parents)?;
    let roots: std::vec::Vec<_> = regions
        .get(&RegionId::ROOT)
        .into_iter()
        .flatten()
        .map(|id| indexes[id].0)
        .collect();
    // The root fragment may hold several nodes, text included.
    if roots.is_empty() {
        return Err(LegacyReason::Structure.into());
    }
    Ok(NativeArtifact { nodes, roots })
}
