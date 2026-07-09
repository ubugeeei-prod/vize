use crate::effect_graph::{EffectGraph, build_effect_graph_from_script_setup};
use crate::script_parser::parse_script_setup;
use vize_carton::{CompactString, cstr};

fn edge_pairs(graph: &EffectGraph) -> Vec<(CompactString, CompactString)> {
    let mut pairs: Vec<_> = graph
        .edges()
        .map(|edge| (edge.from.clone(), edge.to.clone()))
        .collect();
    pairs.sort();
    pairs
}

#[test]
fn builds_computed_dependency_edges_from_script_setup() {
    let graph = build_effect_graph_from_script_setup(
        r#"
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
const tripled = computed(() => doubled.value + count.value)
"#,
    );

    assert_eq!(
        edge_pairs(&graph),
        [
            (cstr!("doubled"), cstr!("count")),
            (cstr!("tripled"), cstr!("count")),
            (cstr!("tripled"), cstr!("doubled")),
        ]
    );
    assert_eq!(graph.summary().node_count, 3);
    assert_eq!(graph.summary().edge_count, 3);
    assert_eq!(graph.summary().cycle_count, 0);
}

#[test]
fn builds_watch_and_watcheffect_edges_with_vue_import_aliases() {
    let graph = build_effect_graph_from_script_setup(
        r#"
import { ref as r, computed as c, watch as observe, watchEffect as fx } from 'vue'
const count = r(0)
const total = c(() => count.value + 1)
observe([count, () => total.value], () => {})
fx(() => {
  console.log(total.value)
})
"#,
    );
    let pairs = edge_pairs(&graph);

    assert!(pairs.contains(&(cstr!("total"), cstr!("count"))));
    assert!(
        pairs
            .iter()
            .any(|(from, to)| from.starts_with("watch@") && to.as_str() == "count")
    );
    assert!(
        pairs
            .iter()
            .any(|(from, to)| from.starts_with("watch@") && to.as_str() == "total")
    );
    assert!(
        pairs
            .iter()
            .any(|(from, to)| from.starts_with("watchEffect@") && to.as_str() == "total")
    );
    assert_eq!(graph.summary().edge_count, 4);
}

#[test]
fn detects_cycles_in_parser_built_computed_chains() {
    let graph = build_effect_graph_from_script_setup(
        r#"
import { computed } from 'vue'
const left = computed(() => right.value)
const right = computed(() => left.value)
"#,
    );

    let cycle = graph.find_cycle().expect("computed chain should cycle");
    assert_eq!(graph.summary().cycle_count, 1);
    assert!(cycle.contains(&"left".into()));
    assert!(cycle.contains(&"right".into()));
}

#[test]
fn ignores_shadowed_callback_parameters_when_collecting_deps() {
    let graph = build_effect_graph_from_script_setup(
        r#"
import { ref, computed } from 'vue'
const count = ref(0)
const source = ref(1)
const doubled = computed((count) => count + source.value)
"#,
    );

    assert_eq!(edge_pairs(&graph), [(cstr!("doubled"), cstr!("source"))]);
}

#[test]
fn overlay_snapshot_includes_parser_built_effect_graph() {
    let source = r#"
import { ref, computed, watchEffect } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
watchEffect(() => {
  console.log(doubled.value)
})
"#;
    let parsed = parse_script_setup(source);
    let graph = build_effect_graph_from_script_setup(source);
    let json =
        serde_json::to_string_pretty(&parsed.reactivity.overlay_with_effect_graph(&graph)).unwrap();

    insta::assert_snapshot!(json);
}
