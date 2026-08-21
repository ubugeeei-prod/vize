//! The legacy lane's filter checks (P2-9 series 7; split from
//! [`super::legacy`] under the source budget): the per-site comparison
//! against the shipped splitter, the filter-armed second legacy run's
//! registration check, and the two narrowing probes — reasoning in the
//! [`super::legacy`] module docs.

#![allow(dead_code)]

use vize_atelier_core::parser::parse_with_options as old_parse_with_options;
use vize_atelier_core::steps::legacy_filters::{filter_name, parse_filters};
use vize_atelier_core::{ParserOptions, TransformOptions, transform};
use vize_carton::config::VueVersion;
use vize_carton::{Allocator, String};
use vize_disegno::expr::OpaqueReason;
use vize_disegno::folio::{DisegnoFolio, FolioBinding, FolioExpr, FolioOp};

use super::legacy_batt::LegacyCounters;
use super::s2_lane::S2Projection;

/// The filter half of one legacy comparison: sites against the shipped
/// splitter, assets against the armed run, then the probes.
pub fn check_filters(
    name: &str,
    source: &str,
    folio: &DisegnoFolio,
    facts: &vize_ricalco::pass::S2Facts,
    s2: &S2Projection,
    legacy: &mut LegacyCounters,
) {
    // The filter-site structural comparison: every S2 site re-split by
    // the shipped splitter itself, byte-equal.
    let mut site_sources: Vec<String> = Vec::new();
    collect_filter_sources(&folio.ops, &mut site_sources);
    let sites = facts.legacy.sites.sorted_entries();
    assert_eq!(
        site_sources.len(),
        sites.len(),
        "{name}: filter-op count diverged from the site table"
    );
    for (source_text, (_, site)) in site_sources.iter().zip(sites.iter()) {
        let shipped = parse_filters(source_text).unwrap_or_else(|| {
            panic!("{name}: the shipped splitter refuses an S2 filter site: {source_text}")
        });
        assert_eq!(
            shipped.base.as_str(),
            site.base.as_str(),
            "{name}: filter base diverged at {source_text}"
        );
        assert_eq!(
            shipped.filters.len(),
            site.names.len(),
            "{name}: segment count diverged at {source_text}"
        );
        for (segment, s2_name) in shipped.filters.iter().zip(site.names.iter()) {
            let shipped_name = filter_name(segment.as_str()).unwrap_or_else(|| {
                panic!("{name}: the shipped lane rejects a segment S2 admitted: {segment}")
            });
            assert_eq!(
                shipped_name,
                s2_name.as_str(),
                "{name}: filter name diverged at {source_text}"
            );
            legacy.filter_segments += 1;
        }
        legacy.filter_sites += 1;
    }

    // The armed second legacy run: the shipped registration against the
    // S2 assets — equality, or the counted subset narrowing.
    let armed_allocator = Allocator::new();
    let options = ParserOptions {
        is_pre_tag: |tag| tag == "pre",
        ..ParserOptions::default()
    };
    let (mut armed_root, _) = old_parse_with_options(&armed_allocator, source, options);
    let _ = transform(
        &armed_allocator,
        &mut armed_root,
        TransformOptions {
            dialect: VueVersion::V2,
            prefix_identifiers: true,
            ..TransformOptions::default()
        },
        None,
    );
    let shipped_assets: Vec<&str> = armed_root.filters.iter().copied().collect();
    let s2_assets: Vec<&str> = facts.legacy.assets.iter().map(|s| s.as_str()).collect();
    if shipped_assets == s2_assets {
        legacy.assets_matched += 1;
    } else {
        // The one-sided law as an exact order-preserving oracle: the S2
        // list must be a strict subsequence of the shipped list (every
        // filter S2 registers, the shipped lane registers, in the same
        // first-seen order).
        let mut rest = shipped_assets.as_slice();
        for asset in &s2_assets {
            let Some(position) = rest.iter().position(|shipped| shipped == asset) else {
                panic!(
                    "{name}: S2 registered a filter outside the shipped registration \
                     ({asset}): {shipped_assets:?} vs {s2_assets:?}"
                );
            };
            rest = &rest[position + 1..];
        }
        assert!(
            shipped_assets.len() > s2_assets.len(),
            "{name}: asset lists diverged without narrowing: {shipped_assets:?} vs {s2_assets:?}"
        );
        legacy.assets_narrowed += 1;
    }

    // The two narrowing probes, template-level. Bind values are outside
    // the probe by design: a splittable bind value is an S2 site
    // (compared above), or the whole-split bail fired identically in
    // both lanes (the bail templates pin that agreement).
    let splittable = |text: &str| {
        parse_filters(text)
            .is_some_and(|split| split.filters.iter().all(|f| filter_name(f).is_some()))
    };
    let other_positions = s2
        .chains
        .iter()
        .flat_map(|chain| chain.branches.iter())
        .filter_map(|branch| branch.condition.as_ref().map(|t| t.as_str()))
        .chain(s2.fors.iter().map(|f| f.source.as_str()))
        .chain(s2.surfaces.iter().flat_map(|surface| {
            surface
                .ons
                .iter()
                .filter_map(|unit| unit.value.as_ref().and_then(|v| v.as_ref()))
                .chain(
                    surface
                        .directives
                        .iter()
                        .filter_map(|d| d.value.as_ref().and_then(|v| v.as_ref())),
                )
                .chain(
                    surface
                        .models
                        .iter()
                        .filter_map(|model| model.value.as_ref()),
                )
                .map(|text| text.as_str())
        }))
        .any(splittable);
    if other_positions {
        legacy.filters_other_positions += 1;
    }
    let in_compounds = s2.text_units.iter().any(|unit| {
        unit.compound
            && unit.parts.iter().any(|part| {
                part.dynamic
                    && part.text.as_ref().is_some_and(|text| {
                        parse_filters(text.as_str()).is_some_and(|split| {
                            split.filters.iter().all(|f| filter_name(f).is_some())
                        })
                    })
            })
    });
    if in_compounds {
        legacy.filters_in_compounds += 1;
    }
}

/// The authored text of every filter site, in page order — `vue.filter`
/// expressions and `ui.bind` values under the `legacy-filter` reason.
fn collect_filter_sources(ops: &[FolioOp], out: &mut Vec<String>) {
    fn expr_source(expr: &FolioExpr, out: &mut Vec<String>) {
        if let FolioExpr::Opaque { reason, source, .. } = expr
            && *reason == OpaqueReason::LegacyFilter
        {
            out.push(source.clone());
        }
    }
    for op in ops {
        match op {
            FolioOp::Element(element) => {
                for binding in &element.bindings {
                    if let FolioBinding::Bind(bind) = binding
                        && let Some(value) = &bind.value
                    {
                        expr_source(value, out);
                    }
                }
                collect_filter_sources(&element.children, out);
            }
            FolioOp::Component(component) => {
                for binding in &component.bindings {
                    if let FolioBinding::Bind(bind) = binding
                        && let Some(value) = &bind.value
                    {
                        expr_source(value, out);
                    }
                }
                collect_filter_sources(&component.children, out);
            }
            FolioOp::VueFilter(filter) => expr_source(&filter.expression, out),
            FolioOp::Text(_) | FolioOp::Interpolation(_) => {}
            FolioOp::If(if_op) => {
                for branch in &if_op.branches {
                    collect_filter_sources(&branch.ops, out);
                }
            }
            FolioOp::For(for_op) => collect_filter_sources(&for_op.ops, out),
            FolioOp::Slot(slot) => {
                for binding in &slot.bindings {
                    if let FolioBinding::Bind(bind) = binding
                        && let Some(value) = &bind.value
                    {
                        expr_source(value, out);
                    }
                }
                collect_filter_sources(&slot.fallback, out);
            }
        }
    }
}
