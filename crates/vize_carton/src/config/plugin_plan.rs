//! Native verification of the canonical plugin plan.
//!
//! `vue.plugins` is resolved once, by `loader/plugin-plan.js`, into canonical
//! data. Rust products never rebuild that plan: they verify the canonical
//! bytes, the registry versions, the ordering, and the ownership claims, and
//! refuse any capability addressed to them that they do not implement.

mod canonical;
mod registry;
#[cfg(test)]
mod tests;

use serde::Deserialize;

use crate::{FxHashMap, FxHashSet, String, cstr};

pub use canonical::canonical_json;
pub use registry::{PluginRegistry, plugin_registry};

/// A plugin-plan failure with the same stable code as the JS resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginPlanError {
    pub code: &'static str,
    pub subject: String,
    pub message: String,
}

impl PluginPlanError {
    pub(crate) fn new(code: &'static str, subject: &str, message: String) -> Self {
        Self {
            code,
            subject: subject.into(),
            message,
        }
    }
}

impl core::fmt::Display for PluginPlanError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "[plugin/{}] {}: {}",
            self.code, self.subject, self.message
        )
    }
}

impl std::error::Error for PluginPlanError {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginPlan {
    pub claims: Vec<PluginClaim>,
    pub plan_schema: u32,
    pub plugins: Vec<PlannedPlugin>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginClaim {
    pub key: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlannedPlugin {
    pub after: Vec<String>,
    pub api_version: u32,
    pub backend: String,
    pub before: Vec<String>,
    /// `None` only for a v1 plugin that never declared its cache inputs.
    pub cache_inputs: Option<Vec<PluginCacheInput>>,
    pub capabilities: Vec<PlannedCapability>,
    pub fingerprint: String,
    /// Position in the authored `vue.plugins` array.
    pub index: u32,
    pub name: String,
    pub options: serde_json::Value,
    pub permissions: Vec<serde_json::Value>,
    pub skipped: Vec<SkippedCapability>,
    pub stability: String,
    #[serde(default)]
    pub supports: Option<PluginSupports>,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginCacheInput {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlannedCapability {
    pub data: serde_json::Value,
    pub id: String,
    pub key: String,
    pub required: bool,
    pub schema_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SkippedCapability {
    pub id: String,
    pub schema_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginSupports {
    pub outputs: Vec<String>,
    pub runtimes: Vec<String>,
    pub vue: Vec<String>,
}

/// A plan whose canonical bytes and invariants were checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPluginPlan {
    plan: PluginPlan,
    canonical: String,
    digest: String,
}

/// One capability a product would have to drop to run the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRefusal {
    pub plugin: String,
    pub capability: String,
    pub key: String,
    /// `unadmitted`, or `output:<name>` when the plugin lacks that output.
    pub reason: String,
}

impl VerifiedPluginPlan {
    pub fn plan(&self) -> &PluginPlan {
        &self.plan
    }

    /// The canonical serialization shared with the JS resolver.
    pub fn canonical_json(&self) -> &str {
        &self.canonical
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn is_empty(&self) -> bool {
        self.plan.plugins.is_empty()
    }

    /// Capabilities addressed to `product` that it does not implement.
    pub fn refusals(&self, product: &str, output: Option<&str>) -> Vec<PluginRefusal> {
        let registry = plugin_registry();
        let mut refused = Vec::new();
        for plugin in &self.plan.plugins {
            for capability in &plugin.capabilities {
                let Some(known) = registry.capabilities.get(capability.id.as_str()) else {
                    continue;
                };
                if !known.consumers.iter().any(|consumer| consumer == product) {
                    continue;
                }
                let reason = if !known.admitted_by.iter().any(|admitted| admitted == product) {
                    Some(String::from("unadmitted"))
                } else {
                    output
                        .filter(|output| {
                            plugin.supports.as_ref().is_some_and(|supports| {
                                !supports.outputs.iter().any(|value| value == output)
                            })
                        })
                        .map(|output| cstr!("output:{output}"))
                };
                if let Some(reason) = reason {
                    refused.push(PluginRefusal {
                        plugin: plugin.name.clone(),
                        capability: capability.id.clone(),
                        key: capability.key.clone(),
                        reason,
                    });
                }
            }
        }
        refused
    }

    /// Fail closed when `product` cannot honor every capability addressed to it.
    pub fn admit(&self, product: &str, output: Option<&str>) -> Result<(), PluginPlanError> {
        if !plugin_registry()
            .products
            .iter()
            .any(|known| known == product)
        {
            return Err(PluginPlanError::new(
                "product-unknown",
                product,
                "is not a Vize product".into(),
            ));
        }
        let refusals = self.refusals(product, output);
        let Some(first) = refusals.first() else {
            return Ok(());
        };
        let more = match refusals.len() {
            1 => String::default(),
            count => cstr!(" (and {} more)", count - 1),
        };
        Err(match first.reason.strip_prefix("output:") {
            None => PluginPlanError::new(
                "capability-unadmitted",
                &first.plugin,
                cstr!(
                    "{product} does not implement {} `{}` yet{more}",
                    first.capability,
                    first.key
                ),
            ),
            Some(output) => PluginPlanError::new(
                "host-unsupported",
                &first.plugin,
                cstr!(
                    "{} `{}` does not support the {output} output{more}",
                    first.capability,
                    first.key
                ),
            ),
        })
    }
}

/// Verify one canonical plan value; `expected_digest` detects a stale or skewed plan.
pub fn verify_plugin_plan(
    value: &serde_json::Value,
    expected_digest: Option<&str>,
) -> Result<VerifiedPluginPlan, PluginPlanError> {
    let invalid = |message: String| PluginPlanError::new("plan-invalid", "plan", message);
    let canonical = canonical_json(value).map_err(invalid)?;
    let digest = canonical::digest(&canonical);
    if let Some(expected) = expected_digest
        && expected != digest.as_str()
    {
        return Err(PluginPlanError::new(
            "plan-stale",
            "plan",
            cstr!(
                "digest {digest} does not match the resolved plan {expected}; reload vite.config.*"
            ),
        ));
    }
    let plan = PluginPlan::deserialize(value).map_err(|error| invalid(cstr!("{error}")))?;
    let registry = plugin_registry();
    if plan.plan_schema != registry.plan_schema {
        return Err(invalid(cstr!(
            "schema {} is not supported (supported: {})",
            plan.plan_schema,
            registry.plan_schema
        )));
    }
    verify_plugins(&plan, registry)?;
    verify_order(&plan)?;
    verify_claims(&plan)?;
    Ok(VerifiedPluginPlan {
        plan,
        canonical,
        digest,
    })
}

fn verify_plugins(plan: &PluginPlan, registry: &PluginRegistry) -> Result<(), PluginPlanError> {
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
fn verify_order(plan: &PluginPlan) -> Result<(), PluginPlanError> {
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

fn verify_claims(plan: &PluginPlan) -> Result<(), PluginPlanError> {
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
