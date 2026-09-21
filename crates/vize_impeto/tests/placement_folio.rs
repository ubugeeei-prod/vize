//! TS-16 laws for the S3 placement page.

use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_impeto::op::OpId;
use vize_impeto::placement::{
    FolioPlacement, Placement, PlacementRecord, PlacementSet, S3PlacementFolio,
};
use vize_s0::cstr;

const CANONICAL: &str = "\
[s3-placement-folio]

[s3-placement-folio.placements]
op=1 alternatives=inline,cache leader=- chosen=cache
op=3 alternatives=inline,group leader=2 chosen=inline
op=5 alternatives=inline,hoist leader=- chosen=hoist

";

fn folio() -> S3PlacementFolio {
    let row = |op, alternative, leader: Option<u32>, chosen| {
        FolioPlacement(
            PlacementRecord::new(
                OpId::new(op),
                PlacementSet::INLINE.with(alternative),
                leader.map(OpId::new),
            )
            .with_chosen(chosen),
        )
    };
    S3PlacementFolio {
        placements: vec![
            row(1, Placement::Cache, None, Placement::Cache),
            row(3, Placement::Group, Some(2), Placement::Inline),
            row(5, Placement::Hoist, None, Placement::Hoist),
        ],
    }
}

#[test]
fn full_print_is_identity_on_canonical_text() {
    let parsed = S3PlacementFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(parsed.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(parsed, folio());
}

#[test]
fn parse_print_is_structural_identity() {
    let printed = folio().print_to_string(FolioMode::Full);
    assert_eq!(printed.as_str(), CANONICAL);
    assert_eq!(S3PlacementFolio::parse(printed.as_str()), Ok(folio()));
    assert_eq!(
        folio().print_to_string(FolioMode::Display),
        folio().print_to_string(FolioMode::Full)
    );
}

#[test]
fn every_set_round_trips_including_the_empty_one() {
    for bits in 0..16u8 {
        let set = PlacementSet::from_bits(bits).expect("known bits");
        let page = S3PlacementFolio {
            placements: vec![FolioPlacement(PlacementRecord::new(
                OpId::new(bits.into()),
                set,
                None,
            ))],
        };
        let printed = page.print_to_string(FolioMode::Full);
        assert_eq!(S3PlacementFolio::parse(printed.as_str()), Ok(page));
    }
    assert_eq!(PlacementSet::from_bits(16), None);
}

#[test]
fn parse_rejects_malformed_rows_exactly() {
    let page =
        |row: &str| cstr!("[s3-placement-folio]\n\n[s3-placement-folio.placements]\n{row}\n");
    let cases = [
        (
            "op=1 alternatives=cache,inline leader=- chosen=inline",
            "placement alternatives `cache,inline` are not in canonical order",
        ),
        (
            "op=1 alternatives=inline,inline leader=- chosen=inline",
            "placement alternatives `inline,inline` are not in canonical order",
        ),
        (
            "op=1 alternatives=inline,spill leader=- chosen=inline",
            "unknown placement `spill`",
        ),
        (
            "op=1 alternatives=inline leader=x chosen=inline",
            "invalid `leader` integer `x`",
        ),
        (
            "op=1 alternatives=inline leader=-",
            "missing `chosen` field",
        ),
        (
            "op=1 alternatives=inline chosen=inline leader=-",
            "expected `leader=...`, got `chosen=inline`",
        ),
        (
            "op=1 alternatives=inline leader=- chosen=inline extra=1",
            "unexpected field `extra=1`",
        ),
    ];
    for (row, message) in cases {
        assert_eq!(
            S3PlacementFolio::parse(page(row).as_str()),
            Err(FolioError::new(4, cstr!("{message}"))),
            "{row}"
        );
    }
}
