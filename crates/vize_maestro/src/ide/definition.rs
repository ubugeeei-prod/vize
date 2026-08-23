//! Go-to-definition for Vue SFC template bindings, components, imports, and Corsa.
mod art;
pub mod bindings;
mod component_import;
mod component_model;
#[cfg(all(test, feature = "native"))]
mod corsa_tests;
pub(crate) mod helpers;
mod html;
#[cfg(all(test, feature = "native"))]
mod html_tests;
pub(crate) mod import_resolver;
mod inline_art;
mod module_specifier;
pub(crate) mod script;
mod service;
mod slot;
mod template;
#[cfg(test)]
mod tests;

use super::IdeContext;
pub use bindings::{BindingKind, BindingLocation, extract_bindings_with_locations};

/// Definition service for providing go-to-definition functionality.
pub struct DefinitionService;
