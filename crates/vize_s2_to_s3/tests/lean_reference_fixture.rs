//! TS-28 bridge from Rust S2->S3 lowering into the Lean reference fixture set.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::Allocator;
use vize_s2_to_s3::lower;
use vize_s3::folio::S3Folio;
use vize_s3::verify::verify;

const STATIC_DYNAMIC_SOURCE: &str =
    r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#;

#[test]
fn rust_lowered_static_dynamic_fixture_matches_impeto_reference_input() {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, STATIC_DYNAMIC_SOURCE);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let lowered = lower(&allocator, &s2.root);
    assert_eq!(verify(&lowered.program), []);

    let actual = S3Folio::of(&lowered.program).print_to_string(FolioMode::Full);
    let expected =
        include_str!("../../../formal/impeto/fixtures/rust-lowered-static-dynamic.s3.folio");
    assert_eq!(
        actual.trim_end_matches('\n'),
        expected.trim_end_matches('\n')
    );
}
