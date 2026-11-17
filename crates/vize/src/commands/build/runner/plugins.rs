//! `vue.plugins` contributions `vize build` honors (#13074).
//!
//! The config parse already verified the plan and refused any capability
//! that a consuming product does not implement. The build admits the plan as
//! the compiler and lowers the declared directive table once. The table
//! carries each declaring plugin's supported outputs, so the SFC compiler
//! refuses a file whose resolved output (SSR, Vapor or VDOM) a plugin lacks.

use vize_l0::config::{ConfigDocument, DirectiveContracts};

use crate::config::plugin_plan::{PluginPlanError, VerifiedPluginPlan, document_plugin_plan};

/// The directive table of `document`; a refused plan ends the build.
pub(super) fn directive_contracts(document: &ConfigDocument) -> DirectiveContracts {
    document_plugin_plan(document)
        .and_then(|plan| {
            plan.as_ref()
                .map_or(Ok(DirectiveContracts::EMPTY), admitted)
        })
        .unwrap_or_else(|error| {
            eprintln!("\x1b[31mError:\x1b[0m {error}");
            std::process::exit(1);
        })
}

fn admitted(plan: &VerifiedPluginPlan) -> Result<DirectiveContracts, PluginPlanError> {
    plan.admit("compiler", None)?;
    plan.directive_table()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::admitted;
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
        let contracts = admitted(&tracking(None)).unwrap();
        let names: Vec<_> = contracts
            .as_slice()
            .iter()
            .map(|contract| (contract.plugin.as_str(), contract.name.as_str()))
            .collect();
        assert_eq!(names, [("@acme/tracking", "track")]);
    }

    #[test]
    fn the_table_refuses_exactly_the_outputs_a_plugin_lacks() {
        let table = admitted(&tracking(Some(&["vdom"]))).unwrap();
        assert_eq!(table.unsupported("vdom"), None);
        assert_eq!(
            [table.unsupported("ssr"), table.unsupported("vapor")],
            [
                Some("[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the ssr output".into()),
                Some("[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the vapor output".into()),
            ]
        );
    }
}
