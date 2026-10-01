//! Shared ordered glob semantics for lint execution and inspection.

#[cfg(test)]
pub(crate) use vize_carton::config::matcher::GlobSequence;
pub(crate) use vize_carton::config::matcher::{LintPlanScope, absolute_path, normalize_path};
