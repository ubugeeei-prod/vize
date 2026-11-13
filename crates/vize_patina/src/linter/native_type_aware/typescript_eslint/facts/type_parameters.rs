//! no-unnecessary-type-parameters: the checker count of each type
//! parameter's uses, asked of its owner when the written uses do not settle it.
use super::queries::Planner;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery, TypeParameterUsage};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use vize_l0::FxHashMap;

/// Queries the owner of every declaration with a parameter the written
/// references leave undecided, and the ranges its walk starts from.
pub(super) fn request(nodes: &[LintNode], plan: &mut Planner) {
    for node in nodes {
        let Some((owner, usage)) = usage_of(node) else {
            continue;
        };
        let ranges = usage
            .parameters
            .iter()
            .copied()
            .chain(usage.defaults.iter().flatten().copied())
            .chain(usage.members.iter().map(|&(start, end, _)| (start, end)))
            .collect::<Vec<_>>();
        for (start, end) in ranges {
            plan.push(ExpressionQuery::at(start, end));
        }
        let mut query = ExpressionQuery::at(owner.0, owner.1);
        query.type_parameter_usage = Some(usage);
        plan.push(query);
    }
}

fn usage_of(node: &LintNode) -> Option<((u32, u32), TypeParameterUsage)> {
    if node.kind != "TSTypeParameterDeclaration" {
        return None;
    }
    let owner = range(node.fields.get("__ownerRange")?)?;
    let params = node.child_list("params")?;
    if params
        .iter()
        .all(|param| param.field_bool("__repeatedInAst") != Some(false))
    {
        return None;
    }
    let members = node
        .fields
        .get("__ownerMembers")
        .and_then(Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(|member| {
                    let (start, end) = range(member)?;
                    let constructor = member.get("constructor").and_then(Value::as_bool);
                    Some((start, end, constructor == Some(true)))
                })
                .collect()
        })
        .unwrap_or_default();
    let usage = TypeParameterUsage {
        parameters: params
            .iter()
            .map(|param| (param.range.start, param.range.end))
            .collect(),
        defaults: params
            .iter()
            .map(|param| param.fields.get("__defaultRange").and_then(range))
            .collect(),
        members,
        class: node
            .field_str("__ownerKind")
            .is_some_and(|kind| kind.starts_with("Class")),
        signature: node.field_bool("__ownerSignature") == Some(true),
    };
    Some((owner, usage))
}

/// Stamps each parameter's checker count from its owner's fact.
pub(super) fn apply(node: &mut LintNode, facts: &FxHashMap<(u32, u32), &ExpressionFact>) {
    if node.kind != "TSTypeParameterDeclaration" {
        return;
    }
    let Some(counts) = node
        .fields
        .get("__ownerRange")
        .and_then(range)
        .and_then(|owner| facts.get(&owner))
        .and_then(|fact| fact.type_parameter_usage.clone())
    else {
        return;
    };
    let Some(params) = node.child_lists.get_mut("params") else {
        return;
    };
    for (param, count) in params.iter_mut().zip(counts) {
        param
            .fields
            .insert("__typeParameterUsageCount".into(), json!(count));
    }
}

fn range(value: &Value) -> Option<(u32, u32)> {
    let field = |key: &str| u32::try_from(value.get(key)?.as_u64()?).ok();
    Some((field("start")?, field("end")?))
}
