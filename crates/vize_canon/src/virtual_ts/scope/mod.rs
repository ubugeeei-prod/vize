//! Scope closure generation for virtual TypeScript.
//!
//! Generates TypeScript closures that mirror Vue's template scope hierarchy,
//! including v-for, v-slot, and event handler scopes. Uses recursive
//! tree-based generation so nested scopes are properly contained.

mod closures;
mod component_events;
mod component_props;
mod context;
mod emit;
mod event_handler;
mod globals;

pub(crate) use closures::generate_scope_closures;
pub(crate) use context::ScopeGenerationOptions;
