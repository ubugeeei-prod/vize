use super::*;
use vize_l0::{
    diag::{PartKind, WitnessChain, WitnessKey},
    pass::AnalysisId,
    stage::Stage,
};
use vize_l1_to_l2::native::NativeHoleKind;

#[test]
fn exact_primary_related_and_witness_cursors_borrow_the_original_rows() {
    let text = "a🌸\r\nz";
    let holes = [NativeHole {
        span: Span::new(1, 5),
        kind: NativeHoleKind::MissingMarkup,
    }];
    let link = WitnessLink::new(AnalysisId::new(7), Span::new(8, 8), WitnessKey::Index(1));
    let diagnostics = [Diagnostic::proven(
        Stage::Surface,
        Span::new(0, 8),
        "unclosed",
        WitnessChain::new(link),
    )
    .with_part(DiagnosticPart::new(
        PartKind::Secondary,
        Span::new(5, 7),
        "line break",
    ))];
    let recovery = TemplateRecovery {
        holes: &holes,
        diagnostics: &diagnostics,
        text,
        start: 100,
    };
    let offsets = [100, 101, 101, 105, 106, 107, 108, 108];
    let mut rows = Vec::new();
    let result = recovery.visit_positions(&offsets, |row| {
        let expected_positions = match row.record {
            RecoveryRecord::Hole(h) => {
                assert!(std::ptr::eq(h, &holes[0]));
                &offsets[1..3]
            }
            RecoveryRecord::Diagnostic(d) => {
                assert!(std::ptr::eq(d, &diagnostics[0]));
                &offsets[..6]
            }
            RecoveryRecord::Part { diagnostic, part } => {
                assert!(std::ptr::eq(diagnostic, &diagnostics[0]));
                assert!(std::ptr::eq(part, &diagnostic.parts[0]));
                &offsets[3..5]
            }
            RecoveryRecord::Witness { diagnostic, link } => {
                assert!(std::ptr::eq(diagnostic, &diagnostics[0]));
                assert!(std::ptr::eq(
                    link,
                    diagnostic.witness_chain().unwrap().first()
                ));
                &offsets[6..]
            }
        };
        assert!(std::ptr::eq(&offsets[row.positions], expected_positions));
        rows.push((row.authored, row.text));
        ControlFlow::<()>::Continue(())
    });
    assert_eq!(result, Ok(ControlFlow::Continue(())));
    assert_eq!(
        rows,
        [
            (Span::new(101, 105), "🌸"),
            (Span::new(100, 108), text),
            (Span::new(105, 107), "\r\n"),
            (Span::new(108, 108), "")
        ]
    );
    for (input, expected) in [
        (&[100, 103][..], RecoveryPositionError::OutsideSource(103)),
        (&[100, 109][..], RecoveryPositionError::OutsideSource(109)),
        (&[99][..], RecoveryPositionError::OutsideSource(99)),
        (
            &[107, 100][..],
            RecoveryPositionError::OutOfOrder {
                previous: 107,
                next: 100,
            },
        ),
    ] {
        assert_eq!(
            recovery.visit_positions(input, |_| -> ControlFlow<()> {
                panic!("invalid input reached visitor")
            }),
            Err(expected)
        );
    }
    assert_eq!(
        recovery.visit_positions(&[], |_| -> ControlFlow<()> { panic!("empty query") }),
        Ok(ControlFlow::Continue(()))
    );
    assert_eq!(
        recovery.visit_positions(&offsets, |row| ControlFlow::Break(row.authored)),
        Ok(ControlFlow::Break(Span::new(101, 105)))
    );
}

#[test]
fn malformed_retained_spans_report_the_original_range() {
    let span = Span::new(0, 2);
    let hole = NativeHole {
        span,
        kind: NativeHoleKind::MissingMarkup,
    };
    let recovery = TemplateRecovery {
        text: "x",
        start: 3,
        holes: core::slice::from_ref(&hole),
        diagnostics: &[],
    };
    assert_eq!(
        recovery.visit_positions(&[3], |_| -> ControlFlow<()> {
            panic!("invalid producer span")
        }),
        Err(RecoveryPositionError::InvalidRecord(span))
    );
}

#[test]
fn binary_windows_match_direct_containment_at_full_width_endpoints() {
    let offsets = [0, 0, 1, 3, 3, 5, u32::MAX - 1, u32::MAX, u32::MAX];
    for lo in 0..=offsets.len() {
        for hi in lo..=offsets.len() {
            let points = &offsets[lo..hi];
            for start in offsets {
                for end in offsets.into_iter().filter(|end| *end >= start) {
                    let expected: Vec<_> = points
                        .iter()
                        .copied()
                        .filter(|&p| (start <= p && p < end) || (start == end && p == start))
                        .collect();
                    assert_eq!(points[matching_positions(points, start, end)], expected);
                }
            }
        }
    }
}
