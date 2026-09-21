//! TS-16 laws for the S3 extraction page.

use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_impeto::extract::{
    Decision, DecisionKind, Delta, FolioDecision, Metric, Reason, S3ExtractionFolio,
};
use vize_impeto::op::OpId;
use vize_impeto::placement::Placement;
use vize_s0::{Span, String, cstr};

const CANONICAL: &str = "\
[s3-extraction-folio]
tier=O1
candidate_budget=8
budget_left=5
emitted_size_before=192
emitted_size_after=158
reactive_edges_before=5
reactive_edges_after=4
update_path_before=7
update_path_after=6

[s3-extraction-folio.decisions]
op=1 placement=cache kind=missed reason=regressed-emitted-size span=5:15 size=6 edges=-1 path=-1 budget=7
op=3 placement=group kind=applied reason=committed span=30:40 size=-21 edges=-1 path=0 budget=6
op=5 placement=hoist kind=applied reason=committed span=45:75 size=-13 edges=0 path=-1 budget=5

";

fn row(
    op: u32,
    placement: Placement,
    reason: Reason,
    span: (u32, u32),
    delta: [i64; 3],
    budget: u32,
) -> FolioDecision {
    FolioDecision(Decision {
        op: OpId::new(op),
        span: Span::new(span.0, span.1),
        placement,
        kind: if reason == Reason::Committed {
            DecisionKind::Applied
        } else {
            DecisionKind::Missed
        },
        reason,
        delta: Delta {
            emitted_size: delta[0],
            reactive_edges: delta[1],
            update_path: delta[2],
        },
        budget_left: budget,
    })
}

fn page_value() -> S3ExtractionFolio {
    S3ExtractionFolio {
        tier: String::from("O1"),
        candidate_budget: 8,
        budget_left: 5,
        emitted_size_before: 192,
        emitted_size_after: 158,
        reactive_edges_before: 5,
        reactive_edges_after: 4,
        update_path_before: 7,
        update_path_after: 6,
        decisions: vec![
            row(
                1,
                Placement::Cache,
                Reason::Regressed(Metric::EmittedSize),
                (5, 15),
                [6, -1, -1],
                7,
            ),
            row(
                3,
                Placement::Group,
                Reason::Committed,
                (30, 40),
                [-21, -1, 0],
                6,
            ),
            row(
                5,
                Placement::Hoist,
                Reason::Committed,
                (45, 75),
                [-13, 0, -1],
                5,
            ),
        ],
    }
}

#[test]
fn full_print_is_identity_on_canonical_text() {
    let parsed = S3ExtractionFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(parsed.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(parsed, page_value());
}

#[test]
fn parse_print_is_structural_identity_for_every_reason() {
    let mut value = page_value();
    value.decisions = Reason::ALL
        .into_iter()
        .enumerate()
        .map(|(index, reason)| {
            let index = index as u32;
            row(
                index,
                Placement::Group,
                reason,
                (index, index + 1),
                [-1, 0, 1],
                index,
            )
        })
        .collect();
    let printed = value.print_to_string(FolioMode::Full);
    assert_eq!(S3ExtractionFolio::parse(printed.as_str()), Ok(value));
}

#[test]
fn parse_rejects_malformed_rows_exactly() {
    let (header, _) = CANONICAL
        .split_once("[s3-extraction-folio.decisions]")
        .expect("canonical page has a decisions section");
    let page = |row: &str| cstr!("{header}[s3-extraction-folio.decisions]\n{row}\n");
    let valid =
        "op=1 placement=cache kind=missed reason=subsumed span=1:2 size=0 edges=0 path=0 budget=0";
    assert_eq!(
        S3ExtractionFolio::parse(page(valid).as_str()),
        Ok(S3ExtractionFolio {
            decisions: vec![row(
                1,
                Placement::Cache,
                Reason::Subsumed,
                (1, 2),
                [0; 3],
                0
            )],
            ..page_value()
        })
    );
    let cases = [
        (
            "op=1 placement=cache kind=applied reason=subsumed span=1:2 size=0 edges=0 path=0 budget=0",
            "decision kind `applied` contradicts reason `subsumed`",
        ),
        (
            "op=1 placement=cache kind=missed reason=committed span=1:2 size=0 edges=0 path=0 budget=0",
            "decision kind `missed` contradicts reason `committed`",
        ),
        (
            "op=1 placement=cache kind=skipped reason=subsumed span=1:2 size=0 edges=0 path=0 budget=0",
            "unknown decision kind `skipped`",
        ),
        (
            "op=1 placement=cache kind=missed reason=slow span=1:2 size=0 edges=0 path=0 budget=0",
            "unknown decision reason `slow`",
        ),
        (
            "op=1 placement=cache kind=missed reason=subsumed span=12 size=0 edges=0 path=0 budget=0",
            "invalid span `12`",
        ),
        (
            "op=1 placement=cache kind=missed reason=subsumed span=1:2 size=x edges=0 path=0 budget=0",
            "invalid `size` integer `x`",
        ),
        (
            "op=1 placement=cache kind=missed reason=subsumed span=1:2 size=0 edges=0 path=0 budget=0 x=1",
            "unexpected field `x=1`",
        ),
    ];
    for (row, message) in cases {
        assert_eq!(
            S3ExtractionFolio::parse(page(row).as_str()),
            Err(FolioError::new(13, cstr!("{message}"))),
            "{row}"
        );
    }
}
