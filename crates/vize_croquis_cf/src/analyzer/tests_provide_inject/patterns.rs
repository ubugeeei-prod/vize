use super::*;

#[test]
fn test_inject_object_destructure_pattern() {
    use vize_croquis::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    // Destructuring inject() loses reactivity
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const { count, name } = inject('state') as { count: number; name: string }"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    // Should detect the inject with ObjectDestructure pattern
    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1, "Should have 1 inject");
    match &injects[0].pattern {
        InjectPattern::ObjectDestructure(props) => {
            assert!(props.contains(&vize_carton::CompactString::new("count")));
            assert!(props.contains(&vize_carton::CompactString::new("name")));
        }
        _ => panic!(
            "Expected ObjectDestructure pattern, got {:?}",
            injects[0].pattern
        ),
    }
}

#[test]
fn test_inject_simple_pattern() {
    use vize_croquis::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Simple inject without destructuring
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const state = inject('state')"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1);
    assert!(matches!(injects[0].pattern, InjectPattern::Simple));
}

#[test]
fn test_inject_destructure_with_type_assertion() {
    use vize_croquis::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    // Destructuring with TSAsExpression
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const { foo } = inject('data') as { foo: string }"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1);
    match &injects[0].pattern {
        InjectPattern::ObjectDestructure(props) => {
            assert!(props.contains(&vize_carton::CompactString::new("foo")));
        }
        _ => panic!("Expected ObjectDestructure pattern"),
    }
}

#[test]
fn test_inject_destructure_in_vue_sfc() {
    use vize_croquis::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    // Add Vue SFC script content (not full SFC - the caller should extract this)
    // The cross-file analyzer expects script content only for .vue files
    analyzer.add_file(
        Path::new("Child.vue"),
        r#"import { inject } from 'vue'

const { name } = inject('user') as { name: string; id: number }"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1, "Should have 1 inject");
    match &injects[0].pattern {
        InjectPattern::ObjectDestructure(props) => {
            assert!(
                props.contains(&vize_carton::CompactString::new("name")),
                "Should contain 'name' prop"
            );
        }
        other => panic!("Expected ObjectDestructure pattern, got {:?}", other),
    }
}

#[test]
fn test_inject_wrapped_in_torefs_tracks_inject_without_loss() {
    use vize_croquis::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject, toRefs } from 'vue'
const { count } = toRefs(inject('state') as { count: number })"#,
    );

    let result = analyzer.analyze();
    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1);
    assert!(matches!(injects[0].pattern, InjectPattern::Simple));
    assert!(
        result.diagnostics.iter().all(|diagnostic| !matches!(
            diagnostic.kind,
            crate::diagnostics::CrossFileDiagnosticKind::DestructuringBreaksReactivity { .. }
        )),
        "toRefs(inject(...)) should not be reported as reactivity loss"
    );
}
