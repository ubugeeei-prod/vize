//! Focused DOM parity pins for component table structure.

#![expect(clippy::panic, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use crate::support;

use support::{assert_transformed_sound, with_transformed};
use vize_l0::Allocator;
use vize_l1_to_l2::lower::{LegacyCaps, LowerPolicy, lower_with_caps_and_comment_policy};
use vize_l2::op::Op;
use vize_l4::targets::dom::artifact::emit_dom;

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
    assert_transformed_sound(source, "component table normalization");
    let emitted = emitted(source);
    assert_eq!(emitted, shipped(source), "{source}");
    // Vue's compiler emits the authored rows; no owner is implied.
    assert_eq!(
        emitted.matches("\"tbody\"").count(),
        source.matches("<tbody").count()
    );
    assert_eq!(
        emitted.matches("\"tr\"").count(),
        source.matches("<tr").count()
    );
}

/// Element tags of `ops`, depth first, as written in the L2 tree.
fn tags<'a>(ops: &[Op<'a>], out: &mut Vec<&'a str>) {
    for op in ops {
        if let Op::Element(element) = op {
            out.push(element.tag);
            tags(&element.children.ops, out);
        }
    }
}

fn implied_tags(source: &str) -> Vec<String> {
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::markup::parse(&allocator, source);
    let lowered = lower_with_caps_and_comment_policy(
        &allocator,
        &tree,
        &errors,
        LegacyCaps::VUE3,
        LowerPolicy::compile(false).with_implied_table_sections(),
        &[],
        None,
    );
    let mut out = Vec::new();
    tags(&lowered.artifact.root.ops, &mut out);
    out.into_iter().map(str::to_owned).collect()
}

#[test]
fn direct_table_rows_stay_authored() {
    assert_shipped_parity(
        r#"<table><tr><th>{{ title }}</th><td class="value">x</td></tr><tr><td>y</td></tr></table>"#,
    );
}

#[test]
fn row_group_cells_remain_direct_authored_children() {
    assert_shipped_parity(
        r#"<table><thead><th>Token</th><th>Usage</th></thead><tbody><td>A</td><td>B</td></tbody></table>"#,
    );
}

#[test]
fn direct_table_cells_stay_authored() {
    assert_shipped_parity(r#"<table><th>Token</th><td>{{ value }}</td></table>"#);
}

#[test]
fn structural_rows_stay_direct_table_children() {
    assert_shipped_parity(
        r#"<table><tr v-for="item in rows" :key="item.id"><td>{{ item.label }}</td></tr><tr v-if="footer"><td>done</td></tr><tr v-else><td>empty</td></tr></table>"#,
    );
}

#[test]
fn implied_table_sections_policy_keeps_the_html_owners() {
    assert_eq!(
        implied_tags("<table><tr><td>a</td></tr></table>"),
        ["table", "tbody", "tr", "td"]
    );
    assert_eq!(
        implied_tags("<table><th>a</th><td>b</td></table>"),
        ["table", "tbody", "tr", "th", "td"]
    );
    assert_eq!(
        implied_tags("<table><thead><th>a</th></thead></table>"),
        ["table", "thead", "th"]
    );
}
