use vize_carton::FxHashMap;

use super::support::{CHILD, Fixture, HOST, immutable_documents, same_mtime_write};

#[test]
fn changed_host_regenerates_retained_vue_and_keeps_old_immutable_catalog_facts() {
    let fixture = Fixture::new(HOST);
    let overlays = FxHashMap::default();
    let original = fixture.project(HOST, &overlays);
    let first = fixture.open(HOST, &overlays);
    assert!(first.query_surface.get().is_some());
    let old_catalog = first.source_catalog.clone();
    let old_documents = first.materialized_sources();
    drop(first);
    let edited = HOST.replace("count = 1", "count = 2");
    let updated = fixture.open(&edited, &overlays);
    assert!(
        updated.graph_reused,
        "the control must exercise exclusive graph takeover"
    );
    assert!(
        updated.query_surface.get().is_none(),
        "query bytes belong to the new revision"
    );
    assert_eq!(updated.mirror.as_ref().unwrap().source_patch_work(), (2, 2));
    fixture.assert_cold_equal(&updated, &edited, &overlays);
    for source in old_documents {
        let old = old_catalog.get(&source.materialized_path).unwrap();
        assert_eq!(old.source, source.source);
        assert_eq!(old.code, source.code);
        assert_eq!(old.mapping, source.mapping);
        assert_eq!(old.import_source_map, source.import_source_map);
    }
    let project = fixture.project(&edited, &overlays);
    assert!(project.host.code.contains("count = 2"));
    assert!(original.host.code.contains("count = 1"));
}

#[test]
fn same_mtime_shared_source_edit_regenerates_all_retained_vue_and_changed_script() {
    let host = "<script setup lang='ts'>import Child from './Child.vue'; import Other from './Other.vue';</script><template><Child/><Other/></template>";
    let fixture = Fixture::new(host);
    fixture.write("src/Other.vue", "<script setup lang='ts'>import { value } from './value';</script><template>{{ value }}</template>");
    let child = "<script setup lang='ts'>import { value } from './value';</script><template>{{ value }}</template>";
    fixture.write("src/Child.vue", child);
    let value = fixture.write("src/value.ts", "export const value = 1;\n");
    drop(fixture.open(host, &FxHashMap::default()));
    same_mtime_write(&value, "export const value = 2;\n");
    let updated = fixture.open(host, &FxHashMap::default());
    assert!(updated.graph_reused);
    assert_eq!(updated.mirror.as_ref().unwrap().source_patch_work(), (4, 4));
    fixture.assert_cold_equal(&updated, host, &FxHashMap::default());
}

#[test]
fn known_source_delete_exercises_takeover_and_removes_all_owned_queries_and_artifacts() {
    let fixture = Fixture::new(HOST);
    let child = fixture.root.join("src/Child.vue");
    let overlays = FxHashMap::from_iter([(child.clone(), CHILD)]);
    let first = fixture.open(HOST, &overlays);
    let path = first
        .mirror
        .as_ref()
        .unwrap()
        .preferred_materialized_path_for_original(&child)
        .unwrap();
    assert!(path.is_file());
    drop(first);
    std::fs::remove_file(&child).unwrap();
    crate::corsa_bridge::vue_dependencies_alias::AliasContext::forget_cached_sources(
        &fixture.session,
        &[child.clone()],
    );
    let empty = FxHashMap::default();
    assert_eq!(fixture.guard(HOST, &empty).unwrap(), vec![child.clone()]);
    let deleted = fixture.open(HOST, &empty);
    assert!(
        deleted.graph_reused,
        "whole equality must not silently receive cold fallback credit"
    );
    assert!(!path.exists());
    assert!(deleted.source_catalog.get(&path).is_none());
    assert!(
        deleted
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&child)
            .is_none()
    );
    assert!(
        !deleted
            .mirror
            .as_ref()
            .unwrap()
            .editor_query_paths(&fixture.host)
            .contains(&path)
    );
    assert_eq!(deleted.mirror.as_ref().unwrap().source_patch_work(), (1, 3));
    fixture.assert_cold_equal(&deleted, HOST, &empty);
    drop(deleted);
    fixture.write("src/Child.vue", CHILD);
    let restored = fixture.open(HOST, &empty);
    assert!(
        !restored.graph_reused,
        "a newly created negative probe must rebuild cold"
    );
    fixture.assert_cold_equal(&restored, HOST, &empty);
}

#[test]
fn removing_one_shared_owner_keeps_the_other_and_new_relative_targets_join_exactly() {
    let host = "<script setup lang='ts'>import A from './A.vue'; import B from './B.vue';</script><template><A/><B/></template>";
    let fixture = Fixture::new(host);
    fixture.write("src/A.vue", "<script setup lang='ts'>import { value } from './value';</script><template>{{ value }}</template>");
    fixture.write("src/B.vue", "<script setup lang='ts'>import { value } from './value';</script><template>{{ value }}</template>");
    fixture.write("src/value.ts", "export const value = 1;\n");
    drop(fixture.open(host, &FxHashMap::default()));
    fixture.write(
        "src/A.vue",
        "<script setup lang='ts'>const value = 3;</script><template>{{ value }}</template>",
    );
    let shared = fixture.open(host, &FxHashMap::default());
    assert!(shared.graph_reused);
    assert!(
        shared
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&fixture.root.join("src/value.ts"))
            .is_some()
    );
    fixture.assert_cold_equal(&shared, host, &FxHashMap::default());
    drop(shared);
    fixture.write("src/new.ts", "export const extra = 'é🦀';\n");
    let edited = host.replace(
        "</script>",
        "import { extra } from './new'; void extra;</script>",
    );
    let added = fixture.open(&edited, &FxHashMap::default());
    assert!(added.graph_reused);
    fixture.assert_cold_equal(&added, &edited, &FxHashMap::default());
}

#[test]
fn a_live_immutable_context_reader_forces_cold_without_altering_its_whole_facts() {
    let fixture = Fixture::new(HOST);
    let first = fixture.open(HOST, &FxHashMap::default());
    let before = immutable_documents(&first);
    let catalog = first.source_catalog.clone();
    let old_documents = first.materialized_sources();
    let edited = HOST.replace("count = 1", "count = 2");
    let second = fixture.open(&edited, &FxHashMap::default());
    assert!(!second.graph_reused);
    // Immutable readers retain source facts even after disk materialization changes.
    assert_eq!(before, immutable_documents(&first));
    for source in old_documents {
        let old = catalog.get(&source.materialized_path).unwrap();
        assert_eq!(old.source, source.source);
        assert_eq!(old.code, source.code);
        assert_eq!(old.mapping, source.mapping);
        assert_eq!(old.import_source_map, source.import_source_map);
    }
    fixture.assert_cold_equal(&second, &edited, &FxHashMap::default());
}
