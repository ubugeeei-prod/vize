//! `vue.plugins` contributions `vize build` honors (#13074).
//!
//! The config parse already verified the plan and refused any capability
//! that a consuming product does not implement. Once the inputs are known,
//! the build admits the plan as the compiler for exactly the outputs those
//! inputs produce, and lowers the declared directive contracts into the
//! table each file's compile reads.

use std::path::Path;

use vize_l0::config::{ConfigDocument, DirectiveContracts};

use super::settings::CompileFileSettings;
use crate::config::plugin_plan::{PluginPlanError, VerifiedPluginPlan, document_plugin_plan};

/// The verified plan of `document`; a refused plan ends the build.
pub(super) fn verified_plan(document: &ConfigDocument) -> Option<VerifiedPluginPlan> {
    document_plugin_plan(document).unwrap_or_else(|error| fail(&error))
}

impl CompileFileSettings {
    /// Admit the plan for the outputs `inputs` compile to and lower its
    /// directive table; a refusal ends the build. Without a plan this does
    /// nothing.
    pub(super) fn admit_plugin_plan<'p>(&mut self, inputs: impl Iterator<Item = &'p Path>) {
        let Some(plan) = self.plugin_plan.as_ref() else {
            return;
        };
        let (mut vapor, mut other) = (false, false);
        for input in inputs {
            if self.compiler_for_path(input).vapor {
                vapor = true;
            } else {
                other = true;
            }
        }
        let base = if self.ssr { "ssr" } else { "vdom" };
        // An SSR render of a Vapor component is still SSR output.
        let outputs: Vec<&str> = [
            (other || (self.ssr && vapor)).then_some(base),
            vapor.then_some("vapor"),
        ]
        .into_iter()
        .flatten()
        .collect();
        self.directive_contracts =
            admitted_directives(plan, &outputs).unwrap_or_else(|error| fail(&error));
    }
}

/// Fail closed when a plugin does not support an output the build produces.
fn admitted_directives(
    plan: &VerifiedPluginPlan,
    outputs: &[&str],
) -> Result<DirectiveContracts, PluginPlanError> {
    for output in outputs {
        plan.admit("compiler", Some(output))?;
    }
    Ok(DirectiveContracts::new(plan.directives()?))
}

fn fail(error: &dyn core::fmt::Display) -> ! {
    eprintln!("\x1b[31mError:\x1b[0m {error}");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::admitted_directives;
    use crate::config::plugin_plan::{VerifiedPluginPlan, verify_plugin_plan};

    /// The declared-capabilities fixture reduced to its directive plugin.
    fn tracking(outputs: Option<&[&str]>) -> VerifiedPluginPlan {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../../tests/_fixtures/plugin-plan/resolve/declared-capabilities.json"
        ))
        .unwrap();
        let mut plugin = fixture["expect"]["plan"]["plugins"][2].clone();
        plugin["index"] = json!(0);
        if let Some(outputs) = outputs {
            plugin["supports"]["outputs"] = json!(outputs);
        }
        let plan = json!({
            "claims": [{ "key": "vue.directive:track", "owner": "@acme/tracking" }],
            "planSchema": 1,
            "plugins": [plugin],
        });
        verify_plugin_plan(&plan, None).unwrap()
    }

    #[test]
    fn the_build_lowers_the_declared_directives_once() {
        let contracts = admitted_directives(&tracking(None), &["vdom", "vapor"]).unwrap();
        let names: Vec<_> = contracts
            .as_slice()
            .iter()
            .map(|contract| (contract.plugin.as_str(), contract.name.as_str()))
            .collect();
        assert_eq!(names, [("@acme/tracking", "track")]);
    }

    #[test]
    fn only_outputs_the_build_produces_are_admitted() {
        let vdom_only = tracking(Some(&["vdom"]));
        assert!(admitted_directives(&vdom_only, &["vdom"]).is_ok());
        // A Vapor-only build never asks a Vapor plugin for VDOM support.
        assert!(admitted_directives(&tracking(Some(&["vapor"])), &["vapor"]).is_ok());
        let refusals: Vec<_> = ["ssr", "vapor"]
            .into_iter()
            .map(|output| {
                admitted_directives(&vdom_only, &[output])
                    .unwrap_err()
                    .to_string()
            })
            .collect();
        assert_eq!(
            refusals,
            [
                "[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the ssr output",
                "[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the vapor output",
            ]
        );
    }
}
