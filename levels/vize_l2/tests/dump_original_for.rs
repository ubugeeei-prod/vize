//! Original For diagnostics preserve a numeric readback, never a File receipt.

use vize_l0::dump::{Dump, Mode};
use vize_l0::{Span, String};
use vize_l2::dump::{Op, OriginalFor, Page, Text};
use vize_l2::verify::{Rigor, ViolationCode, verify};

const CANONICAL: &str = "\
[l2-dump-v2]
ops=3

[l2-dump-v2.ops]
ui.for original-ref=7 @0:40
  ui.for original-ref=8 @10:30
    ui.text \"body\" @20:24

";

fn diagnostics() -> Page {
    Page {
        ops: vec![Op::OriginalFor(OriginalFor {
            node: 7,
            ops: vec![Op::OriginalFor(OriginalFor {
                node: 8,
                ops: vec![Op::Text(Text {
                    content: String::from("body"),
                    span: Span::new(20, 24),
                })],
                span: Span::new(10, 30),
            })],
            span: Span::new(0, 40),
        })],
    }
}

#[test]
fn diagnostic_original_references_and_regions_round_trip() {
    let page = diagnostics();
    assert_eq!(page.op_count(), 3);
    assert_eq!(page.print_to_string(Mode::Full).as_str(), CANONICAL);
    assert_eq!(Page::parse(CANONICAL), Ok(page));
}

#[test]
fn display_preserves_readback_and_elides_only_spans() {
    assert_eq!(
        diagnostics().print_to_string(Mode::Display).as_str(),
        "[l2-dump-v2]\nops=3\n\n[l2-dump-v2.ops]\nui.for original-ref=7\n  ui.for original-ref=8\n    ui.text \"body\"\n\n"
    );
}

#[test]
fn original_diagnostics_cannot_accept_neutral_alias_payloads() {
    for line in [
        "ui.for original-ref=4294967296 @0:40",
        "ui.for original-ref=not-a-node @0:40",
        "ui.for original-ref=7 source=js(\"items\" @1:6) @0:40",
        "ui.for original-ref=7 @0:40\n  attr value=\"item\" @1:6",
        "ui.for original-ref=7 @0:40\n  branch @1:6",
    ] {
        let input = format!("[l2-dump-v2]\nops=1\n[l2-dump-v2.ops]\n{line}\n");
        assert!(Page::parse(&input).is_err(), "accepted {line}");
    }
}

#[test]
fn verifier_checks_each_real_region_under_its_original_for_span() {
    let mut page = diagnostics();
    assert!(verify(&page, Rigor::Canonical).is_empty());
    let Op::OriginalFor(outer) = &mut page.ops[0] else {
        unreachable!();
    };
    let Op::OriginalFor(inner) = &mut outer.ops[0] else {
        unreachable!();
    };
    inner.span = Span::new(30, 50);
    let violations = verify(&page, Rigor::Canonical);
    assert_eq!(violations.len(), 2);
    assert!(
        violations
            .iter()
            .all(|it| it.code == ViolationCode::RegionNesting)
    );
    assert_eq!(violations[0].span, Span::new(30, 50));
    assert_eq!(violations[1].span, Span::new(20, 24));
}
