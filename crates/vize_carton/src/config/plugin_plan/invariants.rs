//! Structural invariants of a canonical plan: registry versions, the stable
//! topological order, and exclusive ownership claims.

use super::{PluginClaim, PluginPlan, PluginPlanError, PluginRegistry};
use crate::{FxHashMap, FxHashSet, String, cstr};

pub(super) fn verify_plugins(
    plan: &PluginPlan,
    registry: &PluginRegistry,
) -> Result<(), PluginPlanError> {
    let mut names = FxHashSet::default();
    let mut indexes = FxHashSet::default();
    for plugin in &plan.plugins {
        let fail = |code, message| Err(PluginPlanError::new(code, &plugin.name, message));
        if !names.insert(plugin.name.as_str()) {
            return fail("duplicate-name", "is listed more than once".into());
        }
        if !indexes.insert(plugin.index) {
            return fail(
                "plan-invalid",
                cstr!("reuses config index {}", plugin.index),
            );
        }
        if !registry.api_versions.contains(&plugin.api_version) {
            return fail(
                "api-unsupported",
                cstr!("plugin API {} is not supported", plugin.api_version),
            );
        }
        if !registry.admitted_backends.contains(&plugin.backend) {
            return fail(
                "backend-unavailable",
                cstr!("the {} backend is not available", plugin.backend),
            );
        }
        if plugin.capabilities.is_empty() {
            return fail("plan-invalid", "has no capability".into());
        }
        for capability in &plugin.capabilities {
            let Some(known) = registry.capabilities.get(capability.id.as_str()) else {
                return fail(
                    "capability-unknown",
                    cstr!("unknown capability `{}`", capability.id),
                );
            };
            if !known.api_versions.contains(&plugin.api_version)
                || !known.schema_versions.contains(&capability.schema_version)
            {
                return fail(
                    "capability-unsupported",
                    cstr!(
                        "capability `{}` schema {} is not supported for plugin API {}",
                        capability.id,
                        capability.schema_version,
                        plugin.api_version
                    ),
                );
            }
        }
    }
    Ok(())
}

/// The plan must already be the stable topological order of its own edges.
pub(super) fn verify_order(plan: &PluginPlan) -> Result<(), PluginPlanError> {
    let position: FxHashMap<&str, usize> = plan
        .plugins
        .iter()
        .enumerate()
        .map(|(index, plugin)| (plugin.name.as_str(), index))
        .collect();
    let mut incoming = vec![0usize; plan.plugins.len()];
    let mut edges: Vec<Vec<usize>> = vec![Vec::new(); plan.plugins.len()];
    for (index, plugin) in plan.plugins.iter().enumerate() {
        let lookup = |name: &String| {
            position.get(name.as_str()).copied().ok_or_else(|| {
                PluginPlanError::new(
                    "order-unknown",
                    &plugin.name,
                    cstr!("names `{name}`, which is not in the plan"),
                )
            })
        };
        let mut add = |from: usize, to: usize| {
            if let Some(targets) = edges.get_mut(from)
                && !targets.contains(&to)
            {
                targets.push(to);
                if let Some(count) = incoming.get_mut(to) {
                    *count += 1;
                }
            }
        };
        for name in &plugin.before {
            add(index, lookup(name)?);
        }
        for name in &plugin.after {
            add(lookup(name)?, index);
        }
    }
    let mut ready: Vec<usize> = (0..plan.plugins.len())
        .filter(|index| incoming.get(*index) == Some(&0))
        .collect();
    let mut expected = Vec::with_capacity(plan.plugins.len());
    while !ready.is_empty() {
        ready.sort_by_key(|index| plan.plugins.get(*index).map_or(u32::MAX, |p| p.index));
        let next = ready.remove(0);
        expected.push(next);
        for target in edges.get(next).into_iter().flatten() {
            if let Some(count) = incoming.get_mut(*target) {
                *count -= 1;
                if *count == 0 {
                    ready.push(*target);
                }
            }
        }
    }
    if expected.len() != plan.plugins.len() {
        return Err(PluginPlanError::new(
            "order-cycle",
            "plan",
            "contains an ordering cycle".into(),
        ));
    }
    if expected.iter().copied().ne(0..plan.plugins.len()) {
        return Err(PluginPlanError::new(
            "plan-invalid",
            "plan",
            "plugins are not in their stable topological order".into(),
        ));
    }
    Ok(())
}

pub(super) fn verify_claims(plan: &PluginPlan) -> Result<(), PluginPlanError> {
    let mut claims: Vec<PluginClaim> = plan
        .plugins
        .iter()
        .flat_map(|plugin| {
            plugin.capabilities.iter().map(|capability| PluginClaim {
                key: cstr!("{}:{}", capability.id, capability.key),
                owner: plugin.name.clone(),
            })
        })
        .collect();
    claims.sort_by(|left, right| left.key.cmp(&right.key));
    if claims != plan.claims {
        return Err(PluginPlanError::new(
            "plan-invalid",
            "plan",
            "claims do not match the planned capabilities".into(),
        ));
    }
    let mut seen = FxHashSet::default();
    let mut blocks: FxHashMap<&str, (bool, usize)> = FxHashMap::default();
    for claim in &claims {
        if !seen.insert(claim.key.as_str()) {
            return Err(PluginPlanError::new(
                "owner-conflict",
                &claim.key,
                "has more than one owner".into(),
            ));
        }
        if let Some(block) = claim.key.strip_prefix("sfc.custom-block:") {
            let (block_type, lang) = match block.split_once(':') {
                Some((block_type, _)) => (block_type, true),
                None => (block, false),
            };
            let entry = blocks.entry(block_type).or_insert((false, 0));
            if lang {
                entry.1 += 1;
            } else {
                entry.0 = true;
            }
            if entry.0 && entry.1 > 0 {
                return Err(PluginPlanError::new(
                    "owner-conflict",
                    &claim.key,
                    "a block claimed for every language cannot also be claimed per language".into(),
                ));
            }
        }
    }
    Ok(())
}
