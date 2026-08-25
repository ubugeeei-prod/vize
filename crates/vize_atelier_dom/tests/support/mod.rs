//! Shared S2 vs shipped DOM-lane differential harness.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, EmitError, UnsupportedReason, emit_dom_source};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ExpectedRefusal {
    Diagnostics,
    Unsupported(UnsupportedReason),
}

pub fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
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
