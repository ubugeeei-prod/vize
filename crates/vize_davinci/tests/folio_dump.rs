//! `FolioDump` behavior: page naming, and the `--folio-after-change` hash
//! gate (P2-13).

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use vize_davinci::folio::dump::FolioDump;
use vize_davinci::pass::{Fusability, PassDesc, PassEvent, PassKind, Pipeline, Preserved};

const ALPHA: PassDesc = PassDesc::new(
    "alpha",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const BETA: PassDesc = PassDesc::new(
    "beta",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const PASSES: &[PassDesc] = &[ALPHA, BETA];
const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

fn event(pass_index: usize) -> PassEvent<'static> {
    let group = PIPELINE.group(0).expect("the fused pipeline has one group");
    PassEvent {
        pipeline: &PIPELINE,
        group_index: 0,
        group,
        pass_index,
    }
}

#[test]
fn ungated_dumps_emit_a_page_per_pass_in_emission_order() {
    let mut dump = FolioDump::new(false);
    dump.seed("artifact v1\n");
    dump.after_pass(&event(0), "artifact v1\n");
    dump.after_pass(&event(1), "artifact v1\n");
    let names: Vec<&str> = dump.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(names, ["000-s2.alpha.folio", "001-s2.beta.folio"]);
    assert_eq!(dump.pages[0].text.as_str(), "artifact v1\n");
    assert_eq!(dump.pages[1].text.as_str(), "artifact v1\n");
}

#[test]
fn the_gate_emits_nothing_when_no_pass_changes_the_artifact() {
    let mut dump = FolioDump::new(true);
    dump.seed("artifact v1\n");
    dump.after_pass(&event(0), "artifact v1\n");
    dump.after_pass(&event(1), "artifact v1\n");
    assert_eq!(dump.pages.len(), 0);
}

#[test]
fn the_gate_emits_exactly_the_changing_passes() {
    let mut dump = FolioDump::new(true);
    dump.seed("artifact v1\n");
    dump.after_pass(&event(0), "artifact v1\n");
    dump.after_pass(&event(1), "artifact v2\n");
    let names: Vec<&str> = dump.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(names, ["000-s2.beta.folio"]);
    assert_eq!(dump.pages[0].text.as_str(), "artifact v2\n");
}

#[test]
fn an_unseeded_gated_dump_emits_its_first_page_unconditionally() {
    let mut dump = FolioDump::new(true);
    dump.after_pass(&event(0), "artifact v1\n");
    dump.after_pass(&event(1), "artifact v1\n");
    let names: Vec<&str> = dump.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(names, ["000-s2.alpha.folio"]);
}
