//! Expression and component prop check generation for virtual TypeScript.
//!
//! Handles generating TypeScript code for template expressions (with optional
//! v-if narrowing) and component prop value type assertions.

mod component_props;
#[cfg(test)]
mod component_props_tests;
mod native_props;
#[cfg(test)]
mod native_props_tests;
mod prop_sources;
mod reserved_props;
mod statements;
mod vif_chain;

#[cfg(test)]
mod tests;

pub(crate) use component_props::{ComponentPropSource, generate_component_prop_checks};
pub(crate) use native_props::{NativePropBindings, collect_native_prop_bindings};
pub(crate) use reserved_props::rewrite_reserved_template_prop;
pub(crate) use statements::{
    ExpressionListEmitContext, generate_expressions, generate_expressions_in_enclosing_guard,
};
