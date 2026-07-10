//! Source Atlas contract tests.

use super::*;
use vize_carton::config::VueVersion;

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
fn plates_classify_into_their_atlas_family() {
    use PlateFamily::*;
    use SourceAtlasPlate::*;

    let pairs = [
        (Sfc, Source),
        (Template, Source),
        (Script, Source),
        (Style, Source),
        (Relief, Syntax),
        (Croquis, Semantic),
        (VirtualTs, Projection),
        (Rendu, Render),
        (AtelierOutput, Finish),
        (SourceMap, Finish),
    ];

    for (plate, family) in pairs {
        assert_eq!(plate.family(), family, "{plate:?} should map to {family:?}");
    }
}

#[test]
fn every_known_plate_has_a_family_and_targets_are_target_family() {
    // Each known plate resolves to one of the seven families.
    for plate in SourceAtlasPlate::KNOWN {
        assert!(PlateFamily::KNOWN.contains(&plate.family()));
    }
    // Every target lane belongs to the Target family.
    for target in SourceAtlasTarget::KNOWN {
        assert_eq!(target.family(), PlateFamily::Target);
    }
}

#[test]
fn plate_family_counters_are_stable() {
    let counters: std::vec::Vec<_> = PlateFamily::KNOWN
        .into_iter()
        .map(PlateFamily::profile_counter)
        .collect();

    assert_eq!(
        counters,
        [
            "atelier.profile.family.source",
            "atelier.profile.family.syntax",
            "atelier.profile.family.semantic",
            "atelier.profile.family.projection",
            "atelier.profile.family.render",
            "atelier.profile.family.target",
            "atelier.profile.family.finish",
        ]
    );
}

#[test]
fn legacy_coordinates_report_a_compatibility_fallback() {
    // Vue 3 and the Vapor capability layer are not legacy.
    assert!(!SourceAtlasCoordinate::from(VueVersion::V3).is_legacy());
    assert!(!SourceAtlasCoordinate::Vapor.is_legacy());
    assert_eq!(
        SourceAtlasCoordinate::from(VueVersion::V3).compatibility_fallback(),
        None
    );
    assert_eq!(SourceAtlasCoordinate::Vapor.compatibility_fallback(), None);

    // Every pre-Vue-3 line is legacy and reports the same fallback fact.
    for version in [
        VueVersion::V2_7,
        VueVersion::V2,
        VueVersion::V1,
        VueVersion::V0_11,
        VueVersion::V0_10,
    ] {
        let coordinate = SourceAtlasCoordinate::from(version);
        assert!(coordinate.is_legacy(), "{version:?} should be legacy");
        assert_eq!(
            coordinate.compatibility_fallback(),
            Some(SourceAtlasFallback::LegacySyntaxCompatibility),
            "{version:?} should report a compatibility fallback"
        );
    }
}

#[test]
fn registry_records_optional_compatibility_fallback() {
    let legacy = SourceAtlasRegistry::compiler()
        .with_coordinate(SourceAtlasCoordinate::from(VueVersion::V2))
        .with_optional_fallback(
            SourceAtlasCoordinate::from(VueVersion::V2).compatibility_fallback(),
        );
    assert!(
        legacy
            .fallbacks
            .contains(SourceAtlasFallback::LegacySyntaxCompatibility)
    );

    let modern = SourceAtlasRegistry::compiler()
        .with_coordinate(SourceAtlasCoordinate::from(VueVersion::V3))
        .with_optional_fallback(
            SourceAtlasCoordinate::from(VueVersion::V3).compatibility_fallback(),
        );
    assert!(modern.fallbacks.is_empty());
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

#[test]
fn fallback_sets_deduplicate_and_iterate_in_known_order() {
    let set = SourceAtlasFallbackSet::empty()
        .with(SourceAtlasFallback::CacheBypass)
        .with(SourceAtlasFallback::SourceMapCompositionSkipped)
        .with(SourceAtlasFallback::CacheBypass);

    assert!(set.contains(SourceAtlasFallback::SourceMapCompositionSkipped));
    assert!(set.contains(SourceAtlasFallback::CacheBypass));
    assert!(!set.contains(SourceAtlasFallback::VaporSsr));
    assert_eq!(
        set.iter().collect::<std::vec::Vec<_>>(),
        [
            SourceAtlasFallback::SourceMapCompositionSkipped,
            SourceAtlasFallback::CacheBypass,
        ]
    );
}

#[test]
fn registry_carries_lane_requests_and_fallbacks() {
    let registry = SourceAtlasRegistry::compiler()
        .with_source(SourceAtlasSource::Sfc)
        .with_plate(SourceAtlasPlate::Rendu)
        .with_target(SourceAtlasTarget::Ssr)
        .with_coordinate(SourceAtlasCoordinate::from(VueVersion::V3))
        .with_fallback(SourceAtlasFallback::SourceMapCompositionSkipped);

    assert_eq!(
        registry.lane.profile_counter(),
        "atelier.profile.lane.compiler"
    );
    assert!(registry.route.sources.contains(SourceAtlasSource::Sfc));
    assert!(registry.route.plates.contains(SourceAtlasPlate::Rendu));
    assert!(registry.route.targets.contains(SourceAtlasTarget::Ssr));
    assert!(
        registry
            .fallbacks
            .contains(SourceAtlasFallback::SourceMapCompositionSkipped)
    );

    let mut requests = std::vec::Vec::new();
    registry.visit_requests(|request| requests.push(request.profile_counter()));
    assert_eq!(
        requests,
        [
            "atelier.profile.input.sfc",
            "atelier.profile.plate.rendu",
            "atelier.profile.target.ssr",
            "atelier.profile.dialect.vue3",
        ]
    );
}

#[test]
fn registry_route_builder_keeps_existing_requests() {
    let registry = SourceAtlasRegistry::typecheck()
        .with_source(SourceAtlasSource::Sfc)
        .with_target(SourceAtlasTarget::VirtualTs)
        .with_plate(SourceAtlasPlate::VirtualTs)
        .with_source(SourceAtlasSource::Ts)
        .with_plate(SourceAtlasPlate::Croquis);

    assert_eq!(
        registry.lane.profile_counter(),
        "atelier.profile.lane.typecheck"
    );
    assert!(registry.route.sources.contains(SourceAtlasSource::Sfc));
    assert!(registry.route.sources.contains(SourceAtlasSource::Ts));
    assert!(
        registry
            .route
            .targets
            .contains(SourceAtlasTarget::VirtualTs)
    );
    assert!(registry.route.plates.contains(SourceAtlasPlate::VirtualTs));
    assert!(registry.route.plates.contains(SourceAtlasPlate::Croquis));
}

#[test]
fn source_map_registry_records_deferred_composition_as_lane_fallback() {
    let registry = SourceAtlasRegistry::source_map()
        .with_source(SourceAtlasSource::VueTemplate)
        .with_plate(SourceAtlasPlate::SourceMap)
        .with_target(SourceAtlasTarget::SourceMap)
        .with_fallback(SourceAtlasFallback::SourceMapCompositionSkipped);

    assert_eq!(
        registry.lane.profile_counter(),
        "atelier.profile.lane.source_map"
    );
    assert!(
        registry
            .route
            .sources
            .contains(SourceAtlasSource::VueTemplate)
    );
    assert!(registry.route.plates.contains(SourceAtlasPlate::SourceMap));
    assert!(
        registry
            .route
            .targets
            .contains(SourceAtlasTarget::SourceMap)
    );
    assert!(
        registry
            .fallbacks
            .contains(SourceAtlasFallback::SourceMapCompositionSkipped)
    );
}
