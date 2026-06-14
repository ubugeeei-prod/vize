//! Human-facing fallback and plate-request reports.
//!
//! Fallback and plate facts are recorded as cheap profile counters on the hot
//! path. This module turns a recorded [`SourceAtlasFallbackSet`] or
//! [`SourceAtlasRegistry`] into a first-class, explainable report so a tool
//! (inspector, Vitrine payload, CLI) can show *why* a lane took each fallback
//! and which plates it requested without reconstructing the decision from
//! output strings. Rendering is a requested view, never normal-path work.

use super::{SourceAtlasFallback, SourceAtlasFallbackSet, SourceAtlasRegistry};
use vize_carton::String;

/// Render the observed fallback reasons as a stable, line-oriented report.
///
/// Each present reason is rendered as `counter — description`, in the same
/// stable order as [`SourceAtlasFallbackSet::iter`]. An empty set renders a
/// single "no fallbacks" line so the report is never blank.
pub fn render_fallback_report(fallbacks: SourceAtlasFallbackSet) -> String {
    use std::fmt::Write as _;

    if fallbacks.is_empty() {
        return String::from("no fallbacks observed");
    }

    let mut report = String::default();
    for (index, fallback) in fallbacks.iter().enumerate() {
        if index > 0 {
            report.push('\n');
        }
        let _ = write!(
            report,
            "{} — {}",
            fallback.profile_counter(),
            fallback.description()
        );
    }
    report
}

/// Render which plates a product lane requested, for profile/inspector views.
///
/// This is the lane-shaped counterpart to the fallback report: any toolchain
/// lane (compiler, linter, typecheck, source-map, ...) can show which source,
/// plate, target, and coordinate facts it asked for, plus any fallbacks it
/// took. Derived entirely from a registry the lane already built, so it adds no
/// new analysis to the hot path.
pub fn render_registry_report(registry: SourceAtlasRegistry) -> String {
    use std::fmt::Write as _;

    let mut report = String::default();
    let _ = write!(report, "lane: {}", registry.lane.profile_counter());

    report.push_str("\nrequests:");
    let mut any_request = false;
    registry.visit_requests(|request| {
        any_request = true;
        let _ = write!(report, "\n  {}", request.profile_counter());
    });
    if !any_request {
        report.push_str("\n  (none)");
    }

    report.push_str("\nfallbacks:");
    if registry.fallbacks.is_empty() {
        report.push_str("\n  no fallbacks observed");
    } else {
        for fallback in registry.fallbacks.iter() {
            let _ = write!(
                report,
                "\n  {} — {}",
                fallback.profile_counter(),
                fallback.description()
            );
        }
    }
    report
}

/// Render every known fallback reason and its description, regardless of
/// whether it was observed. Useful for documentation and inspector legends.
pub fn render_fallback_legend() -> String {
    render_fallback_report(SourceAtlasFallback::KNOWN.iter().copied().fold(
        SourceAtlasFallbackSet::empty(),
        SourceAtlasFallbackSet::with,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_set_reports_no_fallbacks() {
        assert_eq!(
            render_fallback_report(SourceAtlasFallbackSet::empty()),
            "no fallbacks observed"
        );
    }

    #[test]
    fn report_explains_each_observed_reason_in_stable_order() {
        let set = SourceAtlasFallbackSet::empty()
            .with(SourceAtlasFallback::VaporSsr)
            .with(SourceAtlasFallback::LegacySyntaxCompatibility)
            .with(SourceAtlasFallback::SourceMapFragmentUnavailable);

        let report = render_fallback_report(set);

        assert_eq!(
            report,
            "atelier.fallback.source_map.fragment_unavailable — the producing Atelier emitted no source-map fragment to register\n\
             atelier.fallback.vapor_ssr — Vapor SSR is unimplemented, so standard SSR output was used\n\
             atelier.fallback.legacy_syntax_compatibility — a pre-Vue-3 dialect required a compatibility path"
        );
    }

    #[test]
    fn compiler_registry_report_lists_requests_and_fallbacks() {
        use crate::source_atlas::{
            SourceAtlasCoordinate, SourceAtlasPlate, SourceAtlasSource, SourceAtlasTarget,
        };
        use vize_carton::config::VueVersion;

        let coordinate = SourceAtlasCoordinate::from(VueVersion::V2);
        let registry = SourceAtlasRegistry::compiler()
            .with_source(SourceAtlasSource::Sfc)
            .with_plate(SourceAtlasPlate::Template)
            .with_target(SourceAtlasTarget::Ssr)
            .with_coordinate(coordinate)
            .with_optional_fallback(coordinate.compatibility_fallback());

        assert_eq!(
            render_registry_report(registry),
            "lane: atelier.profile.lane.compiler\n\
             requests:\n  \
             atelier.profile.input.sfc\n  \
             atelier.profile.source.template\n  \
             atelier.profile.target.ssr\n  \
             atelier.profile.dialect.vue2\n\
             fallbacks:\n  \
             atelier.fallback.legacy_syntax_compatibility — a pre-Vue-3 dialect required a compatibility path"
        );
    }

    #[test]
    fn each_lane_report_names_its_lane_and_a_clean_run_has_no_fallbacks() {
        use crate::source_atlas::{SourceAtlasLane, SourceAtlasPlate};

        let lanes = [
            (SourceAtlasLane::Compiler, "atelier.profile.lane.compiler"),
            (SourceAtlasLane::Linter, "atelier.profile.lane.linter"),
            (SourceAtlasLane::Typecheck, "atelier.profile.lane.typecheck"),
            (
                SourceAtlasLane::SourceMap,
                "atelier.profile.lane.source_map",
            ),
        ];

        for (lane, counter) in lanes {
            let registry = SourceAtlasRegistry::new(lane).with_plate(SourceAtlasPlate::Croquis);
            let report = render_registry_report(registry);
            assert!(report.starts_with("lane: "));
            assert!(report.contains(counter), "{counter} report: {report}");
            assert!(report.contains("no fallbacks observed"));
            assert!(report.contains("atelier.profile.plate.croquis"));
        }
    }

    #[test]
    fn legend_covers_every_known_reason() {
        let legend = render_fallback_legend();
        assert_eq!(legend.lines().count(), SourceAtlasFallback::KNOWN.len());
        for fallback in SourceAtlasFallback::KNOWN {
            assert!(legend.contains(fallback.profile_counter()));
        }
    }
}
