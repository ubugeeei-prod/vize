use crate::{CrossFileAnalyzer, CrossFileOptions, FileId};
use vize_croquis::{Croquis, EffectGraphSummary};

const FIRST: &str = "<template><section v-if=\"visible\"><p v-for=\"item in items\">{{ item }}</p></section></template>";
const SECOND: &str = "<template><p v-if=\"日本語\">🙂</p>\r\n<p v-else>復元</p></template>";

fn register(analyzer: &mut CrossFileAnalyzer, path: &str, source: &str) -> FileId {
    analyzer.add_file_with_analysis_and_effect_summary(
        path,
        source,
        Croquis::new(),
        EffectGraphSummary::default(),
    )
}

#[test]
fn clear_invalidates_every_per_file_getter_and_component_index() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::all());
    let ids = [
        register(&mut analyzer, "First.vue", FIRST),
        register(&mut analyzer, "Second.vue", SECOND),
    ];
    assert_eq!(analyzer.registry().len(), 2);
    assert_eq!(analyzer.graph().len(), 2);
    assert!(analyzer.graph().find_by_component("First").is_some());
    for id in ids {
        assert!(analyzer.get_analysis(id).is_some());
        assert!(analyzer.get_file_path(id).is_some());
        assert!(analyzer.template_complexity_of(id).is_some());
        assert_eq!(
            analyzer.effect_graph_summary(id),
            Some(EffectGraphSummary::default())
        );
    }

    analyzer.clear();

    assert!(analyzer.registry().is_empty());
    assert!(analyzer.graph().is_empty());
    assert_eq!(analyzer.graph().find_by_component("First"), None);
    assert_eq!(analyzer.graph().find_by_component("Second"), None);
    assert!(analyzer.graph().entries().is_empty());
    assert!(analyzer.graph().circular_dependencies().is_empty());
    assert!(analyzer.template_complexity.is_empty());
    assert!(analyzer.effect_graph_summaries.is_empty());
    for id in ids {
        assert!(analyzer.registry().get(id).is_none());
        assert!(analyzer.get_analysis(id).is_none());
        assert!(analyzer.get_file_path(id).is_none());
        assert_eq!(analyzer.template_complexity_of(id), None);
        assert_eq!(analyzer.effect_graph_summary(id), None);
    }
    assert!(analyzer.analyze().template_complexity.is_empty());
}

#[test]
fn smaller_replacement_project_matches_fresh_analyzer_and_leaves_no_orphan() {
    let mut retained = CrossFileAnalyzer::new(CrossFileOptions::all());
    let first = register(&mut retained, "First.vue", FIRST);
    let removed = register(&mut retained, "Second.vue", SECOND);
    retained.clear();
    let replacement = register(&mut retained, "Replacement.vue", SECOND);
    let mut fresh = CrossFileAnalyzer::new(CrossFileOptions::all());
    let expected = register(&mut fresh, "Replacement.vue", SECOND);

    assert_eq!(replacement, first, "registry epoch reuses the first ID");
    assert_eq!(replacement, expected);
    assert_eq!(
        retained.template_complexity_of(replacement),
        fresh.template_complexity_of(expected)
    );
    assert_eq!(retained.template_complexity_of(removed), None);
    assert_eq!(retained.template_complexity.len(), 1);
    assert_eq!(
        retained.analyze().template_complexity,
        fresh.analyze().template_complexity
    );
}

#[test]
fn raw_module_reusing_an_sfc_id_has_no_template_facts() {
    let mut retained = CrossFileAnalyzer::new(CrossFileOptions::all());
    let previous = register(&mut retained, "First.vue", FIRST);
    retained.clear();
    let replacement = retained.add_file("replacement.ts", "export const value = '日本語🙂'");

    assert_eq!(replacement, previous);
    assert_eq!(retained.template_complexity_of(replacement), None);
    assert!(retained.template_complexity.is_empty());
    assert!(retained.analyze().template_complexity.is_empty());
}

#[test]
fn repeated_clear_and_scriptless_unicode_crlf_reuse_match_a_fresh_epoch() {
    let mut retained = CrossFileAnalyzer::new(CrossFileOptions::all());
    register(&mut retained, "First.vue", FIRST);
    for _ in 0..3 {
        retained.clear();
        retained.clear();
        let id = register(&mut retained, "Scriptless.vue", SECOND);
        let mut fresh = CrossFileAnalyzer::new(CrossFileOptions::all());
        let expected = register(&mut fresh, "Scriptless.vue", SECOND);
        assert_eq!(
            retained.template_complexity_of(id),
            fresh.template_complexity_of(expected)
        );
        assert_eq!(
            retained.analyze().template_complexity,
            fresh.analyze().template_complexity
        );
        assert_eq!(retained.template_complexity.len(), 1);
    }
    retained.clear();
    assert!(retained.template_complexity.is_empty());
}
