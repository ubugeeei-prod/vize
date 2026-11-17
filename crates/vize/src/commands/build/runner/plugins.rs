//! `vue.plugins` contributions `vize build` honors (#13074).
//!
//! The config parse already verified the plan and refused any capability
//! that a consuming product does not implement. The build admits the plan as
//! the compiler, once for every output it produces, and lowers the declared
//! directive contracts into the table each file's compile reads.

use vize_l0::config::{ConfigDocument, DirectiveContracts};

use crate::config::plugin_plan::{PluginPlanError, VerifiedPluginPlan, document_plugin_plan};

/// The verified plan of `document`; a refused plan ends the build.
pub(super) fn verified_plan(document: &ConfigDocument) -> Option<VerifiedPluginPlan> {
    document_plugin_plan(document).unwrap_or_else(|error| fail(&error))
}

/// The directive table for a build producing SSR or VDOM output, plus Vapor
/// output when `vapor` files exist; a refusal ends the build.
pub(super) fn directive_contracts(
    plan: Option<&VerifiedPluginPlan>,
    ssr: bool,
    vapor: bool,
) -> DirectiveContracts {
    plan.map_or(Ok(DirectiveContracts::EMPTY), |plan| {
        admitted_directives(plan, ssr, vapor)
    })
    .unwrap_or_else(|error| fail(&error))
}

/// Fail closed when a plugin does not support an output the build produces.
fn admitted_directives(
    plan: &VerifiedPluginPlan,
    ssr: bool,
    vapor: bool,
) -> Result<DirectiveContracts, PluginPlanError> {
    let base = if ssr { "ssr" } else { "vdom" };
    for output in core::iter::once(base).chain(vapor.then_some("vapor")) {
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
        let contracts = admitted_directives(&tracking(None), false, true).unwrap();
        let names: Vec<_> = contracts
            .as_slice()
            .iter()
            .map(|contract| (contract.plugin.as_str(), contract.name.as_str()))
            .collect();
        assert_eq!(names, [("@acme/tracking", "track")]);
    }

    #[test]
    fn an_output_a_plugin_does_not_support_refuses_the_build() {
        let plan = tracking(Some(&["vdom"]));
        assert!(admitted_directives(&plan, false, false).is_ok());
        let refusals: Vec<_> = [(true, false), (false, true)]
            .into_iter()
            .map(|(ssr, vapor)| {
                admitted_directives(&plan, ssr, vapor)
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
