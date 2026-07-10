use super::{CrossFileAnalyzer, CrossFileOptions};
use vize_croquis::{Croquis, EffectGraphSummary};

#[test]
fn explicit_max_effect_summaries_saturate_global_and_hotspot_counts() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let summary = EffectGraphSummary {
        edge_count: usize::MAX,
        cycle_count: usize::MAX,
        ..EffectGraphSummary::default()
    };
    for path in ["A.vue", "B.vue"] {
        analyzer.add_file_with_analysis_and_effect_summary(path, "", Croquis::default(), summary);
    }

    let result = analyzer.analyze();
    assert_eq!(
        result.complexity_report.input.reactive_edge_count,
        usize::MAX
    );
    assert_eq!(
        result.complexity_report.input.reactive_cycle_count,
        usize::MAX
    );
    assert_eq!(result.complexity_hotspots.len(), 2);
    assert!(result.complexity_hotspots.iter().all(|hotspot| {
        hotspot.input.reactive_edge_count == usize::MAX
            && hotspot.input.reactive_cycle_count == usize::MAX
    }));
}

#[test]
fn clear_drops_explicit_effect_summaries_before_file_ids_are_reused() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    analyzer.add_file_with_analysis_and_effect_summary(
        "Before.vue",
        "",
        Croquis::default(),
        EffectGraphSummary {
            edge_count: 7,
            cycle_count: 3,
            ..EffectGraphSummary::default()
        },
    );

    analyzer.clear();
    analyzer.add_file_with_analysis_and_effect_summary(
        "After.vue",
        "",
        Croquis::default(),
        EffectGraphSummary::default(),
    );
    let result = analyzer.analyze();

    assert_eq!(result.complexity_report.input.reactive_edge_count, 0);
    assert_eq!(result.complexity_report.input.reactive_cycle_count, 0);
}

#[test]
fn raw_jsx_and_tsx_modules_use_matching_parser_modes() {
    let source = r#"
import { computed } from 'vue'
const render = () => <strong>ready</strong>
const left = computed(() => right.value)
const right = computed(() => left.value)
"#;
    for path in ["Plain.jsx", "Typed.tsx"] {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
        let file_id = analyzer.add_file(path, source);
        let summary = analyzer.effect_graph_summary(file_id).unwrap();
        let analysis = analyzer.get_analysis(file_id).unwrap();

        assert_eq!(analysis.reactivity.count(), 2, "path={path}");
        assert_eq!(summary.edge_count, 2, "path={path}");
        assert_eq!(summary.cycle_count, 1, "path={path}");
    }
}
