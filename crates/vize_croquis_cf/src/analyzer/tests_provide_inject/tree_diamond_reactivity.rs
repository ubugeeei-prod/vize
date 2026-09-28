use super::*;
use vize_carton::cstr;

#[test]
fn deep_diamond_reactivity_resolves_shared_ancestor_once() {
    const DEPTH: usize = 20;
    let mut analyzer = CrossFileAnalyzer::new(
        CrossFileOptions::minimal()
            .with_provide_inject(true)
            .with_reactivity_tracking(true),
    );
    analyzer.add_file_with_analysis(
        Path::new("Provider.vue"),
        "",
        script_analysis(
            "import { provide, reactive } from 'vue'; const state = reactive({ count: 1 }); provide('state', state)",
            &["A0", "B0"],
        ),
    );
    for level in 0..=DEPTH {
        let next_a = cstr!("A{}", level + 1);
        let next_b = cstr!("B{}", level + 1);
        let children = if level == DEPTH {
            vec![]
        } else {
            vec![next_a.as_str(), next_b.as_str()]
        };
        for prefix in ["A", "B"] {
            let name = cstr!("{prefix}{level}.vue");
            let script = if level == DEPTH {
                "import { inject } from 'vue'; const { count } = inject('state') as { count: number }"
            } else {
                "// pass through"
            };
            analyzer.add_file_with_analysis(
                Path::new(name.as_str()),
                "",
                script_analysis(script, &children),
            );
        }
    }
    analyzer.rebuild_component_edges();
    let result = analyzer.analyze();
    assert_eq!(result.provide_inject_matches.len(), 2);
    let reactivity_losses = result
        .cross_file_reactivity_issues
        .iter()
        .filter(|issue| {
            matches!(
                issue.kind,
                crate::rules::CrossFileReactivityIssueKind::InjectValueDestructured { .. }
            )
        })
        .count();
    assert_eq!(reactivity_losses, 2);
}
