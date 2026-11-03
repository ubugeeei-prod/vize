//! Complete legacy/L2 modules for the accepted authored branch-key controls.

#![cfg(feature = "legacy-differential")]
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "integration tests compare whole authored modules"
)]

mod davinci_dom_corpus_support;

use davinci_dom_corpus_support::{Lane, Report, compare_sfc_template_lane};
use toml::Value;

const CONTROLS: &[&str] = &[
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/local_member_three.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/global_identifier.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/attribute_bind_kind.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/global_processed_to_raw.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/member_without_references.toml"
    ),
    include_str!("../../../tests/_fixtures/differential/compiler/n8n-if-keys/local_shorthand.toml"),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/binding_identifier_only.toml"
    ),
    include_str!(
        "../../../tests/_fixtures/differential/compiler/n8n-if-keys/expression_local_reference.toml"
    ),
];

#[test]
fn accepted_default_prefixed_and_binding_modules_agree_in_full() {
    let mut report = Report::default();
    for control in CONTROLS {
        let control: Value = toml::from_str(control).expect("official control capture");
        let name = control["name"].as_str().expect("name");
        let source = control["source"].as_str().expect("complete SFC");
        let modes = control["modes"].as_array().expect("two modes");
        for (index, lane) in [(0, Lane::Default), (1, Lane::Prefixed)] {
            if modes[index]["error_codes"]
                .as_array()
                .expect("errors")
                .is_empty()
            {
                compare_sfc_template_lane(name, source, &mut report, lane);
            }
        }
        // Setup metadata rewrites foo to $setup.foo, so even the raw _ctx.foo
        // control cannot collide in this independent third compiler surface.
        compare_sfc_template_lane(name, source, &mut report, Lane::Bindings);
    }
    assert_eq!(
        (report.files, report.templates, report.compared),
        (17, 17, 17)
    );
    assert_eq!(report.old_error_skips, 0);
    assert_eq!(report.s2_refusals, Vec::<std::string::String>::new());
    assert_eq!(report.divergences, Vec::<std::string::String>::new());
    assert_eq!(report.patch_fact_compared, 17);
}
