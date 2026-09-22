#[test]
fn named_colour_scanning_has_linear_work_at_1k_through_8k() {
    let mut previous_steps = 0;
    for count in [1_000, 2_000, 4_000, 8_000] {
        let mut source = String::from("--colours: ");
        source.reserve(count * 4);
        for _ in 0..count {
            source.push_str("red ");
        }
        let source_len = source.len();
        let (found, steps, rgb_probes) =
            super::super::scan::colors_in_with_metrics(&source, (0, source_len), true);
        assert_eq!(found.len(), count);
        assert_eq!(
            rgb_probes, 0,
            "named tokens must never probe the rgb parser"
        );
        assert!(
            steps <= source_len,
            "scanner revisited input at {count} tokens"
        );
        if previous_steps > 0 {
            assert!(
                steps <= previous_steps * 2 + 16,
                "scanner work grew faster than input at {count} tokens"
            );
        }
        previous_steps = steps;
    }
}

#[test]
fn declaration_context_scanning_counts_internal_work_linearly() {
    let mut previous_work = 0;
    for count in [1_000, 2_000, 4_000, 8_000] {
        let source = format!(
            ".a {{ /*{}*/ --chain: {}red; }}",
            "x".repeat(count),
            "a:".repeat(count)
        );
        let source_len = source.len();
        let (found, work, rgb_probes) =
            super::super::scan::colors_in_with_metrics(&source, (0, source_len), false);
        assert_eq!(found.len(), 1);
        assert_eq!(rgb_probes, 0);
        assert!(work <= source_len * 2, "too much work at {count}: {work}");
        if previous_work > 0 {
            assert!(
                work <= previous_work * 2 + 32,
                "internal work grew faster than input at {count}: {work}"
            );
        }
        previous_work = work;
    }
}
