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
mod tests {
    use super::super::ServerState;
    use crate::{ide::CodeLensService, virtual_code::PhysicalOccurrences};
    use tower_lsp::lsp_types::Url;

    const SOURCE: &str = "<script setup>const id = 1</script><template>{{ id }}</template>";

    fn open(state: &ServerState, uri: &Url, source: &str) {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        state.update_virtual_docs(uri, source);
    }

    #[test]
    fn an_owned_snapshot_cannot_revalidate_after_edit_or_configuration_change() {
        let state = ServerState::new();
        let uri = Url::parse("file:///Packet.vue").unwrap();
        open(&state, &uri, SOURCE);
        let packet = state.binding_occurrence_facts(&uri, SOURCE).unwrap();
        let before = CodeLensService::get_lenses(&state, SOURCE, &uri);
        assert_eq!(before.len(), 1);
        *state.type_checker_options_api.write() = false;
        state.lsp_features.write().options_api = false;
        assert!(state.binding_occurrence_facts(&uri, SOURCE).is_none());
        state.update_virtual_docs(&uri, SOURCE);
        assert!(state.binding_occurrence_facts(&uri, SOURCE).is_some());
        *state.type_checker_legacy_vue2.write() = true;
        assert!(state.binding_occurrence_facts(&uri, SOURCE).is_none());
        state.update_virtual_docs(&uri, SOURCE);
        let legacy = state.binding_occurrence_facts(&uri, SOURCE).unwrap();
        assert!(legacy.is_legacy());
        assert!(legacy.authored().is_none());
        *state.type_checker_legacy_vue2.write() = false;
        state.update_virtual_docs(&uri, SOURCE);
        let current = state.binding_occurrence_facts(&uri, SOURCE).unwrap();
        assert!(!std::sync::Arc::ptr_eq(&packet, &current));
        let changed = SOURCE.replace("{{ id }}", "text");
        state
            .documents
            .open(uri.clone(), changed.clone(), 1, "vue".into());
        assert!(state.binding_occurrence_facts(&uri, SOURCE).is_none());
        assert!(state.binding_occurrence_facts(&uri, &changed).is_none());
        state.update_virtual_docs(&uri, &changed);
        assert!(CodeLensService::get_lenses(&state, &changed, &uri).is_empty());
    }

    #[test]
    fn stale_or_refused_production_never_overwrites_the_newer_packet() {
        let state = ServerState::new();
        let uri = Url::parse("file:///Packet.vue").unwrap();
        open(&state, &uri, SOURCE);
        let old_revision = state.occurrence_source_revision(&uri, SOURCE);
        let old_config = state.occurrence_config();
        let changed = SOURCE.replace("{{ id }}", "{{ id }} {{ id }}");
        open(&state, &uri, &changed);
        let current = state.binding_occurrence_facts(&uri, &changed).unwrap();
        state.publish_occurrences(
            &uri,
            SOURCE,
            old_revision,
            old_config,
            Some(PhysicalOccurrences::default()),
        );
        state.publish_occurrences(&uri, &changed, old_revision, old_config, None);
        let after = state.binding_occurrence_facts(&uri, &changed).unwrap();
        assert!(std::sync::Arc::ptr_eq(&current, &after));
        open(
            &state,
            &uri,
            "<script setup>const id = 1; if(true){use(id)}</script><template>{{ id }}</template>",
        );
        assert!(state.binding_occurrences.get(&uri).is_none());
        assert!(state.binding_occurrence_facts(&uri, &changed).is_none());
        state.clear_virtual_docs();
        assert!(state.binding_occurrences.is_empty());
    }
}
