use vize_carton::FxHashMap;

use super::support::{Fixture, same_mtime_write};
use crate::corsa_bridge::vue_dependencies_alias::context::cache::fingerprint::SourceGuard;

const HOST: &str = "<script setup lang='ts'>import Widget from '@scope/ui'; import Other from './nested/Other.vue'; const count = 1;</script><template><Widget :label='String(count)'/><Other/></template>";
const MANIFEST: &str = "{\n  \"name\": \"@scope/ui\", \"version\": \"1.0.0\",\n  \"exports\": { \".\": \"./Widget.vue\" }\n}\n";

fn fixture() -> Fixture {
    let fixture = Fixture::new(HOST);
    fixture.write(
        "package.json",
        "{\n  \"name\": \"graph-editor\", \"private\": true\n}\n",
    );
    fixture.write("node_modules/@scope/ui/package.json", MANIFEST);
    fixture.write("node_modules/@scope/ui/Widget.vue", "<script setup lang='ts'>defineProps<{ label: string }>()</script><template>{{ label }}</template>\n");
    fixture.write("src/nested/Other.vue", "<script setup lang='ts'>import Widget from '@scope/ui';</script><template><Widget :value='1'/></template>\n");
    fixture.write("src/nested/node_modules/@scope/ui/package.json", MANIFEST);
    fixture.write("src/nested/node_modules/@scope/ui/Widget.vue", "<script setup lang='ts'>defineProps<{ value: number }>()</script><template>{{ value }}</template>\n");
    fixture
}

#[test]
fn source_only_takeover_retains_importer_scopes_raw_manifests_and_all_whole_projection_facts() {
    let fixture = fixture();
    let empty = FxHashMap::default();
    let first = fixture.open(HOST, &empty);
    let inputs = first.route_inputs.clone();
    assert!(!inputs.is_empty());
    let mirror = first.mirror.as_ref().unwrap();
    let old_host = mirror
        .preferred_materialized_path_for_original(&fixture.host)
        .unwrap();
    let other = mirror
        .preferred_materialized_path_for_original(&fixture.root.join("src/nested/Other.vue"))
        .unwrap();
    let scopes = [
        old_host.parent().unwrap().join("node_modules/@scope/ui"),
        other.parent().unwrap().join("node_modules/@scope/ui"),
    ];
    assert_ne!(scopes[0], scopes[1]);
    assert!(
        std::fs::read_to_string(scopes[0].join("Widget.vue.ts"))
            .unwrap()
            .contains("label: string")
    );
    assert!(
        std::fs::read_to_string(scopes[1].join("Widget.vue.ts"))
            .unwrap()
            .contains("value: number")
    );
    for scope in &scopes {
        assert_eq!(
            std::fs::read(scope.join("package.json")).unwrap(),
            MANIFEST.as_bytes()
        );
        assert!(scope.join("Widget.d.vue.ts").is_file());
    }
    drop(first);
    let edited = HOST.replace("count = 1", "count = 2");
    let updated = fixture.open(&edited, &empty);
    assert!(updated.graph_reused);
    assert_eq!(updated.route_inputs, inputs);
    let vue_count = updated
        .mirror
        .as_ref()
        .unwrap()
        .registered_original_paths_sorted()
        .iter()
        .filter(|path| {
            updated
                .mirror
                .as_ref()
                .unwrap()
                .find_by_original(path)
                .is_some_and(|file| file.source_map.sfc_map.is_some())
        })
        .count();
    assert!(vue_count >= 2);
    assert_eq!(
        updated.mirror.as_ref().unwrap().source_patch_work(),
        (vue_count, vue_count)
    );
    fixture.assert_cold_equal(&updated, &edited, &empty);
    for scope in scopes {
        assert_eq!(
            std::fs::read(scope.join("package.json")).unwrap(),
            MANIFEST.as_bytes()
        );
    }
}

#[test]
fn same_mtime_package_bytes_and_changed_package_occurrences_fall_back_to_cold() {
    let fixture = fixture();
    let empty = FxHashMap::default();
    drop(fixture.open(HOST, &empty));
    let manifest = fixture.root.join("node_modules/@scope/ui/package.json");
    same_mtime_write(&manifest, &MANIFEST.replace("1.0.0", "2.0.0"));
    assert_eq!(fixture.guard(HOST, &empty), Err(SourceGuard::PackageInput));
    let changed_manifest = fixture.open(HOST, &empty);
    assert!(!changed_manifest.graph_reused);
    fixture.assert_cold_equal(&changed_manifest, HOST, &empty);
    drop(changed_manifest);
    // Occurrences differ even though every prior strong filesystem stamp is current.
    let removed_import = HOST.replace("import Widget from '@scope/ui';", "const Widget = 'div';");
    let changed_occurrence = fixture.open(&removed_import, &empty);
    assert!(!changed_occurrence.graph_reused);
    fixture.assert_cold_equal(&changed_occurrence, &removed_import, &empty);
}

#[test]
fn editing_an_existing_package_source_never_reuses_stale_route_input_authority() {
    let fixture = fixture();
    let empty = FxHashMap::default();
    drop(fixture.open(HOST, &empty));
    fixture.write("node_modules/@scope/ui/Widget.vue", "<script setup lang='ts'>defineProps<{ other: boolean }>()</script><template>{{ other }}</template>\n");
    assert_eq!(fixture.guard(HOST, &empty), Err(SourceGuard::PackageInput));
    let changed = fixture.open(HOST, &empty);
    assert!(!changed.graph_reused);
    fixture.assert_cold_equal(&changed, HOST, &empty);
}

#[test]
fn an_unsaved_package_input_uses_the_complete_cold_route_producer() {
    let fixture = fixture();
    let empty = FxHashMap::default();
    drop(fixture.open(HOST, &empty));
    let package_source = fixture.root.join("node_modules/@scope/ui/Widget.vue");
    let unsaved = "<script setup lang='ts'>defineProps<{ unsaved: boolean }>()</script><template>{{ unsaved }}</template>\n";
    let overlays = FxHashMap::from_iter([(package_source, unsaved)]);
    let updated = fixture.open(HOST, &overlays);
    assert!(!updated.graph_reused);
    fixture.assert_cold_equal(&updated, HOST, &overlays);
}
