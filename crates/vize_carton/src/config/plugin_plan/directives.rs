//! Lowers declared `vue.directive` capabilities into the L0 contract table (#13074).

use super::{PluginPlanError, VerifiedPluginPlan};
use crate::config::{DirectiveContract, DirectiveModifier};
use crate::cstr;

impl VerifiedPluginPlan {
    /// Every declared directive, in plan order.
    pub fn directives(&self) -> Result<Vec<DirectiveContract>, PluginPlanError> {
        let mut contracts = Vec::new();
        for plugin in &self.plan().plugins {
            for capability in &plugin.capabilities {
                if capability.id != "vue.directive" {
                    continue;
                }
                let invalid = || {
                    PluginPlanError::new(
                        "plan-invalid",
                        &plugin.name,
                        cstr!("malformed vue.directive `{}`", capability.key),
                    )
                };
                let modifiers = capability
                    .data
                    .get("modifiers")
                    .and_then(serde_json::Value::as_object)
                    .ok_or_else(invalid)?
                    .iter()
                    .map(|(name, contract)| {
                        Some(DirectiveModifier {
                            name: name.as_str().into(),
                            optional: contract.get("optional")?.as_bool()?,
                            default: contract.get("default")?.as_bool()?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()
                    .ok_or_else(invalid)?;
                contracts.push(DirectiveContract {
                    plugin: plugin.name.clone(),
                    name: capability.key.clone(),
                    modifiers,
                });
            }
        }
        Ok(contracts)
    }
}
