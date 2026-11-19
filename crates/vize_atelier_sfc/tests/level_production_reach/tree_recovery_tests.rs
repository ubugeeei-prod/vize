use super::super::*;

#[test]
fn nested_interactive_recoveries_keep_production_dom_parity() {
    let _guard = PROFILER_TEST_LOCK.lock().unwrap();
    // A Vue template keeps nested anchors as authored, as Vue's compiler
    // does, so neither lane reports a tree-construction recovery.
    let cases = [
        "<template><a href='/outer'><a href='/inner'>inner</a></a></template><script setup>const label = 'inner'</script>",
        "<template><a><a>one</a></a><a><a>two</a></a></template><script setup>const label = 'inner'</script>",
    ];

    for source in cases {
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

        let warnings: Vec<_> = selected
            .warnings
            .iter()
            .map(|warning| warning.message.as_str())
            .collect();
        assert_eq!(warnings, Vec::<&str>::new(), "{shape:?}: {source}");

        let legacy = vize_atelier_dom::differential::with_legacy_lane(|| {
            compile(&descriptor, "NestedAnchor.vue", shape)
        });
        assert_eq!(divergence(&selected, &legacy), None, "{shape:?}: {source}");
    }
}
