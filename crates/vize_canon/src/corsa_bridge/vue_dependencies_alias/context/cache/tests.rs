use std::sync::{Arc, Mutex};

use super::{ContextFingerprint, ProjectMember, RevisionKey, SessionCache, recover_lock};

fn host(source_path: std::path::PathBuf) -> RevisionKey {
    RevisionKey {
        source_path,
        workspace: false,
    }
}
use crate::corsa_bridge::vue_dependencies_alias::AliasContext;
use vize_carton::{FxHashMap, FxHashSet, cstr};

#[test]
fn cache_evicts_only_the_least_recently_used_context() {
    let mut cache = SessionCache::default();
    let overlays = FxHashMap::default();
    for index in 0..9 {
        let path = std::path::PathBuf::from(cstr!("/workspace/{index}/App.vue").as_str());
        let virtual_root = std::path::PathBuf::from(cstr!("/mirror/{index}").as_str());
        let context = Arc::new(AliasContext::for_host(&path, "", &overlays));
        let mut fingerprint = ContextFingerprint::capture(
            &path,
            "",
            &overlays,
            Default::default(),
            &Default::default(),
            None,
            None,
        );
        fingerprint.stamp(&context);
        cache.record_project_member(
            virtual_root.clone(),
            host(path.clone()),
            ProjectMember {
                source_paths: Vec::new(),
                refreshed_sources: Vec::new(),
                expected_files: FxHashSet::default(),
                package_links: FxHashMap::default(),
                query_paths: Vec::new(),
                fingerprint: fingerprint.clone(),
                requires_arbitrary_extensions: false,
            },
        );
        cache.set_materialized_snapshot(virtual_root, Default::default());
        cache.insert(path, fingerprint, context);
    }
    assert_eq!(cache.slots.len(), 8);
    assert!(
        !cache
            .slots
            .contains_key(&host("/workspace/0/App.vue".into()))
    );
    assert!(
        cache
            .slots
            .contains_key(&host("/workspace/8/App.vue".into()))
    );
    assert_eq!(cache.project_members.len(), 8);
    assert_eq!(cache.project_snapshots.len(), 8);
    assert!(
        !cache
            .project_members
            .contains_key(std::path::Path::new("/mirror/0"))
    );
}

#[test]
fn context_eviction_preserves_members_of_a_live_native_project() {
    let mut cache = SessionCache::default();
    let overlays = FxHashMap::default();
    let root = tempfile::tempdir().unwrap();
    let virtual_root = root.path().join("mirror");
    for index in 0..12 {
        let path = root.path().join(cstr!("Host{index}.vue"));
        std::fs::write(&path, "<template />").unwrap();
        let context = Arc::new(AliasContext::for_host(&path, "<template />", &overlays));
        let mut fingerprint = ContextFingerprint::capture(
            &path,
            "<template />",
            &overlays,
            Default::default(),
            &Default::default(),
            None,
            None,
        );
        fingerprint.stamp(&context);
        cache.record_project_member(
            virtual_root.clone(),
            host(path.clone()),
            ProjectMember {
                source_paths: Vec::new(),
                refreshed_sources: Vec::new(),
                expected_files: FxHashSet::from_iter([
                    virtual_root.join(cstr!("Host{index}.vue.ts"))
                ]),
                package_links: FxHashMap::default(),
                query_paths: Vec::new(),
                fingerprint: fingerprint.clone(),
                requires_arbitrary_extensions: false,
            },
        );
        cache.insert(path, fingerprint, context);
    }
    assert_eq!(cache.slots.len(), 8);
    assert_eq!(cache.project_members[&virtual_root].len(), 12);
    let union = cache.project_union_snapshot(
        &virtual_root,
        &host(root.path().join("Other.vue")),
        &ContextFingerprint::capture(
            &root.path().join("Other.vue"),
            "",
            &overlays,
            Default::default(),
            &Default::default(),
            None,
            None,
        ),
    );
    assert_eq!(
        union.files.len(),
        12,
        "native project membership is independent of the heavy context cache"
    );
}

#[test]
fn unstamped_request_identity_reuses_a_valid_stamped_context() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("App.vue");
    std::fs::write(&source, "<template />\n").unwrap();
    let overlays = FxHashMap::default();
    let context = Arc::new(AliasContext::for_host(&source, "<template />\n", &overlays));
    let mut cached = ContextFingerprint::capture(
        &source,
        "<template />\n",
        &overlays,
        Default::default(),
        &Default::default(),
        None,
        None,
    );
    cached.stamp(&context);
    let mut cache = SessionCache::default();
    cache.insert(source.clone(), cached, context);

    let request = ContextFingerprint::capture(
        &source,
        "<template />\n",
        &overlays,
        Default::default(),
        &Default::default(),
        None,
        None,
    );
    assert!(cache.get(&source, &request).is_some());
}

#[test]
fn watched_session_trusts_stamps_until_the_event_invalidates_them() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("App.vue");
    let config = root.path().join("tsconfig.json");
    std::fs::write(&source, "<template />\n").unwrap();
    std::fs::write(&config, r#"{"strict":true}"#).unwrap();
    let overlays = FxHashMap::default();
    let context = Arc::new(AliasContext::for_host(&source, "<template />\n", &overlays));
    let mut cached = ContextFingerprint::capture(
        &source,
        "<template />\n",
        &overlays,
        Default::default(),
        &Default::default(),
        None,
        None,
    );
    cached.stamp(&context);
    let request = cached.clone();
    let mut cache = SessionCache::default();
    cache.trust_watched_disk_inputs();
    cache.insert(source.clone(), cached, context);

    std::fs::write(&config, r#"{"strict":false}"#).unwrap();
    assert!(cache.get(&source, &request).is_some());

    cache.invalidate_disk_inputs();
    assert!(cache.get(&source, &request).is_none());
}

fn capture(
    path: &std::path::Path,
    content: &str,
    overlays: &FxHashMap<std::path::PathBuf, &str>,
) -> ContextFingerprint {
    ContextFingerprint::capture(
        path,
        content,
        overlays,
        Default::default(),
        &Default::default(),
        None,
        None,
    )
}

#[test]
fn project_union_drops_members_with_stale_input_stamps() {
    let root = tempfile::tempdir().unwrap();
    let virtual_root = root.path().join("mirror");
    let source = root.path().join("App.vue");
    let manifest = root.path().join("tsconfig.json");
    let expected = virtual_root.join("src/App.vue.ts");
    let link = virtual_root.join("src/node_modules");
    let target = root.path().join("store/node_modules");
    std::fs::write(&source, "<template />").unwrap();
    std::fs::write(&manifest, "{}").unwrap();
    let overlays = FxHashMap::default();
    let context = AliasContext::for_host(&source, "<template />", &overlays);
    let mut fingerprint = capture(&source, "<template />", &overlays);
    fingerprint.stamp(&context);
    let mut cache = SessionCache::default();
    cache.record_project_member(
        virtual_root.clone(),
        host(source.clone()),
        ProjectMember {
            source_paths: Vec::new(),
            refreshed_sources: Vec::new(),
            expected_files: FxHashSet::from_iter([expected]),
            package_links: [(link.clone(), target.clone())].into_iter().collect(),
            query_paths: Vec::new(),
            fingerprint: fingerprint.clone(),
            requires_arbitrary_extensions: true,
        },
    );

    let current = cache.project_union_snapshot(&virtual_root, &host(source.clone()), &fingerprint);
    assert!(current.files.is_empty());
    assert!(!current.requires_arbitrary_extensions);
    let other = host(root.path().join("Other.vue"));
    let union = cache.project_union_snapshot(&virtual_root, &other, &fingerprint);
    assert_eq!(union.package_links.get(&link), Some(&target));
    assert!(union.requires_arbitrary_extensions);

    std::fs::write(&manifest, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
    let union = cache.project_union_snapshot(&virtual_root, &other, &fingerprint);
    assert!(union.files.is_empty());
    assert!(union.package_links.is_empty());
    assert!(union.query_paths.is_empty());
    assert!(!union.requires_arbitrary_extensions);
}

#[test]
fn project_union_drops_members_from_a_closed_or_changed_dependency_overlay() {
    for close in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let virtual_root = root.join("mirror");
        let source = root.join("Host.vue");
        let dependency = root.join("UnsavedDependency.vue");
        let content = "<script setup>import Dependency from './UnsavedDependency.vue';</script><template><Dependency /></template>";
        std::fs::write(&source, content).unwrap();
        std::fs::write(&dependency, "<template />").unwrap();
        let expected = virtual_root.join("src/UnsavedDependency.vue.ts");
        let overlays = FxHashMap::from_iter([(dependency.clone(), "<template>unsaved</template>")]);
        let context = AliasContext::for_host(&source, content, &overlays);
        let mut fingerprint = capture(&source, content, &overlays);
        fingerprint.stamp(&context);
        let mut cache = SessionCache::default();
        cache.record_project_member(
            virtual_root.clone(),
            host(source),
            ProjectMember {
                source_paths: Vec::new(),
                refreshed_sources: Vec::new(),
                expected_files: FxHashSet::from_iter([expected.clone()]),
                package_links: FxHashMap::default(),
                query_paths: vec![expected.clone()],
                fingerprint: fingerprint.clone(),
                requires_arbitrary_extensions: true,
            },
        );
        let other = host(root.join("Other.vue"));
        let union = cache.project_union_snapshot(&virtual_root, &other, &fingerprint);
        assert_eq!(union.files, FxHashSet::from_iter([expected.clone()]));
        assert!(union.requires_arbitrary_extensions);
        assert_eq!(union.query_paths, vec![expected]);
        let overlays = if close {
            FxHashMap::default()
        } else {
            FxHashMap::from_iter([(dependency, "<template>changed</template>")])
        };
        let changed = capture(&other.source_path, "<template />", &overlays);
        let union = cache.project_union_snapshot(&virtual_root, &other, &changed);
        assert!(union.files.is_empty());
        assert!(union.package_links.is_empty());
        assert!(union.query_paths.is_empty());
        assert!(!union.requires_arbitrary_extensions);
    }
}

#[test]
fn poisoned_mutex_is_recovered() {
    let mutex = Arc::new(Mutex::new(0));
    let poisoned = Arc::clone(&mutex);
    let _ = std::thread::spawn(move || {
        let _guard = poisoned.lock().unwrap();
        panic!("poison for test");
    })
    .join();
    *recover_lock(&mutex) = 1;
    assert_eq!(*recover_lock(&mutex), 1);
}
