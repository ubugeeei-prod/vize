//! Shared oracle for the P2-8 suites: the totality/soundness checks every
//! lowered artifact must pass, whatever the input, plus the owned
//! artifact snapshot the exact-pin suites compare against.

// Each test binary uses the subset of these helpers it needs.
#![allow(dead_code)]

mod authored;

use vize_davinci::diagnostic::Diagnostic;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::side_table::SideTable;
use vize_s0::{Allocator, String};
use vize_s1::parse;
use vize_s1_to_s2::{LegacyCaps, Lowered, lower_with_caps};
use vize_s2::folio::DisegnoFolio;
use vize_s2::provenance::ProvenanceRecord;
use vize_s2::scope::ScopeFacts;
use vize_s2::verify::{Rigor, Violation, verify, verify_table};

pub use authored::assert_authored_artifact;

/// The owned snapshot of one lowering, for exact-equality pins.
#[derive(Debug)]
pub struct Artifact {
    /// The canonical `Full`-mode folio text (owned, arena-free).
    pub folio: String,
    /// The minted op count.
    pub op_count: u32,
    /// The diagnostics, in emission order.
    pub diagnostics: Vec<Diagnostic>,
    /// The provenance records, in decision order.
    pub provenance: Vec<ProvenanceRecord>,
    /// The scope facts, as (op index, facts), ascending.
    pub scopes: Vec<(u32, ScopeFacts)>,
}

/// Lower `source` and clone everything owned out of the arena's shadow.
pub fn artifact(source: &str) -> Artifact {
    artifact_caps(source, LegacyCaps::VUE3)
}

/// [`artifact`] under an explicit Vue dialect.
pub fn artifact_caps(source: &str, caps: LegacyCaps) -> Artifact {
    with_lowered_caps(source, caps, |lowered, folio| Artifact {
        folio: folio.print_to_string(FolioMode::Full),
        op_count: lowered.op_count,
        diagnostics: lowered.diagnostics.clone(),
        provenance: lowered.provenance.clone(),
        scopes: lowered
            .scopes
            .sorted_entries()
            .into_iter()
            .map(|(id, facts)| (id.index(), facts.clone()))
            .collect(),
    })
}

/// Parse + lower `source` and hand the artifact (with its owned folio
/// mirror) to `f`.
pub fn with_lowered<R>(source: &str, f: impl FnOnce(&Lowered<'_>, &DisegnoFolio) -> R) -> R {
    with_lowered_caps(source, LegacyCaps::VUE3, f)
}

/// [`with_lowered`] under an explicit Vue dialect.
pub fn with_lowered_caps<R>(
    source: &str,
    caps: LegacyCaps,
    f: impl FnOnce(&Lowered<'_>, &DisegnoFolio) -> R,
) -> R {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let lowered = lower_with_caps(&allocator, &tree, &errors, caps);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    f(&lowered, &folio)
}

/// Parse + lower `source`, run the S2 transform pipeline (P2-9) under
/// the budget observer, and hand the post-pass artifact to `f`.
pub fn with_transformed<R>(
    source: &str,
    f: impl FnOnce(
        &Lowered<'_>,
        &DisegnoFolio,
        &vize_s1_to_s2::pass::S2Facts,
        &vize_davinci::pass::BudgetObserver,
    ) -> R,
) -> R {
    with_transformed_caps(source, LegacyCaps::VUE3, f)
}

/// [`with_transformed`] under an explicit Vue dialect.
pub fn with_transformed_caps<R>(
    source: &str,
    caps: LegacyCaps,
    f: impl FnOnce(
        &Lowered<'_>,
        &DisegnoFolio,
        &vize_s1_to_s2::pass::S2Facts,
        &vize_davinci::pass::BudgetObserver,
    ) -> R,
) -> R {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let mut lowered = lower_with_caps(&allocator, &tree, &errors, caps);
    let mut budget = vize_davinci::pass::BudgetObserver::new();
    let facts = vize_s1_to_s2::pass::run_transform(&mut lowered, &mut budget);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    f(&lowered, &folio, &facts, &budget)
}

/// The soundness oracle of [`assert_sound`], applied **after** the S2
/// transform pipeline ran: the pass may move facts, never break the
/// accounting, the invariants, a side-table key, or the round-trip.
pub fn assert_transformed_sound(source: &str, context: &str) {
    assert_transformed_sound_caps(source, LegacyCaps::VUE3, context);
}

/// [`assert_transformed_sound`] under an explicit Vue dialect.
pub fn assert_transformed_sound_caps(source: &str, caps: LegacyCaps, context: &str) {
    with_transformed_caps(source, caps, |lowered, folio, facts, _budget| {
        assert_authored_artifact(source, lowered);
        assert_eq!(
            u64::from(lowered.op_count),
            folio.op_count(),
            "id accounting diverged post-pass: {context}"
        );
        assert_eq!(
            verify(folio, Rigor::Canonical),
            Vec::<Violation>::new(),
            "verifier rejected post-pass output: {context}"
        );
        assert_eq!(
            verify_table(folio, &lowered.scopes),
            Vec::<Violation>::new(),
            "a scope key dangles post-pass: {context}"
        );
        assert_eq!(
            verify_table(folio, &facts.if_facts),
            Vec::<Violation>::new(),
            "an if-facts key dangles: {context}"
        );
        assert_eq!(
            verify_table(folio, &facts.for_facts),
            Vec::<Violation>::new(),
            "a for-facts key dangles: {context}"
        );
        assert_eq!(
            verify_table(folio, &facts.slot_facts),
            Vec::<Violation>::new(),
            "a slot-facts key dangles: {context}"
        );
        assert_eq!(
            verify_table(folio, &lowered.texts),
            Vec::<Violation>::new(),
            "a recorded text-run key dangles: {context}"
        );
        assert_eq!(
            verify_table(folio, &facts.text_facts),
            Vec::<Violation>::new(),
            "a text-facts key dangles: {context}"
        );
        assert_eq!(
            verify_table(folio, &facts.model_faults),
            Vec::<Violation>::new(),
            "a model-fault key dangles: {context}"
        );
        assert_eq!(
            verify_table(folio, &facts.static_facts),
            Vec::<Violation>::new(),
            "a static-facts key dangles: {context}"
        );
        let printed = folio.print_to_string(FolioMode::Full);
        let reparsed = DisegnoFolio::parse(printed.as_str()).unwrap_or_else(|error| {
            panic!("post-pass folio re-parse failed ({context}): {error:?}")
        });
        assert_eq!(
            &reparsed, folio,
            "post-pass folio round-trip diverged: {context}"
        );
    });
}

/// The soundness oracle, asserted for every input the suites touch:
///
/// 1. **Id accounting** — the minted count equals the folio's op count,
///    so page-order ids resolve by construction.
/// 2. **Verifier-clean at `Canonical` rigor** — the lowering builds
///    canonical `ui.if` shapes from birth and every span nests.
/// 3. **Side tables resolve** — every scope key and every provenance
///    node id names an op the artifact numbers.
/// 4. **Folio round-trip** — `parse(print(v)) == v` structurally and the
///    re-print is byte-identical (TS-16 applied to lowered output).
pub fn assert_sound(source: &str, context: &str) {
    assert_sound_caps(source, LegacyCaps::VUE3, context);
}

/// [`assert_sound`] under an explicit Vue dialect.
pub fn assert_sound_caps(source: &str, caps: LegacyCaps, context: &str) {
    with_lowered_caps(source, caps, |lowered, folio| {
        assert_authored_artifact(source, lowered);
        assert_eq!(
            u64::from(lowered.op_count),
            folio.op_count(),
            "id accounting diverged: {context}"
        );
        assert_eq!(
            verify(folio, Rigor::Canonical),
            Vec::<Violation>::new(),
            "verifier rejected lowered output: {context}"
        );
        assert_eq!(
            verify_table(folio, &lowered.scopes),
            Vec::<Violation>::new(),
            "a scope key dangles: {context}"
        );
        let provenance_ids: SideTable<()> = lowered
            .provenance
            .iter()
            .filter_map(|record| record.node.map(|node| (node, ())))
            .collect();
        assert_eq!(
            verify_table(folio, &provenance_ids),
            Vec::<Violation>::new(),
            "a provenance node dangles: {context}"
        );
        let printed = folio.print_to_string(FolioMode::Full);
        let reparsed = DisegnoFolio::parse(printed.as_str())
            .unwrap_or_else(|error| panic!("folio re-parse failed ({context}): {error:?}"));
        assert_eq!(&reparsed, folio, "folio round-trip diverged: {context}");
        let reprinted = reparsed.print_to_string(FolioMode::Full);
        assert_eq!(reprinted, printed, "folio re-print diverged: {context}");
    });
}
