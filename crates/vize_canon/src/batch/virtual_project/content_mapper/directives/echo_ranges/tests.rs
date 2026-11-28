use super::super::ContentMapperDiagnosticDirective;
use super::{EchoRanges, INDEX_THRESHOLD};

fn directive(start: usize, end: usize) -> ContentMapperDiagnosticDirective {
    ContentMapperDiagnosticDirective([0, 0, start, end, 0, 0])
}

fn overlaps(existing: &[ContentMapperDiagnosticDirective], start: usize, end: usize) -> bool {
    existing.iter().any(|directive| {
        let [_, _, left, right, _, _] = directive.0;
        start < right && left < end
    })
}

#[test]
fn nested_duplicate_touching_and_empty_directives_keep_exact_membership() {
    for existing in [
        vec![directive(0, 2), directive(2, 4)],
        vec![directive(1, 8), directive(2, 3), directive(1, 8)],
        vec![directive(2, 2), directive(4, 4), directive(5, 8)],
        vec![directive(0, 3), directive(2, 5), directive(5, 7)],
    ] {
        let index = EchoRanges::new(&existing, INDEX_THRESHOLD + 1);
        for start in 0..10 {
            for end in start..10 {
                assert_eq!(
                    index.overlaps(&existing, start, end),
                    overlaps(&existing, start, end),
                    "existing={existing:?}, probe={start}..{end}"
                );
            }
        }
    }
}

#[test]
fn arbitrary_admission_order_matches_the_original_strict_overlap_rule() {
    let mut seed = 0x12093_u64;
    for _ in 0..100 {
        let mut next = || {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            (seed >> 32) as usize % 512
        };
        let mut existing = (0..32)
            .map(|_| {
                let start = next();
                directive(start, start + next() % 24)
            })
            .collect::<Vec<_>>();
        let mut index = EchoRanges::new(&existing, INDEX_THRESHOLD + 1);
        for _ in 0..512 {
            let start = next();
            let end = start + next() % 17;
            let expected = overlaps(&existing, start, end);
            assert_eq!(index.overlaps(&existing, start, end), expected);
            if !expected {
                index.insert(start, end);
                existing.push(directive(start, end));
            }
        }
    }
}

#[test]
fn empty_echo_keeps_existing_coverage_at_the_same_start() {
    let mut existing = vec![directive(1, 5)];
    let mut index = EchoRanges::new(&existing, INDEX_THRESHOLD + 1);
    assert!(!index.overlaps(&existing, 1, 1));
    index.insert(1, 1);
    existing.push(directive(1, 1));
    for start in 0..7 {
        for end in start..7 {
            assert_eq!(
                index.overlaps(&existing, start, end),
                overlaps(&existing, start, end),
                "probe={start}..{end}"
            );
        }
    }
}

#[test]
fn small_sfc_path_observes_appended_directives_without_an_index() {
    let mut existing = vec![directive(5, 8)];
    let mut index = EchoRanges::new(&existing, INDEX_THRESHOLD - 1);
    assert!(index.0.is_none());
    assert!(!index.overlaps(&existing, 1, 4));
    index.insert(1, 4);
    existing.push(directive(1, 4));
    assert!(index.overlaps(&existing, 2, 3));
    assert!(!index.overlaps(&existing, 4, 5));
}
