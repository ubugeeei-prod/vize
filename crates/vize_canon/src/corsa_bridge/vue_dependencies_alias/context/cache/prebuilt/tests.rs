use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::super::{ContextFingerprint, SessionCache};
use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

fn fingerprint(path: &Path, content: &str, context: &AliasContext) -> ContextFingerprint {
    let mut fingerprint = ContextFingerprint::capture(
        path,
        content,
        &FxHashMap::default(),
        Default::default(),
        &Default::default(),
        None,
        None,
    );
    fingerprint.stamp(context);
    fingerprint
}

fn host(root: &Path) -> (PathBuf, &'static str) {
    let path = root.join("Host.vue");
    std::fs::write(&path, "<template />").unwrap();
    (path, "<template />")
}

#[test]
fn another_revision_of_the_host_leaves_the_prepared_one_in_place() {
    let root = tempfile::tempdir().unwrap();
    let (path, content) = host(root.path());
    let overlays = FxHashMap::default();
    let mut cache = SessionCache::default();
    let context = AliasContext::for_host(&path, content, &overlays);
    let prepared = fingerprint(&path, content, &context);
    cache.store_prebuilt(cache.generation(), path.clone(), prepared.clone(), context);

    // A hover on the same host without the workspace sources is a different
    // revision: it builds inline and must not consume the prepared projection.
    let unrelated = AliasContext::for_host(&path, "<template>edited</template>", &overlays);
    let edited = fingerprint(&path, "<template>edited</template>", &unrelated);
    assert!(cache.take_prebuilt(&path, &edited, &overlays).is_none());
    assert!(cache.has_prebuilt(&path, &prepared, &overlays));
    assert!(cache.take_prebuilt(&path, &prepared, &overlays).is_some());
    assert!(!cache.has_prebuilt(&path, &prepared, &overlays));
}

#[test]
fn a_disk_change_after_preparation_forces_an_inline_build() {
    let root = tempfile::tempdir().unwrap();
    let (path, content) = host(root.path());
    let config = root.path().join("tsconfig.json");
    std::fs::write(&config, "{}").unwrap();
    let overlays = FxHashMap::default();
    let mut cache = SessionCache::default();
    let context = AliasContext::for_host(&path, content, &overlays);
    let prepared = fingerprint(&path, content, &context);
    cache.store_prebuilt(cache.generation(), path.clone(), prepared.clone(), context);
    std::fs::write(&config, "{\"compilerOptions\":{\"strict\":true}}").unwrap();
    assert!(cache.take_prebuilt(&path, &prepared, &overlays).is_none());
}

#[test]
fn a_session_reset_during_preparation_discards_its_result() {
    let root = tempfile::tempdir().unwrap();
    let (path, content) = host(root.path());
    let overlays = FxHashMap::default();
    let mut cache = SessionCache::default();
    let started = cache.generation();
    cache.clear();
    let context = AliasContext::for_host(&path, content, &overlays);
    let prepared = fingerprint(&path, content, &context);
    cache.store_prebuilt(started, path.clone(), prepared.clone(), context);
    assert!(!cache.has_prebuilt(&path, &prepared, &overlays));
}
