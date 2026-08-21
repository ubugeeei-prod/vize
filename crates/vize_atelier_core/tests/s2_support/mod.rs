//! The P2-9 differential comparator: legacy transform lane vs the S2
//! passes, compared at the DOM-output level — the facts DOM codegen
//! consumes from an if structure (chain order, branch count and order,
//! condition text, branch keys — series 1) and from a for structure
//! (document order, source text, value/key/index alias texts — series
//! 2: `renderList`'s whole input surface; the iterated element's `key`
//! prop stays element surface in both lanes and is compared there by
//! neither, exactly as legacy codegen reads it per vnode). The
//! byte-level DOM comparison arrives when a DOM backend exists to emit
//! from S2 (P2-11); until then this projection is the strongest
//! output-determining oracle the transform lane has, and TS-11
//! (`corpus-diff --surface compiler`) holds the actual output bytes
//! still. Series 3 adds the slot projection — component slot grouping
//! (canonical names with their invented-vs-authored class, params
//! texts, group order) and outlet names — in the [`slots`] module.
//! Series 4 adds the text projection — the merged text-unit surface
//! (`createTextVNode` boundaries with their static/dynamic parts,
//! condensed text included) — in the [`text`] module. Series 5 adds
//! the binding-surface projection — per owner: static attributes,
//! `v-bind`/`v-on` units, custom directives, and the reconstructed
//! `v-model` contract — in the [`surface`] module, and turns the
//! dynamic-key, wrapper-key, and outlet-key skip classes into
//! comparisons.
//!
//! # Why this lives in test space (the dependency direction)
//!
//! `vize_atelier_core` is published; the Davinci crates are not, and
//! the release gate (`tests/tooling/moonbit-publish-crates.test.ts`)
//! rejects a published crate whose release graph names an unpublished
//! one. Dev-dependencies with no version requirement are stripped on
//! publish — the exact carve-out the gate encodes — so the S2 lane and
//! this comparator ride dev-deps, never the compile path. The P1-7
//! in-`src` comparator shape does not apply here because the shipped
//! path has no migrated read yet: the S2 lane runs *beside* the legacy
//! lane, not inside it.
//!
//! # The lane flag (charter #26)
//!
//! `VIZE_DAVINCI_TRANSFORM=legacy` disarms the dual-run: the legacy
//! lane is then the only thing exercised, which is also the shipped
//! default. The plain witness pins non-zero comparison counts, so a
//! flag or cfg regression that silently disarms the lane fails loudly.
//!
//! # Skip classes are counted, never silent
//!
//! The two lanes parse with different S1 front ends, and the S1 v1
//! scope records deliberate tree deviations (no implied-end-tag
//! reconciliation, no entity decoding). The comparator therefore
//! compares exactly the domain both lanes claim to model — templates
//! neither lane **rejects** — and **counts** everything it declines:
//! legacy hard parse errors, S2 error diagnostics (evaluated pre-pass,
//! so a pass's own diagnostics never mask a comparison; a malformed or
//! expressionless `v-for` skips here, matching the legacy transform's
//! refusal to build a `ForNode` from it), the legacy dynamic-argument
//! `:[key]` quirk, compound rebuilds of any expression position, the
//! slot projection's counted classes — conditional carriers, the
//! `v-slots` spread, filler-only implicit defaults ([`slots`] module
//! docs, series 3) — and the surface projection's counted classes
//! ([`surface`] module docs, series 5: still-deferred built-ins,
//! wrapper props, entity-bearing values, dynamic-argument and
//! pattern-scoped models).
//! Recovery-level legacy notes (`ErrorCode::is_recovery` — spec repairs
//! such as self-closing rewrites the parser already applied) do **not**
//! skip: the first corpus run measured them on 3,027 of 12,021
//! templates, and comparing them held zero divergence, so excluding
//! them would have quietly shrunk the claim by a quarter. Divergence
//! inside the compared domain panics (TS-25): investigate, never
//! average.

pub mod battery;
mod checks;
pub mod hoist;
pub mod hoist_old;
pub mod hoist_owner;
pub mod hoist_walk;
#[cfg(feature = "legacy")]
pub mod legacy;
#[cfg(feature = "legacy")]
pub mod legacy_batt;
#[cfg(feature = "legacy")]
pub mod legacy_filters_check;
pub mod old_lane;
pub mod s2_lane;
pub mod slots;
pub mod slots_old;
pub mod surface;
pub mod surface_check;
pub mod surface_old;
pub mod surface_old_help;
pub mod surface_s2;
pub mod text;
pub mod text_old;

// Unused in the legacy witness binary, which runs its own battery (the
// shared-test-module convention `vize_ricalco/tests/support` documents).
#[cfg_attr(feature = "legacy", allow(unused_imports))]
pub use battery::BATTERY;
pub use hoist::HoistCounters;
pub use slots::SlotCounters;
pub use surface::SurfaceCounters;
pub use text::TextCounters;

use vize_atelier_core::parser::parse_with_options as old_parse_with_options;
use vize_atelier_core::{ParserOptions, TransformOptions, transform};
use vize_carton::Allocator;
use vize_davinci::diagnostic::Severity;
use vize_davinci::pass::NoObserver;
use vize_disegno::folio::DisegnoFolio;
use vize_ricalco::pass::{TRANSFORM_LANE_FLAG, run_transform};

/// The comparator's process-global accounting, pinned exactly by the
/// plain witness and printed by the corpus entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counters {
    /// Templates handed to [`compare`].
    pub templates_seen: u64,
    /// Templates dual-run to completion with zero divergence.
    pub compared: u64,
    /// `VIZE_DAVINCI_TRANSFORM=legacy` disarmed the S2 lane.
    pub skipped_legacy_flag: u64,
    /// The legacy parser reported a **hard** error (recovery notes
    /// compare — see the module docs); outside both lanes' shared
    /// domain.
    pub skipped_old_parse_errors: u64,
    /// The S2 lowering reported an `Error` diagnostic (pre-pass).
    pub skipped_s2_errors: u64,
    /// `ui.if` ops compared.
    pub if_ops: u64,
    /// Branches compared.
    pub branches: u64,
    /// Static-key value comparisons that ran (carriers, outlets
    /// included since series 5).
    pub keys_static: u64,
    /// Dynamic-key text comparisons that ran (series 5 closed the
    /// deferral class of the same name).
    pub keys_dynamic: u64,
    /// `<template v-if>` wrapper keys compared (series 5 closed the
    /// installment-1 drop through the lowering's capture channel).
    pub keys_wrapper: u64,
    /// The legacy arg-content quirk: a dynamic-argument `:[key]` the
    /// legacy lane lifts as the branch key; S2 counts, never imitates.
    pub keys_dynamic_arg: u64,
    /// A legacy compound key rebuild: no single source text.
    pub keys_compound: u64,
    /// Old lane rebuilt a compound condition; no single source text to
    /// compare.
    pub conditions_compound: u64,
    /// `ui.for`s compared (series 2).
    pub for_ops: u64,
    /// Value-alias text comparisons that ran.
    pub for_values: u64,
    /// Key-alias text comparisons that ran.
    pub for_keys: u64,
    /// Index-alias text comparisons that ran.
    pub for_indexes: u64,
    /// Both lanes agreed the value alias is absent (`v-for=" in xs"`).
    pub for_values_absent: u64,
    /// Old lane rebuilt a compound source or alias; no single source
    /// text to compare.
    pub for_compound: u64,
    /// The slot half (series 3): units, groups, outlets, and the
    /// counted classes ([`slots`] module docs).
    pub slots: SlotCounters,
    /// The text half (series 4): units, parts, compounds, and the
    /// counted classes ([`text`] module docs).
    pub text: TextCounters,
    /// The binding-surface half (series 5): owners, attrs, binds, ons,
    /// directives, models, and the counted classes ([`surface`] module
    /// docs).
    pub surfaces: SurfaceCounters,
    /// The hoist-decision half (series 6): compared positions, agreed
    /// whole/props hoists, and the counted classes ([`hoist`] module
    /// docs).
    pub hoist: HoistCounters,
}

/// Dual-run `source` through both lanes and compare the projections.
///
/// # Panics
///
/// Panics on any divergence inside the compared domain (TS-25), with
/// the template and both projections in the message.
pub fn compare(name: &str, source: &str, counters: &mut Counters) {
    counters.templates_seen += 1;
    if std::env::var(TRANSFORM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        counters.skipped_legacy_flag += 1;
        return;
    }

    // Legacy lane: the shipped parse + transform. Options stay default
    // except `is_pre_tag`, which takes the shipped DOM configuration
    // (`crates/vize_atelier_dom/src/compile/stage_options.rs`) so both
    // lanes exempt `<pre>` from whitespace condensing the same way —
    // the default `|_| false` would condense inside `<pre>`, which no
    // shipped compile does. `is_pre_tag` feeds only the condense
    // strategy, so every pre-series-4 projection is unaffected.
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
    let _transform_errors = transform(&old_allocator, &mut root, TransformOptions::default(), None);
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

    // S2 lane: sinopia parse -> ricalco lower -> the S2 passes through
    // the P2-2 pass manager (verifier between passes in debug).
    let s2_allocator = Allocator::new();
    let (tree, surface_errors) = vize_sinopia::parse(&s2_allocator, source);
    let mut lowered = vize_ricalco::lower(&s2_allocator, &tree, &surface_errors);
    if lowered
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        counters.skipped_s2_errors += 1;
        return;
    }
    let facts = run_transform(&mut lowered, &mut NoObserver);
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
        // The legacy in-table tree construction class ([`surface`]
        // module docs): owner order and count can genuinely differ
        // inside table subtrees, so the surface half skips whole.
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
    // The text projection's template-level v-pre class ([`text`] module
    // docs): the legacy parser honours `v-pre` and then erases it from
    // its tree, so the deterministic detector is the S2 lowering's own
    // deferral record.
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

    // The hoist-decision half (series 6): the shipped hoisting run's
    // actual mutations against the S2 facts' predictions. Template-
    // level classes first (each detector's reasoning: [`hoist`] module
    // docs), then the shape pre-check (the pairing contract), then the
    // hoist-armed second legacy run and the three-tree walk.
    let models_excluded = s2.models_invalid > 0
        || s2
            .surfaces
            .iter()
            .any(|surface| surface.pattern_scoped && !surface.models.is_empty());
    if has_vpre {
        counters.hoist.vpre_templates += 1;
    } else if s2.has_table {
        counters.hoist.table_templates += 1;
    } else if models_excluded {
        counters.hoist.models_templates += 1;
    } else {
        let mut scan = hoist_old::TemplateScan::default();
        hoist_old::scan_template(&root.children, &mut scan);
        let mut old_shape = vize_carton::String::default();
        hoist_old::shape_of(&root.children, &mut old_shape);
        let mut s2_shape = vize_carton::String::default();
        hoist::shape_of_s2(&folio.ops, &mut s2_shape);
        if scan.classifier {
            counters.hoist.classifier_templates += 1;
        } else if scan.consts {
            counters.hoist.consts_templates += 1;
        } else if old_shape != s2_shape {
            counters.hoist.tree_templates += 1;
        } else {
            let hoist_allocator = Allocator::new();
            let options = ParserOptions {
                is_pre_tag: |tag| tag == "pre",
                ..ParserOptions::default()
            };
            let (mut hoisted_root, _) = old_parse_with_options(&hoist_allocator, source, options);
            let _ = transform(
                &hoist_allocator,
                &mut hoisted_root,
                TransformOptions {
                    hoist_static: true,
                    ..TransformOptions::default()
                },
                None,
            );
            hoist::check(
                name,
                source,
                &root.children,
                &hoisted_root.children,
                &folio.ops,
                &facts.static_facts,
                &mut counters.hoist,
            );
        }
    }
    counters.compared += 1;
}
