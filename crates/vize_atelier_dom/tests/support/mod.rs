//! Shared S2 vs shipped DOM-lane differential harness.

#![allow(
    dead_code,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::{DomCompilerOptions, compile_template, compile_template_with_options};
use vize_carton::Allocator;
use vize_carton::config::VueVersion;
use vize_s1_to_s2::{
    DOM_LANE_FLAG, EmitError, LegacyCaps, UnsupportedReason, emit_dom_source,
    emit_dom_source_with_caps,
};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ExpectedRefusal {
    Diagnostics,
    Unsupported(UnsupportedReason),
}

pub fn shipped(src: &str) -> String {
    shipped_with_dialect(src, VueVersion::V3)
}

pub fn shipped_with_dialect(src: &str, dialect: VueVersion) -> String {
    shipped_with_dialect_and_prefix(src, dialect, false)
}

pub fn shipped_prefixed_with_dialect(src: &str, dialect: VueVersion) -> String {
    shipped_with_dialect_and_prefix(src, dialect, true)
}

fn shipped_with_dialect_and_prefix(
    src: &str,
    dialect: VueVersion,
    prefix_identifiers: bool,
) -> String {
    let allocator = Allocator::new();
    let mut options = DomCompilerOptions::default();
    options.dialect = dialect;
    options.prefix_identifiers = prefix_identifiers;
    let (_, errors, result) = if dialect == VueVersion::V3 && !prefix_identifiers {
        compile_template(&allocator, src)
    } else {
        compile_template_with_options(&allocator, src, options)
    };
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

pub fn assert_s2_matches_shipped(battery: &[(&str, &str)]) {
    let mut compared = 0u64;
    let mut skipped_legacy_flag = 0u64;
    if std::env::var(DOM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        skipped_legacy_flag += 1;
    } else {
        let allocator = Allocator::new();
        for (name, src) in battery {
            let old = shipped(src);
            let new = emit_dom_source(&allocator, src)
                .unwrap_or_else(|error| panic!("{name}: S2 emit refused: {error:?}"))
                .assembled();
            assert_eq!(
                old.as_str(),
                new.as_str(),
                "{name}: S2 DOM emit diverged from the shipped lane"
            );
            compared += 1;
        }
    }
    assert_eq!(
        (compared, skipped_legacy_flag),
        (battery.len() as u64, 0),
        "a cfg or {DOM_LANE_FLAG}=legacy regression disarmed the dual-run"
    );
}

pub fn assert_s2_matches_shipped_with_dialect(battery: &[(&str, &str)], dialect: VueVersion) {
    assert_s2_matches_shipped_with_dialect_inner(battery, dialect, false)
}

pub fn assert_s2_matches_prefixed_shipped_literals_with_dialect(
    battery: &[(&str, &str)],
    dialect: VueVersion,
) {
    assert_s2_matches_shipped_with_dialect_inner(battery, dialect, true)
}

fn assert_s2_matches_shipped_with_dialect_inner(
    battery: &[(&str, &str)],
    dialect: VueVersion,
    prefix_identifiers: bool,
) {
    let mut compared = 0u64;
    let mut skipped_legacy_flag = 0u64;
    if std::env::var(DOM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        skipped_legacy_flag += 1;
    } else {
        let allocator = Allocator::new();
        let caps = LegacyCaps::for_version(dialect);
        for (name, src) in battery {
            let old = shipped_with_dialect_and_prefix(src, dialect, prefix_identifiers);
            let new = emit_dom_source_with_caps(&allocator, src, caps)
                .unwrap_or_else(|error| panic!("{name}: S2 emit refused: {error:?}"))
                .assembled();
            assert_eq!(
                old.as_str(),
                new.as_str(),
                "{name}: S2 DOM emit diverged from the shipped lane"
            );
            compared += 1;
        }
    }
    assert_eq!(
        (compared, skipped_legacy_flag),
        (battery.len() as u64, 0),
        "a cfg or {DOM_LANE_FLAG}=legacy regression disarmed the dual-run"
    );
}

#[allow(dead_code)]
pub fn assert_s2_refuses(battery: &[(&str, &str, ExpectedRefusal)]) {
    let allocator = Allocator::new();
    for (name, src, expected) in battery {
        let error = emit_dom_source(&allocator, src)
            .map(|emit| emit.assembled())
            .expect_err(name);
        match expected {
            ExpectedRefusal::Diagnostics => assert_eq!(
                error,
                EmitError::Diagnostics,
                "{name}: S2 DOM refused with the wrong reason"
            ),
            ExpectedRefusal::Unsupported(reason) => assert_eq!(
                error.reason(),
                Some(*reason),
                "{name}: S2 DOM refused with the wrong reason: {error:?}"
            ),
        }
    }
}
