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
    assert!(state.binding_occurrence_facts(&uri, SOURCE).is_none());
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
