//! Lowers declared `vue.directive` capabilities into the L0 contract table (#13074).

use super::{PluginPlanError, VerifiedPluginPlan};
use crate::config::{DirectiveContract, DirectiveContracts, DirectiveModifier};
use crate::cstr;

impl VerifiedPluginPlan {
    /// The shared table compilers read: every declared directive, and the
    /// outputs each declaring plugin supports, so a template compiled to an
    /// output its plugin lacks fails closed.
    pub fn directive_table(&self) -> Result<DirectiveContracts, PluginPlanError> {
        let contracts = self.directives()?;
        let outputs = self
            .plan()
            .plugins
            .iter()
            .filter(|plugin| {
                contracts
                    .iter()
                    .any(|contract| contract.plugin == plugin.name)
            })
            .filter_map(|plugin| {
                let supports = plugin.supports.as_ref()?;
                Some((plugin.name.clone(), supports.outputs.clone()))
            })
            .collect();
        Ok(DirectiveContracts::with_outputs(contracts, outputs))
    }

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
