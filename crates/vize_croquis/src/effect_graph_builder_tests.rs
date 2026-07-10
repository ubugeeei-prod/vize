use crate::effect_graph::{
    EffectGraph, EffectGraphScript, EffectGraphSummary, build_effect_graph_from_script_setup,
    build_effect_graph_from_sfc_scripts,
};
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
fn parser_built_summary_counts_self_loop_and_disconnected_cycles() {
    let source = r#"
import { computed } from 'vue'
const selfCycle = computed(() => selfCycle.value)
const leftA = computed(() => rightA.value)
const rightA = computed(() => leftA.value)
const leftB = computed(() => rightB.value)
const rightB = computed(() => leftB.value)
"#;
    let graph =
        build_effect_graph_from_sfc_scripts(None, Some(EffectGraphScript::new(source, Some("ts"))));

    assert_eq!(
        edge_pairs(&graph),
        [
            (cstr!("setup:leftA"), cstr!("setup:rightA")),
            (cstr!("setup:leftB"), cstr!("setup:rightB")),
            (cstr!("setup:rightA"), cstr!("setup:leftA")),
            (cstr!("setup:rightB"), cstr!("setup:leftB")),
            (cstr!("setup:selfCycle"), cstr!("setup:selfCycle")),
        ]
    );
    assert_eq!(
        graph.summary(),
        EffectGraphSummary {
            node_count: 5,
            edge_count: 5,
            cycle_count: 3,
            cycle_node_count: 5,
        }
    );
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

#[test]
fn sfc_source_languages_preserve_effect_cycles() {
    for (lang, jsx) in [
        (None, ""),
        (Some("js"), ""),
        (Some("ts"), "const typed: number = 1"),
        (Some("jsx"), "const render = () => <strong>ready</strong>"),
        (
            Some("tsx"),
            "const render = (): JSX.Element => <strong>ready</strong>",
        ),
    ] {
        let source = format!(
            r#"
import {{ computed }} from 'vue'
{jsx}
const left = computed(() => right.value)
const right = computed(() => left.value)
"#
        );
        let graph = build_effect_graph_from_sfc_scripts(
            None,
            Some(EffectGraphScript::new(source.as_str(), lang)),
        );

        assert_eq!(
            graph.summary(),
            EffectGraphSummary {
                node_count: 2,
                edge_count: 2,
                cycle_count: 1,
                cycle_node_count: 2,
            },
            "lang={lang:?}"
        );
    }
}

#[test]
fn split_sfc_scripts_share_outer_sources_and_keep_scoped_cycles_distinct() {
    let script = r#"
import { computed, ref } from 'vue'
export const shared = ref(0)
const left = computed(() => right.value)
const right = computed(() => left.value)
"#;
    let setup = r#"
import { computed, watch } from 'vue'
watch(shared, () => console.log(shared.value))
const left = computed(() => right.value)
const right = computed(() => left.value)
"#;
    let graph = build_effect_graph_from_sfc_scripts(
        Some(EffectGraphScript::new(script, Some("ts"))),
        Some(EffectGraphScript::new(setup, Some("ts"))),
    );

    assert_eq!(
        graph.summary(),
        EffectGraphSummary {
            node_count: 6,
            edge_count: 5,
            cycle_count: 2,
            cycle_node_count: 4,
        }
    );
    assert!(
        graph
            .edges()
            .any(|edge| edge.from.starts_with("setup:watch@") && edge.to == "script:shared")
    );
}

#[test]
fn malformed_sfc_block_does_not_discard_a_valid_sibling_summary() {
    let script = r#"
import { computed } from 'vue'
const left = computed(() => right.value)
const right = computed(() => left.value)
"#;
    let malformed_setup = r#"
import { computed } from 'vue'
const setupLeft = computed(() => setupRight.value)
const setupRight = computed(() => setupLeft.value)
const broken = (
"#;
    let graph = build_effect_graph_from_sfc_scripts(
        Some(EffectGraphScript::new(script, Some("js"))),
        Some(EffectGraphScript::new(malformed_setup, Some("js"))),
    );

    assert_eq!(
        graph.summary(),
        EffectGraphSummary {
            node_count: 2,
            edge_count: 2,
            cycle_count: 1,
            cycle_node_count: 2,
        }
    );
}

#[test]
fn setup_declarations_shadow_normal_script_reactive_sources() {
    let script = r#"
import { ref } from 'vue'
const shared = ref(0)
"#;
    let setup = r#"
import { watch } from 'vue'
function shared() { return 1 }
watch(shared, () => {})
"#;
    let graph = build_effect_graph_from_sfc_scripts(
        Some(EffectGraphScript::new(script, Some("js"))),
        Some(EffectGraphScript::new(setup, Some("js"))),
    );

    assert_eq!(graph.summary(), EffectGraphSummary::default());
}

#[test]
fn setup_local_bindings_block_inherited_api_names_without_leaking_other_aliases() {
    let script = r#"
import {
  ref as vueRef,
  computed,
  watch as observe,
  watchEffect as fx,
} from 'vue'
export const outer = vueRef(0)
"#;
    let setup = r#"
const ref = (value) => ({ value })
const computed = (getter) => getter()
const observe = (source, callback) => callback(source)
const fakeSource = ref(1)
const fakeComputed = computed(() => outer.value)
observe(outer, () => {})
fx(() => fakeSource.value)
fx(() => outer.value)
"#;
    let graph = build_effect_graph_from_sfc_scripts(
        Some(EffectGraphScript::new(script, Some("js"))),
        Some(EffectGraphScript::new(setup, Some("js"))),
    );
    let fx_offset = setup.find("fx(() => outer.value)").unwrap();

    assert_eq!(
        edge_pairs(&graph),
        [(
            cstr!("setup:watchEffect@{fx_offset}"),
            cstr!("script:outer"),
        )]
    );
    assert_eq!(
        graph.summary(),
        EffectGraphSummary {
            node_count: 2,
            edge_count: 1,
            cycle_count: 0,
            cycle_node_count: 0,
        }
    );
}
