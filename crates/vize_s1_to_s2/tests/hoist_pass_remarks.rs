//! TS-17 for the hoist-static pass's optimization remarks (P3-13):
//! committed fixture in, pipeline run under a remark collector, **full
//! `[remarks]` page** snapshot out.
//!
//! Two independent supplements keep the snapshot from being self-blessed:
//!
//! - **The facts law.** Every remark's kind is re-derived from the pass's
//!   *published facts* plus an owner census taken from the S2 folio (a
//!   separate walk over a separate tree): one `static-subtree` per element,
//!   `applied` exactly when the level is `FullyStatic`; one `static-props`
//!   per owner that is not whole-hoistable and has a props surface,
//!   `applied` exactly when `props_hoistable`. Nothing else.
//! - **The detachment law.** Explaining changes nothing: the facts under a
//!   remark collector equal the facts under `NoObserver`.

use std::path::{Path, PathBuf};

use vize_davinci::assert_folio_snapshot;
use vize_davinci::folio::remarks::RemarkLog;
use vize_davinci::pass::{NoObserver, RemarkCollector, RemarkKind};
use vize_s0::{Allocator, Span, String};
use vize_s1::parse;
use vize_s1_to_s2::pass::hoist::{STATIC_PROPS, STATIC_SUBTREE};
use vize_s1_to_s2::pass::{StaticFacts, StaticLevel, run_transform};
use vize_s1_to_s2::{LegacyCaps, lower_with_caps};
use vize_s2::folio::{FolioOp, S2Folio};

fn fixture(name: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hoist")
        .join(name);
    String::from(
        std::fs::read_to_string(path)
            .expect("fixture reads")
            .as_str(),
    )
}

/// One run: the post-pass folio, the facts in id order, and the remarks.
struct Run {
    folio: S2Folio,
    facts: Vec<StaticFacts>,
    remarks: RemarkLog,
}

fn run(source: &str) -> Run {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let mut lowered = lower_with_caps(&allocator, &tree, &errors, LegacyCaps::VUE3);
    let mut collector = RemarkCollector::new();
    let facts = run_transform(&mut lowered, &mut collector);
    Run {
        folio: S2Folio::of(&lowered.root.ops),
        facts: facts
            .static_facts
            .sorted_entries()
            .into_iter()
            .map(|(_, fact)| *fact)
            .collect(),
        remarks: RemarkLog::new(collector.finish()),
    }
}

fn detached_facts(source: &str) -> Vec<StaticFacts> {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let mut lowered = lower_with_caps(&allocator, &tree, &errors, LegacyCaps::VUE3);
    let facts = run_transform(&mut lowered, &mut NoObserver);
    facts
        .static_facts
        .sorted_entries()
        .into_iter()
        .map(|(_, fact)| *fact)
        .collect()
}

/// An owner as the folio sees it, in page (pre-)order.
struct Owner {
    element: bool,
    span: Span,
    has_props: bool,
}

fn census(ops: &[FolioOp], out: &mut Vec<Owner>) {
    for op in ops {
        match op {
            FolioOp::Element(element) => {
                out.push(Owner {
                    element: true,
                    span: element.span,
                    has_props: !element.attributes.is_empty() || !element.bindings.is_empty(),
                });
                census(&element.children, out);
            }
            FolioOp::Component(component) => {
                out.push(Owner {
                    element: false,
                    span: component.span,
                    has_props: !component.attributes.is_empty() || !component.bindings.is_empty(),
                });
                census(&component.children, out);
            }
            FolioOp::If(if_op) => {
                for branch in &if_op.branches {
                    census(&branch.ops, out);
                }
            }
            FolioOp::For(for_op) => census(&for_op.ops, out),
            FolioOp::Slot(slot) => census(&slot.fallback, out),
            FolioOp::Text(_) | FolioOp::Interpolation(_) | FolioOp::Comment(_) => {}
        }
    }
}

type Verdict = (&'static str, u32, u32, RemarkKind);

fn kind(applied: bool) -> RemarkKind {
    if applied {
        RemarkKind::Applied
    } else {
        RemarkKind::Missed
    }
}

/// The facts law, asserted exactly.
fn assert_remarks_follow_facts(run: &Run, source: &str) {
    assert_eq!(run.facts, detached_facts(source), "remarks perturbed facts");
    let mut owners = Vec::new();
    census(&run.folio.ops, &mut owners);
    assert_eq!(owners.len(), run.facts.len(), "one fact per owner");
    let mut expected: Vec<Verdict> = Vec::new();
    for (owner, fact) in owners.iter().zip(&run.facts) {
        let whole = fact.level == StaticLevel::FullyStatic;
        if owner.element {
            expected.push((
                STATIC_SUBTREE,
                owner.span.start,
                owner.span.end,
                kind(whole),
            ));
        }
        if !whole && owner.has_props {
            expected.push((
                STATIC_PROPS,
                owner.span.start,
                owner.span.end,
                kind(fact.props_hoistable),
            ));
        }
    }
    let mut actual: Vec<Verdict> = run
        .remarks
        .remarks
        .iter()
        .map(|remark| {
            let name = if remark.name.as_str() == STATIC_SUBTREE {
                STATIC_SUBTREE
            } else {
                STATIC_PROPS
            };
            assert_eq!(
                (remark.stage.as_str(), remark.pass.as_str()),
                ("s2", "hoist-static")
            );
            assert_eq!(remark.name.as_str(), name);
            (name, remark.span.start, remark.span.end, remark.kind)
        })
        .collect();
    expected.sort_by_key(|verdict| (verdict.1, u32::MAX - verdict.2, verdict.0));
    actual.sort_by_key(|verdict| (verdict.1, u32::MAX - verdict.2, verdict.0));
    assert_eq!(actual, expected);
}

#[test]
fn the_levels_fixture_explains_every_lattice_rung() {
    let source = fixture("levels.vue");
    let run = run(&source);
    assert_folio_snapshot!(run.remarks);
    assert_remarks_follow_facts(&run, &source);
}

#[test]
fn the_positions_fixture_explains_structural_children() {
    let source = fixture("positions.vue");
    let run = run(&source);
    assert_folio_snapshot!(run.remarks);
    assert_remarks_follow_facts(&run, &source);
}

#[test]
fn the_blockers_fixture_names_every_blocker_class() {
    let source = fixture("blockers.vue");
    let run = run(&source);
    assert_folio_snapshot!(run.remarks);
    assert_remarks_follow_facts(&run, &source);
    assert_eq!(
        RemarkKind::ALL.map(|kind| run.remarks.count(kind)),
        [7, 20, 0]
    );
}
