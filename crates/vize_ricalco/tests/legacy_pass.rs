//! The legacy port's filter pins (P2-9 series 7, `_legacy` only): the
//! split against the mirrored splitter's laws, the asset registration,
//! the bails the live lane takes, and the per-dialect pipeline shape.
//! The template-sugar half lives in `legacy_pass_sugar.rs` (the source
//! budget's split).

mod support;

use vize_davinci::pass::{Fusability, PassKind, Preserved};
use vize_disegno::folio::FolioOp;
use vize_ricalco::LegacyVueLine;
use vize_ricalco::pass::{TRANSFORM, TRANSFORM_LEGACY, TRANSFORM_LEGACY_PASSES, legacy};

use support::{assert_transformed_sound_legacy, with_transformed_legacy};

fn v2<R>(
    source: &str,
    f: impl FnOnce(
        &vize_ricalco::Lowered<'_>,
        &vize_disegno::folio::DisegnoFolio,
        &vize_ricalco::pass::S2Facts,
    ) -> R,
) -> R {
    with_transformed_legacy(source, LegacyVueLine::V2, |lowered, folio, facts, _| {
        f(lowered, folio, facts)
    })
}

/// Owned view of one folio page's flat op keywords, for shape pins.
fn keywords(folio: &vize_disegno::folio::DisegnoFolio) -> Vec<&'static str> {
    fn walk(ops: &[FolioOp], out: &mut Vec<&'static str>) {
        for op in ops {
            match op {
                FolioOp::Element(element) => {
                    out.push("element");
                    walk(&element.children, out);
                }
                FolioOp::Component(component) => {
                    out.push("component");
                    walk(&component.children, out);
                }
                FolioOp::Text(_) => out.push("text"),
                FolioOp::Interpolation(_) => out.push("interpolation"),
                FolioOp::VueFilter(_) => out.push("vue.filter"),
                FolioOp::If(_) => out.push("if"),
                FolioOp::For(_) => out.push("for"),
                FolioOp::Slot(_) => out.push("slot"),
            }
        }
    }
    let mut out = Vec::new();
    walk(&folio.ops, &mut out);
    out
}

#[test]
fn the_legacy_pipeline_holds_the_landed_passes() {
    assert_eq!(TRANSFORM_LEGACY_PASSES.len(), 7);
    assert_eq!(TRANSFORM_LEGACY_PASSES[6], legacy::DESC);
    // The plain pipeline is byte-identical in this feature shape — the
    // zero-cost clause's pipeline half.
    assert_eq!(TRANSFORM.group_count(), 6);
    assert_eq!(TRANSFORM_LEGACY.group_count(), 7);
    assert!(TRANSFORM_LEGACY.is_fully_serialized());
    let group = TRANSFORM_LEGACY.group(6).expect("the seventh group exists");
    assert!(group.is_barrier && group.len == 1 && group.start == 6);
}

#[test]
fn the_legacy_classification_is_pinned() {
    // The review-point classification (see `pass::legacy`'s docs): the
    // preserving-mandatory taxonomy tension, fourth occurrence.
    assert_eq!(legacy::DESC.name, "legacy");
    assert_eq!(legacy::DESC.kind, PassKind::MandatoryLowering);
    assert_eq!(legacy::DESC.fusability, Fusability::Barrier);
    assert_eq!(legacy::DESC.preserved, Preserved::ALL);
}

#[test]
fn a_lone_filter_interpolation_lowers_to_the_dialect_op() {
    v2(
        "<div>{{ message | capitalize }}</div>",
        |lowered, folio, facts| {
            assert_eq!(keywords(folio), ["element", "vue.filter"]);
            assert_eq!(
                facts.legacy.assets,
                vec![vize_carton::String::from("capitalize")]
            );
            assert_eq!(facts.legacy.sites.len(), 1);
            let (_, site) = facts.legacy.sites.sorted_entries().remove(0);
            assert_eq!(site.base.as_str(), "message");
            assert_eq!(site.names.len(), 1);
            assert_eq!(site.names[0].as_str(), "capitalize");
            assert!(
                lowered
                    .provenance
                    .iter()
                    .any(|r| r.rule.as_str() == "lower.vue-filter")
            );
            assert!(
                lowered
                    .provenance
                    .iter()
                    .any(|r| r.rule.as_str() == "pass.legacy.filter")
            );
            assert_eq!(lowered.diagnostics, vec![]);
        },
    );
    assert_transformed_sound_legacy(
        "<div>{{ message | capitalize }}</div>",
        LegacyVueLine::V2,
        "lone filter interpolation",
    );
}

#[test]
fn a_filter_bind_value_is_the_pessimal_escape_with_its_site() {
    v2(r#"<a :id="raw | formatId"></a>"#, |_, folio, facts| {
        // The bind stays `ui.bind`; the chain rides the value opaquely.
        let FolioOp::Element(element) = &folio.ops[0] else {
            panic!("expected the element");
        };
        assert_eq!(element.bindings.len(), 1);
        assert_eq!(
            facts.legacy.assets,
            vec![vize_carton::String::from("formatId")]
        );
        assert_eq!(facts.legacy.sites.len(), 1);
        let (_, site) = facts.legacy.sites.sorted_entries().remove(0);
        assert_eq!(site.base.as_str(), "raw");
    });
}

#[test]
fn assets_register_first_seen_and_deduplicated() {
    // The live `ctx.add_filter` order, mirrored: page order, dedup.
    v2(
        "<div>{{ a | f(b) | g }}</div><p>{{ z | h | f }}</p>",
        |_, _, facts| {
            let names: Vec<&str> = facts.legacy.assets.iter().map(|s| s.as_str()).collect();
            assert_eq!(names, ["f", "g", "h"]);
        },
    );
}

#[test]
fn non_filter_pipes_are_never_split() {
    // Logical OR, nested pipes, quoted pipes: the mirrored splitter's
    // negative space, byte-identical to the shipped `parse_filters`.
    for source in [
        "<div>{{ a || b }}</div>",
        "<div>{{ [a | b] }}</div>",
        "<div>{{ f(a | b) }}</div>",
        "<div>{{ 'x | y' }}</div>",
    ] {
        v2(source, |_, folio, facts| {
            assert_eq!(keywords(folio), ["element", "interpolation"], "{source}");
            assert!(facts.legacy.assets.is_empty(), "{source}");
            assert_eq!(facts.legacy.sites.len(), 0, "{source}");
        });
    }
}

#[test]
fn a_malformed_segment_name_abandons_the_whole_split() {
    // The `rewrite_filters_in_place` bail, mirrored: `(bad)` is not a
    // filter name, so the whole expression admits normally (it parses
    // as JS bitwise-or) and no site exists.
    v2("<div>{{ x | (bad) }}</div>", |_, folio, facts| {
        assert_eq!(keywords(folio), ["element", "interpolation"]);
        assert_eq!(facts.legacy.sites.len(), 0);
        assert!(facts.legacy.assets.is_empty());
    });
}

#[test]
fn a_filter_inside_a_merged_run_stays_the_compound_producer() {
    // The recorded narrowing (`filters_in_compounds`): a mergeable run
    // keeps the Compound representation; the pipe rides the dynamic
    // part verbatim and no filter site exists.
    v2("<em>pre {{ m | f }}</em>", |_, folio, facts| {
        assert_eq!(keywords(folio), ["element", "interpolation"]);
        assert_eq!(facts.legacy.sites.len(), 0);
        assert_eq!(facts.text_facts.len(), 1);
    });
}
