use std::sync::{Arc, Mutex};

use super::{ContextFingerprint, ProjectMember, SessionCache, recover_lock};
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
            path.clone(),
            ProjectMember {
                source_paths: Vec::new(),
                expected_files: FxHashSet::default(),
                package_links: FxHashMap::default(),
                query_paths: Vec::new(),
                stamps: Vec::new(),
                overlay_identity: fingerprint.overlay_identity(),
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
            .contains_key(std::path::Path::new("/workspace/0/App.vue"))
    );
    assert!(
        cache
            .slots
            .contains_key(std::path::Path::new("/workspace/8/App.vue"))
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
            path.clone(),
            ProjectMember {
                source_paths: Vec::new(),
                expected_files: FxHashSet::from_iter([
                    virtual_root.join(cstr!("Host{index}.vue.ts"))
                ]),
                package_links: FxHashMap::default(),
                query_paths: Vec::new(),
                stamps: Vec::new(),
                overlay_identity: fingerprint.overlay_identity(),
                requires_arbitrary_extensions: false,
            },
        );
        cache.insert(path, fingerprint, context);
    }
    assert_eq!(cache.slots.len(), 8);
    assert_eq!(cache.project_members[&virtual_root].len(), 12);
    let union = cache.project_union_snapshot(
        &virtual_root,
        &root.path().join("Other.vue"),
        ContextFingerprint::capture(
            &root.path().join("Other.vue"),
            "",
            &overlays,
            Default::default(),
            &Default::default(),
            None,
            None,
        )
        .overlay_identity(),
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

#[test]
fn project_union_drops_members_with_stale_input_stamps() {
    let root = tempfile::tempdir().unwrap();
    let virtual_root = root.path().join("mirror");
    let source = root.path().join("App.vue");
    let manifest = root.path().join("package.json");
    let expected = virtual_root.join("src/App.vue.ts");
    let link = virtual_root.join("src/node_modules");
    let target = root.path().join("store/node_modules");
    std::fs::write(&manifest, "{\"version\":1}").unwrap();
    let mut cache = SessionCache::default();
    cache.record_project_member(
        virtual_root.clone(),
        source.clone(),
        ProjectMember {
            source_paths: Vec::new(),
            expected_files: FxHashSet::from_iter([expected]),
            package_links: [(link.clone(), target.clone())].into_iter().collect(),
            query_paths: Vec::new(),
            stamps: vec![crate::package_route::stamp::InputStamp::capture(
                manifest.clone(),
            )],
            overlay_identity: 7,
            requires_arbitrary_extensions: true,
        },
    );

    let current = cache.project_union_snapshot(&virtual_root, &source, 7);
    assert!(current.files.is_empty());
    assert!(!current.requires_arbitrary_extensions);
    let union = cache.project_union_snapshot(&virtual_root, &root.path().join("Other.vue"), 7);
    assert_eq!(union.package_links.get(&link), Some(&target));
    assert!(union.requires_arbitrary_extensions);

    std::fs::write(&manifest, "{\"version\":2}").unwrap();
    let union = cache.project_union_snapshot(&virtual_root, &root.path().join("Other.vue"), 7);
    assert!(union.files.is_empty());
    assert!(union.package_links.is_empty());
    assert!(union.query_paths.is_empty());
    assert!(!union.requires_arbitrary_extensions);
}

#[test]
fn project_union_drops_members_from_a_closed_or_changed_overlay_epoch() {
    let root = tempfile::tempdir().unwrap();
    let virtual_root = root.path().join("mirror");
    let source = root.path().join("Host.vue");
    let expected = virtual_root.join("src/UnsavedDependency.vue.ts");
    let mut cache = SessionCache::default();
    cache.record_project_member(
        virtual_root.clone(),
        source,
        ProjectMember {
            source_paths: Vec::new(),
            expected_files: FxHashSet::from_iter([expected.clone()]),
            package_links: FxHashMap::default(),
            query_paths: vec![expected.clone()],
            stamps: Vec::new(),
            overlay_identity: 41,
            requires_arbitrary_extensions: true,
        },
    );

    let union = cache.project_union_snapshot(&virtual_root, &root.path().join("Other.vue"), 41);
    assert!(union.files.contains(&expected));
    assert!(union.requires_arbitrary_extensions);
    assert_eq!(union.query_paths, vec![expected]);

    let union = cache.project_union_snapshot(&virtual_root, &root.path().join("Other.vue"), 42);
    assert!(union.files.is_empty());
    assert!(union.package_links.is_empty());
    assert!(union.query_paths.is_empty());
    assert!(!union.requires_arbitrary_extensions);
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
