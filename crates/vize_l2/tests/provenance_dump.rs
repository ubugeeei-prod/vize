//! TS-16 laws for the provenance page (`[l2-provenance-dump-v2]`): canonical
//! text is a print/parse fixed point (escapes included), structural
//! round-trip holds for produced and dropped records, and malformed lines
//! are refused with their line number.

use vize_davinci::dump::{Dump, Error as DumpError, Mode as DumpMode};
use vize_l0::{Span, String, cstr};
use vize_l2::dump::provenance::{Page as ProvenancePage, Record as DumpProvenance};

const CANONICAL: &str = r#"[l2-provenance-dump-v2]

[l2-provenance-dump-v2.records]
rule=lower.element node=0 before="<p class=\"x\">" after="ui.element p" @3:18
rule=drop.comment node=- before="<!-- a\nb -->" after="" @18:31
rule=pass.hoist-static.fact node=0 before="<p>" after="fact level=not-static" @3:18

"#;

fn record(
    rule: &str,
    node: Option<u32>,
    before: &str,
    after: &str,
    span: (u32, u32),
) -> DumpProvenance {
    DumpProvenance {
        rule: String::from(rule),
        node,
        before: String::from(before),
        after: String::from(after),
        span: Span::new(span.0, span.1),
    }
}

#[test]
fn full_print_is_identity_on_canonical_text() {
    let folio = ProvenancePage::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(folio.print_to_string(DumpMode::Full).as_str(), CANONICAL);
    assert_eq!(
        folio.records,
        vec![
            record(
                "lower.element",
                Some(0),
                "<p class=\"x\">",
                "ui.element p",
                (3, 18)
            ),
            record("drop.comment", None, "<!-- a\nb -->", "", (18, 31)),
            record(
                "pass.hoist-static.fact",
                Some(0),
                "<p>",
                "fact level=not-static",
                (3, 18)
            ),
        ]
    );
}

#[test]
fn structural_round_trip_keeps_quotes_backslashes_and_tabs() {
    let folio = ProvenancePage {
        records: vec![record(
            "lower.text",
            Some(7),
            "a\\\"b\tc",
            "ui.text",
            (0, 6),
        )],
    };
    let printed = folio.print_to_string(DumpMode::Full);
    assert_eq!(ProvenancePage::parse(printed.as_str()), Ok(folio));
}

#[test]
fn malformed_records_are_refused_with_their_line() {
    let page = |line: &str| {
        cstr!("[l2-provenance-dump-v2]\n\n[l2-provenance-dump-v2.records]\n{line}\n\n")
    };
    let cases = [
        ("node=0 before=\"\" after=\"\" @0:1", "expected `rule=`"),
        (
            "rule= node=0 before=\"\" after=\"\" @0:1",
            "expected `rule=<name> `",
        ),
        (
            "rule=r node=x before=\"\" after=\"\" @0:1",
            "invalid node id `x`",
        ),
        (
            "rule=r node=0 before=\"a after=\"\" @0:1",
            "expected ` after=`",
        ),
        ("rule=r node=0 before=\"\" after=\"\"", "missing span"),
    ];
    for (line, message) in cases {
        assert_eq!(
            ProvenancePage::parse(page(line).as_str()).unwrap_err(),
            DumpError::new(4, cstr!("{message}")),
            "{line}"
        );
    }
}
