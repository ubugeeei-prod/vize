use vize_carton::FxHashMap;

use super::support::{CHILD, Fixture, HOST, same_mtime_write};
use crate::corsa_bridge::vue_dependencies_alias::context::cache::fingerprint::SourceGuard;

#[test]
fn same_mtime_config_and_generation_settings_keep_the_original_cold_authority() {
    let mut fixture = Fixture::new(HOST);
    let empty = FxHashMap::default();
    drop(fixture.open(HOST, &empty));
    let config = fixture.root.join("tsconfig.json");
    let original = std::fs::read_to_string(&config).unwrap();
    same_mtime_write(&config, &original.replace("src/*", "lib/*"));
    assert_eq!(fixture.guard(HOST, &empty), Err(SourceGuard::Configuration));
    let changed_config = fixture.open(HOST, &empty);
    assert!(!changed_config.graph_reused);
    fixture.assert_cold_equal(&changed_config, HOST, &empty);
    drop(changed_config);
    fixture.options.preserve_event_navigation = true;
    assert_eq!(fixture.guard(HOST, &empty), Err(SourceGuard::Settings));
    let changed_settings = fixture.open(HOST, &empty);
    assert!(!changed_settings.graph_reused);
    fixture.assert_cold_equal(&changed_settings, HOST, &empty);
}

#[test]
fn new_missing_and_higher_priority_companions_cannot_receive_warm_credit() {
    for (specifier, existing, created) in [
        ("./missing", None, "src/missing.ts"),
        ("./value.js", Some("src/value.js"), "src/value.ts"),
    ] {
        let host = format!(
            "<script setup lang='ts'>import {{ value }} from '{specifier}';</script><template>{{{{ value }}}}</template>"
        );
        let fixture = Fixture::new(&host);
        if let Some(existing) = existing {
            fixture.write(existing, "export const value = 'runtime';\n");
        }
        let empty = FxHashMap::default();
        drop(fixture.open(&host, &empty));
        fixture.write(created, "export const value = 'new identity';\n");
        assert_eq!(
            fixture.guard(&host, &empty),
            Err(SourceGuard::ResolutionInput)
        );
        let updated = fixture.open(&host, &empty);
        assert!(!updated.graph_reused);
        fixture.assert_cold_equal(&updated, &host, &empty);
    }
}

#[test]
fn directory_stamp_changes_remain_strict_even_when_the_registered_leaf_was_deleted() {
    let host = "<script setup lang='ts'>import { value } from './folder';</script><template>{{ value }}</template>";
    let fixture = Fixture::new(host);
    let leaf = fixture.write("src/folder/index.ts", "export const value = 1;\n");
    let empty = FxHashMap::default();
    drop(fixture.open(host, &empty));
    std::fs::remove_file(leaf).unwrap();
    // Pin a real directory metadata change independently of timestamp granularity.
    std::fs::File::open(fixture.root.join("src/folder"))
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(std::time::SystemTime::UNIX_EPOCH))
        .unwrap();
    assert_eq!(
        fixture.guard(host, &empty),
        Err(SourceGuard::ResolutionInput)
    );
    let deleted = fixture.open(host, &empty);
    assert!(
        !deleted.graph_reused,
        "source deletion never weakens a non-source stamp"
    );
    fixture.assert_cold_equal(&deleted, host, &empty);
}

#[test]
fn known_unsaved_buffer_edits_take_over_but_close_restores_disk_through_cold() {
    let fixture = Fixture::new(HOST);
    let empty = FxHashMap::default();
    drop(fixture.open(HOST, &empty));
    let child = fixture.root.join("src/Child.vue");
    let unsaved = CHILD.replace("'before'", "'unsaved'");
    let overlays = FxHashMap::from_iter([(child.clone(), unsaved.as_str())]);
    let edited = fixture.open(HOST, &overlays);
    assert!(edited.graph_reused);
    assert_eq!(
        edited
            .mirror
            .as_ref()
            .unwrap()
            .original_content_for_virtual(
                &edited
                    .mirror
                    .as_ref()
                    .unwrap()
                    .find_by_original(&child)
                    .unwrap()
                    .virtual_path
            ),
        Some(unsaved.as_str())
    );
    fixture.assert_cold_equal(&edited, HOST, &overlays);
    drop(edited);
    assert_eq!(fixture.guard(HOST, &empty), Err(SourceGuard::ClosedOverlay));
    let closed = fixture.open(HOST, &empty);
    assert!(!closed.graph_reused);
    fixture.assert_cold_equal(&closed, HOST, &empty);
    drop(closed);
    let reopened = fixture.open(HOST, &overlays);
    assert!(reopened.graph_reused);
    fixture.assert_cold_equal(&reopened, HOST, &overlays);
}

#[test]
fn registered_source_rename_releases_the_old_identity_and_joins_the_new_whole_document() {
    let fixture = Fixture::new(HOST);
    let empty = FxHashMap::default();
    let original = fixture.open(HOST, &empty);
    let old_path = original
        .mirror
        .as_ref()
        .unwrap()
        .preferred_materialized_path_for_original(&fixture.root.join("src/Child.vue"))
        .unwrap();
    drop(original);
    std::fs::rename(
        fixture.root.join("src/Child.vue"),
        fixture.root.join("src/Renamed.vue"),
    )
    .unwrap();
    let source = HOST.replace("./Child.vue", "./Renamed.vue");
    let renamed = fixture.open(&source, &empty);
    assert!(renamed.graph_reused);
    assert!(!old_path.exists());
    assert!(
        renamed
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&fixture.root.join("src/Renamed.vue"))
            .is_some()
    );
    fixture.assert_cold_equal(&renamed, &source, &empty);
}

#[test]
fn unrelated_requested_membership_retains_the_original_cold_constructor() {
    let fixture = Fixture::new(HOST);
    let empty = FxHashMap::default();
    drop(fixture.open(HOST, &empty));
    let source = "<template>unrelated</template>";
    let path = fixture.write("src/Unrelated.vue", source);
    let requested = [(path, source)];
    let updated = crate::corsa_bridge::vue_dependencies_alias::AliasContext::for_hosts_cached(
        &fixture.host,
        HOST,
        &empty,
        &requested,
        fixture.options,
        fixture.environment(),
    )
    .unwrap();
    assert!(!updated.graph_reused);
    let session = crate::corsa_bridge::EditorMirrorSession::new();
    let cold = crate::corsa_bridge::vue_dependencies_alias::AliasContext::for_hosts_cached(
        &fixture.host,
        HOST,
        &empty,
        &requested,
        fixture.options,
        crate::corsa_bridge::vue_document::CorsaProjectEnvironment {
            editor_session: &session,
            ..fixture.environment()
        },
    )
    .unwrap();
    assert_eq!(
        super::support::facts(&updated, &fixture.host),
        super::support::facts(&cold, &fixture.host)
    );
}

#[test]
fn deleting_a_first_party_alias_target_preserves_cold_package_lookup_provenance() {
    let host = HOST.replace("./Child.vue", "@/Child.vue");
    let fixture = Fixture::new(&host);
    let empty = FxHashMap::default();
    drop(fixture.open(&host, &empty));
    std::fs::remove_file(fixture.root.join("src/Child.vue")).unwrap();
    let deleted = fixture.open(&host, &empty);
    assert!(
        !deleted.graph_reused,
        "an unresolved paths alias may introduce new package inputs"
    );
    fixture.assert_cold_equal(&deleted, &host, &empty);
}

#[test]
fn removing_a_source_derived_ambient_stub_restores_the_exact_cold_options() {
    let host = HOST.replace("./Child.vue", "./Example.art.vue");
    let fixture = Fixture::new(&host);
    fixture.write("src/Example.art.vue", "<template>art</template>\n");
    let empty = FxHashMap::default();
    drop(fixture.open(&host, &empty));
    let edited = HOST.replace("import Child from './Child.vue';", "const Child = 'div';");
    let removed = fixture.open(&edited, &empty);
    assert!(!removed.graph_reused);
    fixture.assert_cold_equal(&removed, &edited, &empty);
}

#[test]
fn an_already_open_unrelated_buffer_uses_unsaved_bytes_when_it_becomes_reachable() {
    let fixture = Fixture::new(HOST);
    let path = fixture.write(
        "src/Unrelated.vue",
        "<script setup lang='ts'>const label = 'disk';</script><template>{{ label }}</template>\n",
    );
    let unsaved = "<!-- 🦀 -->\r\n<script setup lang='ts'>const label = 'unsaved';</script>\r\n<template>{{ label }}</template>\r\n";
    let overlays = FxHashMap::from_iter([(path.clone(), unsaved)]);
    let first = fixture.open(HOST, &overlays);
    assert!(
        first
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&path)
            .is_none()
    );
    drop(first);
    let edited = HOST.replace(
        "const count = 1;",
        "import Unrelated from './Unrelated.vue'; const count = 2; void Unrelated;",
    );
    assert!(
        fixture.guard(&edited, &overlays).is_ok(),
        "the original overlay membership is deliberately unchanged"
    );
    let reached = fixture.open(&edited, &overlays);
    assert!(
        !reached.graph_reused,
        "a newly reached supplied overlay must use the full cold producer"
    );
    let mirror = reached.mirror.as_ref().unwrap();
    let file = mirror.find_by_original(&path).unwrap();
    assert_eq!(
        mirror.original_content_for_virtual(&file.virtual_path),
        Some(unsaved)
    );
    assert!(file.content.contains("'unsaved'"));
    assert!(!file.content.contains("'disk'"));
    fixture.assert_cold_equal(&reached, &edited, &overlays);
}
