//! Native verification of the canonical plugin plan.
//!
//! `vue.plugins` is resolved once, by `loader/plugin-plan.js`, into canonical
//! data. Rust products never rebuild that plan: they verify the canonical
//! bytes, the registry versions, the ordering, and the ownership claims, and
//! refuse any capability addressed to them that they do not implement.

mod canonical;
mod directives;
mod invariants;
mod registry;
#[cfg(test)]
mod tests;

use serde::Deserialize;

use crate::{String, cstr};

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

    /// Refuse a capability until every product that consumes it implements
    /// it, so no product can silently ignore one another product honors.
    pub fn refuse_unimplemented(&self) -> Result<(), PluginPlanError> {
        let registry = plugin_registry();
        for plugin in &self.plan.plugins {
            for capability in &plugin.capabilities {
                let Some(known) = registry.capabilities.get(capability.id.as_str()) else {
                    continue;
                };
                let missing: Vec<&str> = known
                    .consumers
                    .iter()
                    .filter(|consumer| !known.admitted_by.contains(consumer))
                    .map(String::as_str)
                    .collect();
                if missing.is_empty() {
                    continue;
                }
                let subject = cstr!("{} `{}`", capability.id, capability.key);
                let refusal = if known.admitted_by.is_empty() {
                    cstr!("no Vize product implements {subject} yet")
                } else {
                    cstr!("{subject} is not implemented by {} yet", missing.join(", "))
                };
                return Err(PluginPlanError::new(
                    "capability-unadmitted",
                    &plugin.name,
                    cstr!("{refusal}; run `vize plugin inspect` for the plan"),
                ));
            }
        }
        Ok(())
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

/// Verify the plan a Vite host projected into `document`; `None` when it has none.
///
/// A projected plan is never empty, and every capability in it must be
/// implemented by at least one product, matching the JS projection.
pub fn document_plugin_plan(
    document: &crate::config::ConfigDocument,
) -> Result<Option<VerifiedPluginPlan>, PluginPlanError> {
    let Some(projected) = document.plugin_plan() else {
        return Ok(None);
    };
    let invalid = |message: &str| PluginPlanError::new("plan-invalid", "plan", message.into());
    let (Some(digest), Some(plan)) = (
        projected.get("digest").and_then(serde_json::Value::as_str),
        projected.get("plan"),
    ) else {
        return Err(invalid("a projected plan is { digest, plan }"));
    };
    if projected.as_object().map(serde_json::Map::len) != Some(2) {
        return Err(invalid("a projected plan is { digest, plan }"));
    }
    let verified = verify_plugin_plan(plan, Some(digest))?;
    if verified.is_empty() {
        return Err(invalid("an empty plan is never projected"));
    }
    verified.refuse_unimplemented()?;
    Ok(Some(verified))
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
    invariants::verify_plugins(&plan, registry)?;
    invariants::verify_order(&plan)?;
    invariants::verify_claims(&plan)?;
    Ok(VerifiedPluginPlan {
        plan,
        canonical,
        digest,
    })
}
