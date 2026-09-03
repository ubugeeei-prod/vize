//! P2-11 witness: source-map requests retain their compatibility contract
//! while normal DOM templates use the S2 backend.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};

#[test]
fn source_maps_keep_the_documented_compatibility_contract() {
    let allocator = vize_s0::Allocator::new();
    let (_, errors, result) = compile_template_with_options(
        &allocator,
        "<div>{{ msg }}</div>",
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
    );

    assert!(errors.is_empty());
    assert!(
        result.map.is_some(),
        "source-map requests must remain additive"
    );
}
