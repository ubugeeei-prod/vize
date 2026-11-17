//! `vue.plugins` contributions `vize check` honors (#13074).
//!
//! The type checker admits the plan once and lowers the declared directive
//! table its virtual TypeScript reads: each declared directive's hook
//! receives the default modifiers the compiler emits, and a binding that
//! breaks its contract is a diagnostic.

use vize_l0::config::{ConfigDocument, DirectiveContracts};

use crate::config::plugin_plan::{PluginPlanError, document_plugin_plan};

/// The directive table of `document`; a refused plan ends the check.
pub(super) fn directive_contracts(document: &ConfigDocument) -> DirectiveContracts {
    admitted_directives(document).unwrap_or_else(|error| {
        eprintln!("\x1b[31mError:\x1b[0m {error}");
        std::process::exit(2);
    })
}

fn admitted_directives(document: &ConfigDocument) -> Result<DirectiveContracts, PluginPlanError> {
    let Some(plan) = document_plugin_plan(document)? else {
        return Ok(DirectiveContracts::EMPTY);
    };
    plan.admit("typecheck", None)?;
    Ok(DirectiveContracts::new(plan.directives()?))
}
