//! Build-profile facts for Source Atlas and per-file source plates.

mod atlas;
mod source;

pub(super) use atlas::{record_atelier_cache_decision, record_atelier_profile_facts};
pub(super) use source::{FileProfileFacts, StatsCacheStatus, file_profile};
