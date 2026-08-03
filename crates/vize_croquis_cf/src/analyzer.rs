//! Main cross-file analyzer.
//!
//! Orchestrates all cross-file analysis passes and manages the module registry
//! and dependency graph.

mod core;
mod types;

pub use core::CrossFileAnalyzer;
pub use types::{CrossFileOptions, CrossFileResult, CrossFileStats};

#[cfg(test)]
#[path = "analyzer/tests_basic.rs"]
mod tests_basic;

#[cfg(test)]
#[path = "analyzer/tests_props_validation_identity.rs"]
mod tests_props_validation_identity;

#[cfg(test)]
#[path = "analyzer/tests_fallthrough.rs"]
mod tests_fallthrough;

#[cfg(test)]
#[path = "analyzer/tests_element_id.rs"]
mod tests_element_id;

#[cfg(test)]
#[path = "analyzer/tests_complexity_effect_graph.rs"]
mod tests_complexity_effect_graph;

#[cfg(test)]
#[path = "analyzer/tests_complexity_effect_graph_overflow.rs"]
mod tests_complexity_effect_graph_overflow;

#[cfg(test)]
mod tests_provide_inject;

#[cfg(test)]
mod tests_reactivity_props;

#[cfg(test)]
mod tests_race_conditions;

#[cfg(test)]
#[path = "analyzer/tests_single_file.rs"]
mod tests_single_file;

#[cfg(test)]
mod tests_snapshots;
