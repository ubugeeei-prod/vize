//! Expression and component prop check generation for virtual TypeScript.
//!
//! Handles generating TypeScript code for template expressions (with optional
//! v-if narrowing) and component prop value type assertions.

mod component_props;
mod reserved_props;
mod statements;
mod vif_chain;

#[cfg(test)]
mod tests;

pub(crate) use component_props::generate_component_prop_checks;
pub(crate) use reserved_props::rewrite_reserved_template_prop;
pub(crate) use statements::generate_expressions;
