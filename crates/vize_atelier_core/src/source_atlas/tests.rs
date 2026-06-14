use super::*;

#[test]
fn vue_coordinates_reuse_config_version_order() {
    let counters: std::vec::Vec<_> = VueVersion::ALL
        .into_iter()
        .map(|version| SourceAtlasCoordinate::from(version).profile_counter())
        .collect();

    assert_eq!(
        counters,
        [
            "atelier.profile.dialect.vue3",
            "atelier.profile.dialect.vue2_7",
            "atelier.profile.dialect.vue2",
            "atelier.profile.dialect.vue1",
            "atelier.profile.dialect.vue0_11",
            "atelier.profile.dialect.vue0_10",
        ]
    );
}

#[test]
fn vapor_is_a_capability_coordinate_not_a_vue_version() {
    assert_eq!(
        SourceAtlasCoordinate::Vapor.profile_counter(),
        "atelier.profile.capability.vapor"
    );
}

#[test]
fn requests_delegate_to_their_plate_family() {
    assert_eq!(
        SourceAtlasRequest::Source(SourceAtlasSource::Tsx).profile_counter(),
        "atelier.profile.input.tsx"
    );
    assert_eq!(
        SourceAtlasRequest::Plate(SourceAtlasPlate::Rendu).profile_counter(),
        "atelier.profile.plate.rendu"
    );
    assert_eq!(
        SourceAtlasRequest::Target(SourceAtlasTarget::Ssr).profile_counter(),
        "atelier.profile.target.ssr"
    );
    assert_eq!(
        SourceAtlasRequest::Coordinate(SourceAtlasCoordinate::from(VueVersion::V2_7))
            .profile_counter(),
        "atelier.profile.dialect.vue2_7"
    );
}

#[test]
fn plate_sets_deduplicate_and_iterate_in_known_order() {
    let set = SourceAtlasPlateSet::empty()
        .with(SourceAtlasPlate::SourceMap)
        .with(SourceAtlasPlate::Sfc)
        .with(SourceAtlasPlate::SourceMap)
        .with(SourceAtlasPlate::VirtualTs);

    assert!(set.contains(SourceAtlasPlate::Sfc));
    assert!(set.contains(SourceAtlasPlate::SourceMap));
    assert!(!set.contains(SourceAtlasPlate::Rendu));

    let plates: std::vec::Vec<_> = set.iter().collect();
    assert_eq!(
        plates,
        [
            SourceAtlasPlate::Sfc,
            SourceAtlasPlate::VirtualTs,
            SourceAtlasPlate::SourceMap,
        ]
    );
}

#[test]
fn routes_capture_multi_source_and_multi_target_lanes() {
    let route = SourceAtlasRoute::empty()
        .with_source(SourceAtlasSource::Sfc)
        .with_source(SourceAtlasSource::Tsx)
        .with_target(SourceAtlasTarget::Sfc)
        .with_target(SourceAtlasTarget::Vdom)
        .with_target(SourceAtlasTarget::Ssr)
        .with_target(SourceAtlasTarget::Tsx)
        .with_plate(SourceAtlasPlate::Croquis)
        .with_plate(SourceAtlasPlate::Rendu)
        .with_coordinate(SourceAtlasCoordinate::Vapor);

    assert!(route.sources.contains(SourceAtlasSource::Sfc));
    assert!(route.sources.contains(SourceAtlasSource::Tsx));
    assert!(route.targets.contains(SourceAtlasTarget::Ssr));
    assert!(route.targets.contains(SourceAtlasTarget::Vdom));
    assert!(route.targets.contains(SourceAtlasTarget::Tsx));
    assert_eq!(
        route.targets.iter().collect::<std::vec::Vec<_>>(),
        [
            SourceAtlasTarget::Vdom,
            SourceAtlasTarget::Sfc,
            SourceAtlasTarget::Ssr,
            SourceAtlasTarget::Tsx,
        ]
    );
    assert!(route.plates.contains(SourceAtlasPlate::Rendu));
    assert_eq!(route.coordinate, Some(SourceAtlasCoordinate::Vapor));
}

#[test]
fn fallback_reasons_keep_existing_counter_names_stable() {
    assert_eq!(
        SourceAtlasFallback::VaporSsr.profile_counter(),
        "atelier.fallback.vapor_ssr"
    );
    assert_eq!(
        SourceAtlasFallback::SourceMapCompositionSkipped.profile_counter(),
        "atelier.fallback.source_map.composition_skipped"
    );
}
