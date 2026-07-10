use super::super::Croquis;
use super::{names, types as snapshot};
use crate::reactivity::{ReactivityLoss, ReactivityLossKind};

pub(super) fn reactivity_loss_snapshots(
    croquis: &Croquis,
) -> Vec<snapshot::SemanticReactivityLossSnapshot> {
    let mut losses: Vec<_> = croquis
        .reactivity
        .losses()
        .iter()
        .map(reactivity_loss_snapshot)
        .collect();
    losses.sort_by(|left, right| {
        (left.range.start, left.range.end, left.kind).cmp(&(
            right.range.start,
            right.range.end,
            right.kind,
        ))
    });
    losses
}

fn reactivity_loss_snapshot(loss: &ReactivityLoss) -> snapshot::SemanticReactivityLossSnapshot {
    let mut snapshot = snapshot::SemanticReactivityLossSnapshot {
        id: names::semantic_id(
            "reactivity-loss",
            names::reactivity_loss_kind_name(&loss.kind),
            loss.start,
        ),
        kind: names::reactivity_loss_kind_name(&loss.kind),
        category: "loss",
        source_name: None,
        target_name: None,
        property_name: None,
        extracted_names: Vec::new(),
        range: snapshot::SemanticSourceRange::new(loss.start, loss.end),
    };

    match &loss.kind {
        ReactivityLossKind::ReactiveDestructure {
            source_name,
            destructured_props,
        }
        | ReactivityLossKind::RefValueDestructure {
            source_name,
            destructured_props,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.extracted_names = destructured_props.clone();
        }
        ReactivityLossKind::RefValueExtract {
            source_name,
            target_name,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::ReactivePropertyExtract {
            source_name,
            prop_name,
            target_name,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.property_name = Some(prop_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::PropsDestructure { destructured_props } => {
            snapshot.extracted_names = destructured_props.clone();
        }
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name: _,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.target_name = Some(argument_name.clone());
        }
        ReactivityLossKind::GetterCallExtract {
            source_name,
            getter_name,
            target_name,
            ..
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.property_name = Some(getter_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.property_name = Some(alias_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::ReactiveSpread { source_name }
        | ReactivityLossKind::ReactiveReassign { source_name } => {
            snapshot.source_name = Some(source_name.clone());
        }
    }

    snapshot
}
