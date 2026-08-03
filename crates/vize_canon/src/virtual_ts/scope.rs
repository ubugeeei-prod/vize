//! Scope closure generation for virtual TypeScript.
//!
//! Generates TypeScript closures that mirror Vue's template scope hierarchy,
//! including v-for, v-slot, and event handler scopes. Uses recursive
//! tree-based generation so nested scopes are properly contained.

mod children;
mod closures;
mod component_events;
mod component_prop_checker;
mod component_prop_expressions;
mod component_prop_navigation;
mod component_props;
mod context;
mod emit;
mod empty_component_props;
mod event_handler;
mod event_scope;
mod expression_scanner;
mod globals;
mod handler_shape;
mod inline_callback_classifier;
mod vif_guard;

pub(crate) use closures::generate_scope_closures;
pub(crate) use component_prop_checker::is_inline_callback_prop;
pub(crate) use context::{GlobalComponentCheck, ScopeGenerationOptions};
pub(crate) use vif_guard::remove_enclosing_vif_guard_prefix;
