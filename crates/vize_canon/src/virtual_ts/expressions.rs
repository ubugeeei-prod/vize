//! Expression and component prop check generation for virtual TypeScript.
//!
//! Handles generating TypeScript code for template expressions (with optional
//! v-if narrowing) and component prop value type assertions.

mod component_props;
#[cfg(test)]
mod component_props_tests;
mod directive_values;
#[cfg(test)]
mod directive_values_tests;
mod generic_props_call;
mod native_props;
#[cfg(test)]
mod native_props_tests;
mod prop_sources;
mod reserved_props;
#[cfg(test)]
mod sequence_prop_tests;
mod spread_reserved_props;
mod statements;
mod value_checks;
mod vif_chain;

#[cfg(test)]
mod tests;

pub(crate) use component_props::{ComponentPropSource, generate_component_prop_checks};
pub(crate) use reserved_props::rewrite_reserved_template_prop;
pub(crate) use statements::{
    ExpressionListEmitContext, generate_expressions, generate_expressions_in_enclosing_guard,
};
pub(crate) use value_checks::{TemplateValueCheckTables, TemplateValueChecks};
