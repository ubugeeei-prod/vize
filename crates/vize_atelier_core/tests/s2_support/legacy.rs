//! The legacy differential lane (P2-9 series 7): the shipped legacy
//! transform lane under the **V2 dialect** against the S2 legacy
//! pipeline, over the committed legacy battery.
//!
//! # How the two lanes line up
//!
//! The S2 lowering mirrors the shipped desugars byte-for-byte (`.sync`,
//! `slot-scope`/`scope`, the v-on event sugar — `lower::legacy`'s
//! measured split), so every structural projection the plain comparator
//! already runs compares **directly**: the run-1 legacy transform (V2
//! dialect, default options otherwise) produces the same desugared
//! surface the S2 ops carry, `.sync` product pairs folding back into
//! contracts on both sides by the one span-sharing rule.
//!
//! Filters need one more step, because the shipped rewrite only runs
//! under identifier prefixing (measured: every `process_expression`
//! call site is `prefix_identifiers || is_ts`-gated — a shipped
//! coupling, recorded): the run-1 tree carries filter chains verbatim
//! (so text/surface projections again compare directly), and the filter
//! *structure* is checked two ways —
//!
//! - **per site** against the shipped splitter itself
//!   (`steps::legacy_filters::parse_filters`, made `pub` for exactly
//!   this — the installment-6 `is_constant_simple_expression`
//!   precedent): base and every segment byte-equal, names through the
//!   shipped `filter_name`;
//! - **per template** against a **filter-armed second legacy run**
//!   (V2 + `prefix_identifiers`, the hoist installment's armed-run
//!   pattern): the shipped `RootNode::filters` registration versus the
//!   S2 [`LegacyFacts::assets`]. Where the armed run registers more —
//!   the S2 split deliberately covers Vue 2's documented positions
//!   (mustache + `v-bind` values) while the shipped rewrite reaches
//!   every prefixed expression, and compound-merged interpolations stay
//!   the Compound producer's parts — the S2 list must be a **subset**
//!   (the one-sided law: everything S2 registers, the shipped lane
//!   registers) and the template is counted, never averaged
//!   (`assets_narrowed`, explained by the `filters_other_positions` /
//!   `filters_in_compounds` probes).
//!
//! The hoist-decision half stays V3-scoped (its oracle set —
//! classifier, const rule, comment taint — was measured and pinned on
//! the default dialect); a legacy-dialect hoist differential would
//! re-run the same lattice over desugared products and is deferred with
//! the exit gate. Everything else runs.
//!
//! [`LegacyFacts::assets`]: vize_ricalco::pass::LegacyFacts

// Each test binary uses the subset of the shared support tree it needs
// (the `vize_ricalco/tests/support` convention): the plain witness and
// corpus binaries compile this module without calling it.
#![allow(dead_code, unused_imports)]

use vize_atelier_core::parser::parse_with_options as old_parse_with_options;
use vize_atelier_core::{ParserOptions, TransformOptions, transform};
use vize_carton::Allocator;
use vize_carton::config::VueVersion;
use vize_davinci::diagnostic::Severity;
use vize_davinci::pass::NoObserver;
use vize_disegno::folio::DisegnoFolio;
use vize_ricalco::pass::{TRANSFORM_LANE_FLAG, run_transform_legacy};

use super::{
    Counters, checks, old_lane, s2_lane, slots, slots_old, surface_check, surface_old, text,
    text_old,
};

pub use super::legacy_batt::{LEGACY_BATTERY, LegacyCounters};

/// Dual-run `source` under the **V2 dialect** through both lanes and
/// compare every projection the lanes share.
///
/// # Panics
///
/// Panics on any divergence inside the compared domain (TS-25).
pub fn compare_legacy(
    name: &str,
    source: &str,
    counters: &mut Counters,
    legacy: &mut LegacyCounters,
) {
    counters.templates_seen += 1;
    if std::env::var(TRANSFORM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        counters.skipped_legacy_flag += 1;
        return;
    }

    // Run 1: the shipped legacy lane under the V2 dialect, default
    // options otherwise (the desugars are dialect-gated, not
    // prefixing-gated), `is_pre_tag` as the plain comparator sets it.
    let old_allocator = Allocator::new();
    let options = ParserOptions {
        is_pre_tag: |tag| tag == "pre",
        ..ParserOptions::default()
    };
    let (mut root, parse_errors) = old_parse_with_options(&old_allocator, source, options);
    if parse_errors.iter().any(|error| !error.code.is_recovery()) {
        counters.skipped_old_parse_errors += 1;
        return;
    }
    let _ = transform(
        &old_allocator,
        &mut root,
        TransformOptions {
            dialect: VueVersion::V2,
            ..TransformOptions::default()
        },
        None,
    );
    let mut old_chains = Vec::new();
    let mut old_fors = Vec::new();
    old_lane::collect(&root.children, &mut old_chains, &mut old_fors);
    let mut old_units = Vec::new();
    let mut old_outlets = Vec::new();
    slots_old::collect_old(&root.children, source, &mut old_units, &mut old_outlets);
    let mut old_text_units = Vec::new();
    text_old::collect_units(&root.children, &mut old_text_units);
    let mut old_surfaces = Vec::new();
    surface_old::collect_surfaces(
        &root.children,
        false,
        &mut old_surfaces,
        &mut counters.surfaces,
    );

    // The S2 lane: sinopia parse → the legacy lowering → the legacy
    // pipeline.
    let s2_allocator = Allocator::new();
    let (tree, surface_errors) = vize_sinopia::parse(&s2_allocator, source);
    let mut lowered = vize_ricalco::lower_legacy(
        &s2_allocator,
        &tree,
        &surface_errors,
        vize_ricalco::LegacyVueLine::V2,
    );
    if lowered
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        counters.skipped_s2_errors += 1;
        return;
    }
    let facts = run_transform_legacy(&mut lowered, &mut NoObserver);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    let s2 = s2_lane::collect(
        &folio,
        &s2_lane::Tables {
            if_facts: &facts.if_facts,
            slot_facts: &facts.slot_facts,
            text_facts: &facts.text_facts,
            model_faults: &facts.model_faults,
        },
    );

    // The shared structural projections, exactly as the plain
    // comparator runs them (the hoist half stays V3-scoped — module
    // docs).
    checks::check(name, source, &old_chains, &s2.chains, counters);
    checks::check_fors(name, source, &old_fors, &s2.fors, counters);
    slots::check(
        name,
        source,
        &old_units,
        &s2.units,
        &old_outlets,
        &s2.outlets,
        &mut counters.slots,
    );
    counters.text.rawtext_excluded += s2.text_rawtext_excluded;
    if s2.has_table {
        counters.surfaces.table_templates += 1;
    } else {
        counters.surfaces.models_invalid += s2.models_invalid;
        counters.surfaces.keys_excluded += s2.keys_excluded;
        surface_check::check(
            name,
            source,
            &old_surfaces,
            &s2.surfaces,
            &mut counters.surfaces,
        );
    }
    let has_vpre = lowered
        .provenance
        .iter()
        .any(|record| record.rule.as_str() == "defer.v-pre");
    if has_vpre {
        counters.text.vpre_templates += 1;
    } else {
        text::check(
            name,
            source,
            &old_text_units,
            &s2.text_units,
            &mut counters.text,
        );
    }

    // The filter half: sites, assets, probes (module docs; split
    // under the source budget).
    super::legacy_filters_check::check_filters(name, source, &folio, &facts, &s2, legacy);

    // The desugar mirrors, counted from the S2 lane's own records.
    let rule_count = |rule: &str| {
        lowered
            .provenance
            .iter()
            .filter(|record| record.rule.as_str() == rule)
            .count() as u64
    };
    legacy.syncs += rule_count("normalize.legacy.sync");
    legacy.scoped_slots += rule_count("normalize.legacy.slot-scope");
    legacy.natives += rule_count("normalize.legacy.native");
    legacy.keycodes += rule_count("normalize.legacy.keycode");

    counters.compared += 1;
}
