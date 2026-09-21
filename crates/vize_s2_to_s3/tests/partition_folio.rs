//! TS-16 laws for the partition-fact page (`[s3-partition-folio]`): canonical
//! text is a print/parse fixed point, the live-fact mirror prints exactly,
//! and malformed records are refused with their line.

use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_s0::{Allocator, cstr};
use vize_s2_to_s3::{FolioPartitionFact, PartitionKind, S3PartitionFolio, lower};

const CANONICAL: &str = "\
[s3-partition-folio]

[s3-partition-folio.ops]
op=0 kind=static span=0:88
op=1 kind=static span=20:81
op=2 kind=dynamic span=28:46
op=3 kind=dynamic span=47:60
op=4 kind=dynamic span=61:72

";

#[test]
fn full_print_is_identity_on_canonical_text() {
    let folio = S3PartitionFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(
        folio.print_to_string(FolioMode::Display),
        folio.print_to_string(FolioMode::Full)
    );
}

#[test]
fn the_live_mirror_prints_the_lowered_facts_exactly() {
    let allocator = Allocator::default();
    let source = r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#;
    let (tree, errors) = vize_s1::parse(&allocator, source);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let lowered = lower(&allocator, &s2.root);

    let folio = S3PartitionFolio::of(&lowered.partition);
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(
        S3PartitionFolio::parse(CANONICAL).expect("canonical text parses"),
        folio
    );
}

#[test]
fn structural_round_trip_holds_for_both_kinds() {
    let folio = S3PartitionFolio {
        ops: vec![
            FolioPartitionFact {
                op: 7,
                kind: PartitionKind::Dynamic,
                span: vize_s0::Span::new(3, 9),
            },
            FolioPartitionFact {
                op: 8,
                kind: PartitionKind::Static,
                span: vize_s0::Span::new(9, 9),
            },
        ],
    };
    let printed = folio.print_to_string(FolioMode::Full);
    assert_eq!(S3PartitionFolio::parse(printed.as_str()), Ok(folio));
}

#[test]
fn malformed_records_are_refused_with_their_line() {
    let unknown_kind =
        "[s3-partition-folio]\n\n[s3-partition-folio.ops]\nop=0 kind=hoisted span=0:1\n\n";
    assert_eq!(
        S3PartitionFolio::parse(unknown_kind).unwrap_err(),
        FolioError::new(4, cstr!("unknown partition kind `hoisted`"))
    );
    let bad_span = "[s3-partition-folio]\n\n[s3-partition-folio.ops]\nop=0 kind=static span=01\n\n";
    assert_eq!(
        S3PartitionFolio::parse(bad_span).unwrap_err(),
        FolioError::new(4, cstr!("invalid span `01`"))
    );
    let extra =
        "[s3-partition-folio]\n\n[s3-partition-folio.ops]\nop=0 kind=static span=0:1 x=1\n\n";
    assert_eq!(
        S3PartitionFolio::parse(extra).unwrap_err(),
        FolioError::new(4, cstr!("unexpected field `x=1`"))
    );
}

#[test]
fn the_kind_spelling_round_trips() {
    for kind in [PartitionKind::Static, PartitionKind::Dynamic] {
        assert_eq!(PartitionKind::from_str(kind.as_str()), Some(kind));
    }
    assert_eq!(PartitionKind::from_str("Static"), None);
}
