use crate::effect_graph::EffectGraph;
use crate::reactivity::{ReactiveKind, ReactivityTracker};
use crate::script_parser::parse_script_setup;
use vize_carton::CompactString;

#[test]
fn overlay_is_stable_for_sources_losses_and_effects() {
    let mut tracker = ReactivityTracker::new();
    tracker.register(CompactString::new("count"), ReactiveKind::Ref, 10);
    tracker.register(CompactString::new("state"), ReactiveKind::Reactive, 30);
    tracker.register(CompactString::new("doubled"), ReactiveKind::Computed, 60);
    tracker.record_destructure(
        CompactString::new("state"),
        vec![CompactString::new("user")],
        80,
        100,
    );
    tracker.record_ref_value_extract(
        CompactString::new("count"),
        CompactString::new("plainCount"),
        110,
        130,
    );

    let mut graph = EffectGraph::default();
    graph.add_edge("doubled", "count");
    graph.add_edge("count", "doubled");

    let json = serde_json::to_string_pretty(&tracker.overlay_with_effect_graph(&graph)).unwrap();
    insta::assert_snapshot!(json);
}

#[test]
fn overlay_sorts_sources_by_id_and_losses_by_range() {
    let mut tracker = ReactivityTracker::new();
    tracker.register(CompactString::new("later"), ReactiveKind::Reactive, 20);
    tracker.register(CompactString::new("earlier"), ReactiveKind::Ref, 5);
    tracker.record_spread(CompactString::new("later"), 50, 70);
    tracker.record_ref_value_extract(
        CompactString::new("earlier"),
        CompactString::new("value"),
        30,
        40,
    );

    let overlay = tracker.overlay();

    assert_eq!(overlay.sources[0].name.as_str(), "later");
    assert_eq!(overlay.sources[1].name.as_str(), "earlier");
    assert_eq!(overlay.losses[0].kind, "refValueExtract");
    assert_eq!(overlay.losses[1].kind, "reactiveSpread");
}

#[test]
fn parser_overlay_preserves_reactive_declaration_offsets() {
    let source = r#"import { ref as vue_ref } from 'vue'
const greeting = 'こんにちは👋'
const count = vue_ref(0), state = shallowReactive({ count: 0 })
const doubled = computed(() => count.value * 2)
"#;
    let overlay = parse_script_setup(source).reactivity.overlay();

    for source_overlay in &overlay.sources {
        assert_eq!(
            source_overlay.declaration_offset as usize,
            source.find(source_overlay.name.as_str()).unwrap(),
            "overlay offset for `{}`",
            source_overlay.name
        );
    }

    let json = serde_json::to_string_pretty(&overlay).unwrap();
    insta::assert_snapshot!(json);
}
