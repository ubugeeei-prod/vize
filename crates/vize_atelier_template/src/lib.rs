//! Independent frontend for raw Vue template sources.
//!
//! A raw template is not modeled as a synthetic SFC. Its source-faithful
//! Relief syntax, transformed syntax, Flow graph, Rendu HIR, and selected
//! backend output are all independently demandable Atlas products.

#![allow(deprecated)]

mod atlas;
pub mod graph_frontend;

pub use atlas::*;
pub use graph_frontend::{
    TemplateGraphAdapter, TemplateGraphAdapterError, lower_relief_snapshot_to_rendu,
    project_relief_snapshot_to_flow,
};
