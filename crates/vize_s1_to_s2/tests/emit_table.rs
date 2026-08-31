//! Focused DOM parity pins for implicit HTML table tree construction.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::{assert_transformed_sound, with_transformed};
use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom;

fn shipped(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&allocator, source);
    let blocking: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    assert!(blocking.is_empty(), "{source:?}: {blocking:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn emitted(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn assert_shipped_parity(source: &str) {
    assert_transformed_sound(source, "implicit table normalization");
    assert_eq!(emitted(source), shipped(source), "{source}");
}

#[test]
fn direct_table_rows_gain_implicit_tbody() {
    assert_shipped_parity(
        r#"<table><tr><th>{{ title }}</th><td class="value">x</td></tr><tr><td>y</td></tr></table>"#,
    );
}

#[test]
fn row_group_cells_gain_implicit_tr() {
    assert_shipped_parity(
        r#"<table><thead><th>Token</th><th>Usage</th></thead><tbody><td>A</td><td>B</td></tbody></table>"#,
    );
}

#[test]
fn direct_table_cells_gain_implicit_tbody_and_tr() {
    assert_shipped_parity(r#"<table><th>Token</th><td>{{ value }}</td></table>"#);
}

#[test]
fn structural_rows_stay_inside_the_implicit_tbody() {
    assert_shipped_parity(
        r#"<table><tr v-for="item in rows" :key="item.id"><td>{{ item.label }}</td></tr><tr v-if="footer"><td>done</td></tr><tr v-else><td>empty</td></tr></table>"#,
    );
}
