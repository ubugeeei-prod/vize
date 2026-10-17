//! Real normal/prebuild entry points release syntax facts before persistent stores.

use super::*;

#[test]
fn normal_preparation_drops_facts_before_materialization_and_session_storage() {
    let fixture = Fixture::new();
    let overlays = FxHashMap::default();
    let mut uncached = ModuleSpecifierFactsCache::uncached();
    let reference = fixture.build(&overlays, &[], &mut uncached);
    let mut fingerprint = fixture.fingerprint(&overlays, &[]);
    fingerprint.stamp_with_import_facts(&reference, &mut uncached);

    let observation = ModuleSpecifierFactsCache::observe_lifetimes();
    let actual = AliasContext::for_hosts_cached(
        &fixture.host,
        HOST,
        &overlays,
        &[],
        Default::default(),
        fixture.environment(),
    )
    .unwrap();
    assert_context_equal(&actual, &reference);
    assert_eq!(
        fixture.session.cache().cached_input_stamps(&fixture.host),
        Some(fingerprint.input_stamps())
    );
    let work = observation.stats();
    assert_eq!(work.created, 1);
    assert_eq!(work.dropped, 1);
    assert_eq!(work.live, 0);
    assert_eq!(work.checkpoints, 1);
    assert!(
        fixture
            .session
            .cache()
            .get(&fixture.host, &fingerprint)
            .is_some()
    );
}

#[test]
fn workspace_prebuild_stores_exact_stamps_and_adopts_without_retaining_facts() {
    let fixture = Fixture::new();
    let requested = fixture.requested(true);
    let overlays = FxHashMap::default();
    let mut uncached = ModuleSpecifierFactsCache::uncached();
    let reference = fixture.build(&overlays, &requested, &mut uncached);
    let mut fingerprint = fixture.fingerprint(&overlays, &requested);
    fingerprint.stamp_with_import_facts(&reference, &mut uncached);

    let observation = ModuleSpecifierFactsCache::observe_lifetimes();
    AliasContext::prebuild_for_hosts(
        &fixture.host,
        HOST,
        &overlays,
        &requested,
        Default::default(),
        fixture.environment(),
    )
    .unwrap();
    let after_prebuild = observation.stats();
    assert_eq!(after_prebuild.created, 1);
    assert_eq!(after_prebuild.dropped, 1);
    assert_eq!(after_prebuild.live, 0);
    assert_eq!(after_prebuild.checkpoints, 1);
    let (prepared, actual_fingerprint) = fixture
        .session
        .cache()
        .take_prebuilt(&fixture.host, &fingerprint, &overlays)
        .expect("workspace result remains stored after its syntax facts dropped");
    assert_context_equal(&prepared, &reference);
    assert_eq!(
        actual_fingerprint.input_stamps(),
        fingerprint.input_stamps()
    );
    assert!(
        prepared
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&fixture.workspace)
            .is_some()
    );
    {
        let mut cache = fixture.session.cache();
        let generation = cache.generation();
        cache.store_prebuilt(
            generation,
            fixture.host.clone(),
            actual_fingerprint,
            prepared,
        );
    }
    let adopted = AliasContext::for_hosts_cached(
        &fixture.host,
        HOST,
        &overlays,
        &requested,
        Default::default(),
        fixture.environment(),
    )
    .unwrap();
    assert_context_equal(&adopted, &reference);
    assert_eq!(
        fixture.session.cache().cached_input_stamps(&fixture.host),
        Some(fingerprint.input_stamps())
    );
    let after_adoption = observation.stats();
    assert_eq!(after_adoption.created, after_prebuild.created);
    assert_eq!(after_adoption.dropped, after_prebuild.dropped);
    assert_eq!(after_adoption.live, 0);
    assert_eq!(after_adoption.checkpoints, after_prebuild.checkpoints + 1);
    assert!(
        fixture
            .session
            .cache()
            .get(&fixture.host, &fingerprint)
            .is_some()
    );
}

#[test]
fn failed_normal_and_workspace_builds_drop_facts_without_storing_contexts() {
    for workspace in [false, true] {
        let fixture = Fixture::new();
        let broken = fixture.root.join("src/Unsupported.txt");
        let source = "unsupported authored input";
        write(&broken, source);
        let overlays = FxHashMap::default();
        let requested = fixture.requested(workspace);
        let fingerprint = AliasContext::request_fingerprint(
            &broken,
            source,
            &overlays,
            &requested,
            Default::default(),
            fixture.environment(),
        );
        let observation = ModuleSpecifierFactsCache::observe_lifetimes();
        let failed = if workspace {
            AliasContext::prebuild_for_hosts(
                &broken,
                source,
                &overlays,
                &requested,
                Default::default(),
                fixture.environment(),
            )
            .is_err()
        } else {
            AliasContext::for_hosts_cached(
                &broken,
                source,
                &overlays,
                &requested,
                Default::default(),
                fixture.environment(),
            )
            .is_err()
        };
        assert!(
            failed,
            "unsupported extension fails the actual source builder"
        );
        let work = observation.stats();
        assert_eq!(work.created, 1);
        assert_eq!(work.dropped, 1);
        assert_eq!(work.live, 0);
        assert_eq!(
            work.checkpoints, 0,
            "failure leaves before the successful-store boundary"
        );
        let mut cache = fixture.session.cache();
        assert!(cache.get(&broken, &fingerprint).is_none());
        assert!(!cache.has_prebuilt(&broken, &fingerprint, &overlays));
    }
}
