//! Directive analysis for template elements.
//!
//! Handles v-bind, v-on, v-if, and v-for directive processing
//! during template traversal.
//!
//! ## Submodules
//!
//! - [`v_bind`] - v-bind directive handling (`:key`, `$attrs`, callbacks)
//! - [`v_on`] - v-on directive handling (event handlers, inline callbacks)
//! - [`control_flow`] - v-if/v-for visiting and variable extraction

mod control_flow;
mod v_bind;
mod v_on;
