//! P2-9 series 7, the legacy-lane coverage witness: the committed
//! legacy battery dual-runs the shipped legacy transform lane (V2
//! dialect) against the S2 legacy pipeline with **exact-pinned**
//! counters, and the same battery dual-runs under the **default
//! dialect** as the V3-meaning control — proving the legacy feature is
//! inert for Vue 3 sources in both lanes at once (the shipped
//! `legacy_filters.rs` suite's gating claim, now cross-lane).
//!
//! Compiled only under `--features legacy` (the manifest's
//! `required-features`): the S2 side's `_legacy` vocabulary is always
//! present (the dev-dependency pins it), the *legacy-lane* side needs
//! this crate's own feature.

mod s2_support;

use s2_support::legacy::{LEGACY_BATTERY, LegacyCounters, compare_legacy};
use s2_support::{Counters, HoistCounters, SlotCounters, SurfaceCounters, TextCounters, compare};

/// The legacy-dialect run's exact accounting.
fn expected_legacy() -> (Counters, LegacyCounters) {
    let counters = Counters {
        templates_seen: 19,
        compared: 19,
        skipped_legacy_flag: 0,
        skipped_old_parse_errors: 0,
        skipped_s2_errors: 0,
        if_ops: 1,
        branches: 1,
        keys_static: 0,
        keys_dynamic: 0,
        keys_wrapper: 0,
        keys_dynamic_arg: 0,
        keys_compound: 0,
        conditions_compound: 0,
        for_ops: 0,
        for_values: 0,
        for_keys: 0,
        for_indexes: 0,
        for_values_absent: 0,
        for_compound: 0,
        slots: SlotCounters {
            units: 3,
            groups: 3,
            group_params: 3,
            groups_invented: 1,
            ..SlotCounters::default()
        },
        text: TextCounters {
            units: 12,
            parts_static: 5,
            parts_dynamic: 8,
            compound_units: 1,
            ..TextCounters::default()
        },
        surfaces: SurfaceCounters {
            owners: 25,
            attrs: 2,
            binds: 3,
            binds_dynamic: 1,
            ons: 4,
            models: 4,
            ..SurfaceCounters::default()
        },
        // The hoist-decision half stays V3-scoped (`legacy` module
        // docs), so the legacy-dialect run leaves it untouched.
        hoist: HoistCounters::default(),
    };
    let legacy = LegacyCounters {
        filter_sites: 6,
        filter_segments: 8,
        assets_matched: 17,
        assets_narrowed: 2,
        filters_other_positions: 1,
        filters_in_compounds: 1,
        syncs: 4,
        scoped_slots: 2,
        natives: 2,
        keycodes: 1,
    };
    (counters, legacy)
}

/// The V3-meaning control's exact accounting: the same sources, both
/// lanes on the default dialect — filters mean bitwise-or, `.sync` and
/// `slot-scope` stay authored surface, and every projection still
/// agrees.
fn expected_control() -> Counters {
    Counters {
        templates_seen: 19,
        compared: 19,
        skipped_legacy_flag: 0,
        skipped_old_parse_errors: 0,
        skipped_s2_errors: 0,
        if_ops: 1,
        branches: 1,
        keys_static: 0,
        keys_dynamic: 0,
        keys_wrapper: 0,
        keys_dynamic_arg: 0,
        keys_compound: 0,
        conditions_compound: 0,
        for_ops: 0,
        for_values: 0,
        for_keys: 0,
        for_indexes: 0,
        for_values_absent: 0,
        for_compound: 0,
        slots: SlotCounters {
            units: 1,
            groups: 1,
            group_params: 1,
            ..SlotCounters::default()
        },
        text: TextCounters {
            units: 12,
            parts_static: 5,
            parts_dynamic: 8,
            compound_units: 1,
            ..TextCounters::default()
        },
        surfaces: SurfaceCounters {
            owners: 25,
            attrs: 5,
            binds: 7,
            binds_dynamic: 1,
            ons: 4,
            ..SurfaceCounters::default()
        },
        hoist: HoistCounters {
            elements: 24,
            whole: 2,
            props: 1,
            ..HoistCounters::default()
        },
    }
}

#[test]
fn the_legacy_battery_compares_exactly_under_the_v2_dialect() {
    let mut counters = Counters::default();
    let mut legacy = LegacyCounters::default();
    for (name, source) in LEGACY_BATTERY {
        compare_legacy(name, source, &mut counters, &mut legacy);
    }
    let (expected, expected_extra) = expected_legacy();
    assert_eq!(counters, expected);
    assert_eq!(legacy, expected_extra);
}

#[test]
fn the_legacy_battery_keeps_its_v3_meaning_under_the_default_dialect() {
    let mut counters = Counters::default();
    for (name, source) in LEGACY_BATTERY {
        compare(name, source, &mut counters);
    }
    assert_eq!(counters, expected_control());
}

/// The capability-mirror pin: `vize_ricalco`'s `LegacyVueLine` mode is
/// a documented mirror of the armature capability model (the ricalco
/// manifest's dependency-direction note), and this is where the two
/// homes are both visible — pinned field-for-field over the consumed
/// surface, per line, so the copies can only drift loudly.
#[test]
fn the_ricalco_capability_mirror_matches_the_armature_model() {
    use vize_armature::legacy::LegacyVueVersion;
    use vize_ricalco::LegacyVueLine;
    let pairs = [
        (LegacyVueVersion::V0_10, LegacyVueLine::V0_10),
        (LegacyVueVersion::V0_11, LegacyVueLine::V0_11),
        (LegacyVueVersion::V1, LegacyVueLine::V1),
        (LegacyVueVersion::V2, LegacyVueLine::V2),
    ];
    for (version, line) in pairs {
        let caps = version.capabilities();
        let mode = vize_ricalco::lower::legacy_mode_probe(line);
        assert_eq!(
            mode,
            (
                caps.supports_filters,
                caps.scoped_slot_attrs,
                matches!(version, LegacyVueVersion::V2),
            ),
            "mirror drifted for {version:?}"
        );
    }
}
