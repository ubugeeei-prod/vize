use super::super::*;

#[test]
fn nested_interactive_recoveries_keep_production_dom_parity() {
    let _guard = PROFILER_TEST_LOCK.lock().unwrap();
    const NESTED_ANCHOR: &str =
        "Nested anchor start tag closed the previous anchor before inserting the new one.";
    const IGNORED_END: &str = "HTML tree construction ignored this end tag because the element was already closed before a nested start tag.";
    let cases = [
        (
            "<template><a href='/outer'><a href='/inner'>inner</a></a></template><script setup>const label = 'inner'</script>",
            1,
        ),
        (
            "<template><a><a>one</a></a><a><a>two</a></a></template><script setup>const label = 'inner'</script>",
            2,
        ),
    ];

    for (source, recoveries) in cases {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let shape = Shape::DomInline;
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        let selected = compile(&descriptor, "NestedAnchor.vue", shape).unwrap();
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();
        assert_eq!(
            classify(shape, &counters),
            Ok(Lane::Accepted),
            "{shape:?}: {source}"
        );

        let expected: Vec<_> = [NESTED_ANCHOR, IGNORED_END]
            .into_iter()
            .cycle()
            .take(recoveries * 2)
            .collect();
        let warnings: Vec<_> = selected
            .warnings
            .iter()
            .map(|warning| warning.message.as_str())
            .collect();
        assert_eq!(warnings, expected, "{shape:?}: {source}");

        let legacy = vize_atelier_dom::differential::with_legacy_lane(|| {
            compile(&descriptor, "NestedAnchor.vue", shape)
        });
        assert_eq!(divergence(&selected, &legacy), None, "{shape:?}: {source}");
    }
}
