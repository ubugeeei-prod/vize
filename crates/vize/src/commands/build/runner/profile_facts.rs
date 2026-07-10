//! Compatibility build-profile facts for the legacy per-file entrypoint.

mod observations;
mod source;

pub(super) use observations::{record_atelier_cache_decision, record_atelier_profile_facts};
pub(super) use source::{FileProfileFacts, StatsCacheStatus, file_profile};
