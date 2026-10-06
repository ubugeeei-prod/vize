//! Revision-bound facts from checker production; requests never analyze text.

use super::{LspFeatureConfig, ServerState};
use crate::virtual_code::PhysicalOccurrences;
use std::sync::Arc;
use tower_lsp::lsp_types::Url;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct OccurrenceConfig {
    pub(super) features: LspFeatureConfig,
    options_api: bool,
    pub(super) legacy_vue2: bool,
}

pub(crate) struct CachedOccurrences {
    revision: u64,
    config: OccurrenceConfig,
    facts: OccurrenceProfile,
}

enum OccurrenceProfile {
    Authored(PhysicalOccurrences),
    Legacy,
}

impl CachedOccurrences {
    pub(crate) fn authored(&self) -> Option<&PhysicalOccurrences> {
        match &self.facts {
            OccurrenceProfile::Authored(facts) => Some(facts),
            OccurrenceProfile::Legacy => None,
        }
    }

    pub(crate) fn is_legacy(&self) -> bool {
        matches!(&self.facts, OccurrenceProfile::Legacy)
    }
}

impl ServerState {
    pub(super) fn occurrence_config(&self) -> OccurrenceConfig {
        OccurrenceConfig {
            features: self.lsp_features(),
            options_api: self.options_api_enabled(),
            legacy_vue2: self.legacy_vue2_enabled(),
        }
    }

    pub(super) fn occurrence_source_revision(&self, uri: &Url, source: &str) -> Option<u64> {
        let document = self.documents.get(uri)?;
        (document.content == source).then_some(document.revision())
    }

    pub(super) fn publish_occurrences(
        &self,
        uri: &Url,
        source: &str,
        revision: Option<u64>,
        config: OccurrenceConfig,
        facts: Option<PhysicalOccurrences>,
    ) {
        self.publish_occurrence_profile(
            uri,
            source,
            revision,
            config,
            facts.map(OccurrenceProfile::Authored),
        );
    }

    pub(super) fn publish_legacy_occurrences(
        &self,
        uri: &Url,
        source: &str,
        revision: Option<u64>,
        config: OccurrenceConfig,
    ) {
        if config.features.references || config.features.code_lens {
            self.publish_occurrence_profile(
                uri,
                source,
                revision,
                config,
                Some(OccurrenceProfile::Legacy),
            );
        }
    }

    fn publish_occurrence_profile(
        &self,
        uri: &Url,
        source: &str,
        revision: Option<u64>,
        config: OccurrenceConfig,
        facts: Option<OccurrenceProfile>,
    ) {
        let Some((revision, facts)) = revision.zip(facts) else {
            return;
        };
        if self.occurrence_source_revision(uri, source) != Some(revision)
            || self.occurrence_config() != config
        {
            return;
        }
        let packet = Arc::new(CachedOccurrences {
            revision,
            config,
            facts,
        });
        self.binding_occurrences
            .entry(uri.clone())
            .and_modify(|entry| {
                if entry.revision <= revision {
                    *entry = Arc::clone(&packet);
                }
            })
            .or_insert(packet);
    }

    /// Owned snapshot only, with no document or map guard escaping to an await.
    pub(crate) fn binding_occurrence_facts(
        &self,
        uri: &Url,
        source: &str,
    ) -> Option<Arc<CachedOccurrences>> {
        let revision = self.occurrence_source_revision(uri, source)?;
        let config = self.occurrence_config();
        let packet = self.binding_occurrences.get(uri).and_then(|entry| {
            (entry.revision == revision && entry.config == config)
                .then(|| Arc::clone(entry.value()))
        })?;
        (self.occurrence_source_revision(uri, source) == Some(revision)
            && self.occurrence_config() == config)
            .then_some(packet)
    }
}

#[cfg(test)]
mod tests;
