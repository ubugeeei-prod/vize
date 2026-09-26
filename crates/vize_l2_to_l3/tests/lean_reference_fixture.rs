//! TS-28 bridge from Rust L2->L3 lowering into the Lean reference fixture set.

use vize_davinci::folio::{Folio, FolioMode};
use vize_l0::Allocator;
use vize_l2_to_l3::lower;
use vize_l3::folio::L3Folio;
use vize_l3::trace::{TraceBackend, backend_trace_text, reference_trace_text};
use vize_l3::values_folio::L3ValuesFolio;
use vize_l3::verify::verify;

mod lean_reference_fixture {
    mod loops;
    mod matrix;
    mod models;
    mod schedule;
    mod slots;
}

const STATIC_DYNAMIC_SOURCE: &str =
    r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#;
const CONTROL_SLOTS_SOURCE: &str =
    include_str!("../../../tests/formal/impeto/fixtures/rust-lowered-control-slots.template.txt");

#[test]
fn rust_lowered_values_match_stateful_reference_input() {
    for (source, expected) in [
        (
            STATIC_DYNAMIC_SOURCE,
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-static-dynamic.values.folio"
            ),
        ),
        (
            CONTROL_SLOTS_SOURCE,
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-control-slots.values.folio"
            ),
        ),
    ] {
        let allocator = Allocator::default();
        let (tree, errors) = vize_l1::parse(&allocator, source.trim_end());
        assert!(errors.is_empty(), "{errors:?}");
        let s2 = vize_l1_to_l2::lower(&allocator, &tree, &errors);
        let lowered = lower(&allocator, &s2.root);
        assert_eq!(verify(&lowered.program), []);
        let actual = L3ValuesFolio::of(&lowered.program).print_to_string(FolioMode::Full);
        assert_eq!(
            actual.trim_end_matches('\n'),
            expected.trim_end_matches('\n')
        );
    }
}

#[test]
fn rust_lowered_fixtures_match_impeto_reference_inputs() {
    let cases = [
        (
            STATIC_DYNAMIC_SOURCE,
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-static-dynamic.s3.folio"
            ),
            include_str!("../../../tests/formal/impeto/fixtures/rust-lowered-static-dynamic.trace"),
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-static-dynamic.vdom.trace"
            ),
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-static-dynamic.vapor.trace"
            ),
        ),
        (
            CONTROL_SLOTS_SOURCE,
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-control-slots.s3.folio"
            ),
            include_str!("../../../tests/formal/impeto/fixtures/rust-lowered-control-slots.trace"),
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-control-slots.vdom.trace"
            ),
            include_str!(
                "../../../tests/formal/impeto/fixtures/rust-lowered-control-slots.vapor.trace"
            ),
        ),
    ];

    for (source, expected_folio, expected_trace, expected_vdom, expected_vapor) in cases {
        assert_rust_lowered_fixture(
            source,
            expected_folio,
            expected_trace,
            expected_vdom,
            expected_vapor,
        );
    }
}

fn assert_rust_lowered_fixture(
    source: &str,
    expected_folio: &str,
    expected_trace: &str,
    expected_vdom: &str,
    expected_vapor: &str,
) {
    let allocator = Allocator::default();
    let (tree, errors) = vize_l1::parse(&allocator, source.trim_end());
    assert!(errors.is_empty(), "{errors:?}");
    let s2 = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    let lowered = lower(&allocator, &s2.root);
    assert_eq!(verify(&lowered.program), []);

    let actual = L3Folio::of(&lowered.program).print_to_string(FolioMode::Full);
    assert_eq!(
        actual.trim_end_matches('\n'),
        expected_folio.trim_end_matches('\n')
    );
    assert_eq!(
        reference_trace_text(&lowered.program).trim_end_matches('\n'),
        expected_trace.trim_end_matches('\n')
    );
    assert_eq!(
        backend_trace_text(TraceBackend::Vdom, &lowered.program).trim_end_matches('\n'),
        expected_vdom.trim_end_matches('\n')
    );
    assert_eq!(
        backend_trace_text(TraceBackend::Vapor, &lowered.program).trim_end_matches('\n'),
        expected_vapor.trim_end_matches('\n')
    );
}
