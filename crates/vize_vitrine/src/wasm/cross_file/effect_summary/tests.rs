use super::from_descriptor;
use vize_atelier_sfc::parse_sfc;
use vize_croquis::EffectGraphSummary;
use vize_l0::cstr;

fn summary(source: &str) -> EffectGraphSummary {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    from_descriptor(&descriptor)
}

const EDGE: EffectGraphSummary = EffectGraphSummary {
    node_count: 2,
    edge_count: 1,
    cycle_count: 0,
    cycle_node_count: 0,
};

#[test]
fn original_js_jsx_ts_tsx_normal_and_setup_grammars_preserve_edges() {
    for lang in ["js", "jsx", "ts", "tsx"] {
        for block in ["script", "script setup"] {
            let view = if matches!(lang, "jsx" | "tsx") {
                "const view = <section>日本語🙂</section>\n"
            } else {
                ""
            };
            let annotation = if matches!(lang, "ts" | "tsx") {
                "<number>"
            } else {
                ""
            };
            let source = cstr!(
                "<{block} lang=\"{lang}\">\nimport {{ ref, computed }} from 'vue'\n{view}const count = ref{annotation}(0)\nconst doubled = computed(() => count.value * 2)\n</script>"
            );
            assert_eq!(summary(&source), EDGE, "{lang}: {block}");
        }
    }
}

#[test]
fn normal_bindings_remain_visible_to_setup_instead_of_concatenating_programs() {
    let source = r#"<script lang="jsx">
import { ref } from 'vue'
const count = ref(0)
const view = <section />
export default {}
</script>
<script setup lang="ts">
import { computed } from 'vue'
const doubled = computed(() => count.value * 2)
</script>"#;
    assert_eq!(summary(source), EDGE);
    assert_eq!(summary(&source.replace('\n', "\r\n")), EDGE);
}

#[test]
fn shadowed_normal_and_setup_names_keep_independent_effect_owners() {
    let source = r#"<script>
import { ref, computed } from 'vue'
const count = ref(0)
const before = computed(() => count.value)
export default {}
</script>
<script setup>
const count = ref(1)
const after = computed(() => count.value)
</script>"#;
    assert_eq!(
        summary(source),
        EffectGraphSummary {
            node_count: 4,
            edge_count: 2,
            cycle_count: 0,
            cycle_node_count: 0,
        }
    );
}

#[test]
fn cycles_and_true_vue_aliases_are_parser_driven() {
    let source = r#"<script setup lang="tsx">
import { computed as derive } from 'vue'
const view = <section />
const left = derive(() => right.value)
const right = derive(() => left.value)
</script>"#;
    assert_eq!(
        summary(source),
        EffectGraphSummary {
            node_count: 2,
            edge_count: 2,
            cycle_count: 1,
            cycle_node_count: 2,
        }
    );
}

#[test]
fn comments_strings_template_text_and_removed_scripts_supply_no_effects() {
    for source in [
        "<template>ref(0), computed(() => count.value)</template>",
        "<script setup>// ref(0), computed(() => count.value)\nconst text = 'ref(0)'</script>",
        "<script setup lang=\"tsx\">const view = <p>computed(() => count.value)</p></script>",
    ] {
        assert_eq!(summary(source), EffectGraphSummary::default());
    }
}

#[test]
fn javascript_comparisons_are_not_typescript_generic_vue_calls() {
    let source = r#"<script setup lang="js">
import { ref, computed } from 'vue'
const count = ref < number > (0)
const doubled = computed(() => count.value)
</script>"#;
    // Summary nodes are edge endpoints, not every reactive declaration.
    assert_eq!(summary(source), EffectGraphSummary::default());
}
