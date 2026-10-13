//! Actual CLI consumers of the original explicit-context corpus.
#![cfg(test)]
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "integration fixtures preserve whole standard JSON and source strings"
)]

#[path = "component_context/cli.rs"]
mod cli;
#[path = "component_context/support.rs"]
mod support;
