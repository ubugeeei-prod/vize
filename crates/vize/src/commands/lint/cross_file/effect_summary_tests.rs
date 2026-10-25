use super::build_cross_file_lint_output_with_report;
use super::sfc::analyze_sfc_for_cross_file;
use std::path::Path;
use vize_croquis::EffectGraphSummary;
use vize_patina::HelpLevel;

const APP: &str = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template><Child /></template>
"#;

const CHILD: &str = r#"<script setup lang="ts">
import { computed, ref } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>
<template><p>{{ doubled }}</p></template>
"#;

// Independently observed from the same authored bytes in published WASM 0.565.0:
// two reactive nodes, one parser-built edge, reactive score 4, total score 6.
const REPORT: &str = "## Cross-file Complexity\n\n\
| Metric | Value |\n\
| --- | ---: |\n\
| Band | low |\n\
| Total score | 6 |\n\
| Cyclomatic score | 2 |\n\
| Cognitive score | 0 |\n\n\
### Dimensions\n\n\
| Dimension | Score |\n\
| --- | ---: |\n\
| template-control-flow | 2 |\n\
| slot-usage | 0 |\n\
| prop-drilling | 0 |\n\
| global-state | 0 |\n\
| provide-inject | 0 |\n\
| fallthrough-attrs | 0 |\n\
| reactive-graph | 4 |\n\n\
### Template Complexity\n\n\
Own: the component's template alone. Rendered: own plus every distinct component it renders, recursion counted once. Own scores above cyclomatic 11 or cognitive 16 exceed the corpus p95.\n\n\
| Component | Own cyclomatic | Own cognitive | Rendered cyclomatic | Rendered cognitive | Renders | Where it comes from |\n\
| --- | ---: | ---: | ---: | ---: | ---: | --- |\n\
| App.vue | 1 | 0 | 2 | 0 | 1 | - |\n\
| Child.vue | 1 | 0 | 1 | 0 | 0 | - |\n\n\
### Top Hotspots\n\n\
| Rank | File | Component | Score | Reason |\n\
| ---: | --- | --- | ---: | --- |\n\
| 1 | Child.vue | Child | 5 | reactive-graph (4) via template cyclomatic=1, reactive nodes=2, reactive edges=1 |\n\
| 2 | App.vue | App | 1 | template-control-flow (1) via template cyclomatic=1 |\n";

const PLAIN_REPORT: &str = "## Cross-file Complexity\n\n\
| Metric | Value |\n\
| --- | ---: |\n\
| Band | low |\n\
| Total score | 2 |\n\
| Cyclomatic score | 2 |\n\
| Cognitive score | 0 |\n\n\
### Dimensions\n\n\
| Dimension | Score |\n\
| --- | ---: |\n\
| template-control-flow | 2 |\n\
| slot-usage | 0 |\n\
| prop-drilling | 0 |\n\
| global-state | 0 |\n\
| provide-inject | 0 |\n\
| fallthrough-attrs | 0 |\n\
| reactive-graph | 0 |\n\n\
### Template Complexity\n\n\
Own: the component's template alone. Rendered: own plus every distinct component it renders, recursion counted once. Own scores above cyclomatic 11 or cognitive 16 exceed the corpus p95.\n\n\
| Component | Own cyclomatic | Own cognitive | Rendered cyclomatic | Rendered cognitive | Renders | Where it comes from |\n\
| --- | ---: | ---: | ---: | ---: | ---: | --- |\n\
| App.vue | 1 | 0 | 2 | 0 | 1 | - |\n\
| Child.vue | 1 | 0 | 1 | 0 | 0 | - |\n\n\
### Top Hotspots\n\n\
| Rank | File | Component | Score | Reason |\n\
| ---: | --- | --- | ---: | --- |\n\
| 1 | App.vue | App | 1 | template-control-flow (1) via template cyclomatic=1 |\n\
| 2 | Child.vue | Child | 1 | template-control-flow (1) via template cyclomatic=1 |\n";

fn effects(source: &str) -> EffectGraphSummary {
    analyze_sfc_for_cross_file(source, Path::new("日本語🙂Child.vue"))
        .expect("valid SFC descriptor")
        .effects
}

#[test]
fn actual_two_file_cli_report_retains_the_complete_effect_graph() {
    for child in [CHILD.into(), CHILD.replace('\n', "\r\n")] {
        let files = [
            ("App.vue".into(), APP),
            ("Child.vue".into(), child.as_str()),
        ];
        let output =
            build_cross_file_lint_output_with_report(&files, HelpLevel::Short, false, true, None);
        assert_eq!(output.complexity_report.as_deref(), Some(REPORT));
        assert_eq!(output.results.len(), 2);
        for (result, expected_file) in output.results.iter().zip(["App.vue", "Child.vue"]) {
            assert_eq!(result.filename, expected_file);
            assert_eq!(result.error_count, 0);
            assert_eq!(result.warning_count, 0);
            assert!(result.diagnostics.is_empty());
        }
    }
}

#[test]
fn block_grammars_and_normal_setup_scope_supply_actual_effect_edges() {
    for source in [
        CHILD,
        r#"<script>
import { ref } from 'vue'
const count = ref(0)
export default {}
</script>
<script setup lang="ts">
import { computed } from 'vue'
const doubled: unknown = computed(() => count.value * 2)
</script>"#,
        r#"<script lang="jsx">
import { computed, ref } from 'vue'
const view = <div>日本語🙂</div>
const count = ref(0)
const doubled = computed(() => count.value * 2)
export default {}
</script>"#,
        r#"<script setup lang="tsx">
import { computed, ref } from 'vue'
const view = <div>日本語🙂</div>
const count = ref<number>(0)
const doubled = computed(() => count.value * 2)
</script>"#,
    ] {
        assert_eq!(
            effects(source),
            EffectGraphSummary {
                node_count: 2,
                edge_count: 1,
                cycle_count: 0,
                cycle_node_count: 0,
            }
        );
    }
}

#[test]
fn cyclic_effect_edges_are_not_lost_to_the_sfc_container() {
    let source = r#"<script setup lang="ts">
import { computed } from 'vue'
const left = computed(() => right.value)
const right = computed(() => left.value)
</script><template><p>{{ left }}</p></template>"#;
    assert_eq!(
        effects(source),
        EffectGraphSummary {
            node_count: 2,
            edge_count: 2,
            cycle_count: 1,
            cycle_node_count: 2,
        }
    );
}

#[test]
fn block_shadowing_retains_independent_binding_owners() {
    let source = r#"<script>
import { computed, ref } from 'vue'
const count = ref(0)
const before = computed(() => count.value)
export default {}
</script>
<script setup>
const count = ref(1)
const after = computed(() => count.value)
</script>"#;
    assert_eq!(
        effects(source),
        EffectGraphSummary {
            node_count: 4,
            edge_count: 2,
            cycle_count: 0,
            cycle_node_count: 0,
        }
    );
}

#[test]
fn edits_deletions_and_repairs_do_not_retain_old_effect_summaries() {
    let plain = r#"<script setup>const label = '日本語🙂'</script>
<template><p>{{ label }}</p></template>"#;
    let template_only = r#"<template><p>ref(0), computed(() => count.value)</p></template>"#;
    let comment_only = r#"<script setup>
// const count = ref(0); const doubled = computed(() => count.value)
const label = 'ref(0), computed(() => count.value)'
</script>"#;
    for source in [plain, template_only, comment_only] {
        assert_eq!(effects(source), EffectGraphSummary::default());
    }
    for child in [CHILD, plain, CHILD, template_only, CHILD] {
        let files = [("App.vue".into(), APP), ("Child.vue".into(), child)];
        let output =
            build_cross_file_lint_output_with_report(&files, HelpLevel::Short, false, true, None);
        if child == CHILD {
            assert_eq!(output.complexity_report.as_deref(), Some(REPORT));
        } else {
            assert_eq!(output.complexity_report.as_deref(), Some(PLAIN_REPORT));
        }
    }
}
