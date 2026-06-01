use super::*;

#[test]
fn test_snapshot_reactivity_issues() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    // File with reactivity issues
    let mut comp = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    comp.analyze_script_setup(
        r#"import { inject, ref, computed } from 'vue'

// Good: Simple inject
const theme = inject('theme')

// Issue: Destructuring inject loses reactivity
const { count, name } = inject('state') as { count: number; name: string }

// Good: Using computed
const doubled = computed(() => count * 2)

// Good: Using ref
const localCount = ref(0)"#,
    );
    let analysis = comp.finish();

    analyzer.add_file_with_analysis(Path::new("Component.vue"), "", analysis);
    let result = analyzer.analyze();

    // Build output
    let mut output = String::new();
    output.push_str("=== Reactivity Analysis ===\n\n");

    output.push_str("== Reactivity Issues ==\n");
    for issue in &result.reactivity_issues {
        append!(output, "  File: {:?}\n", issue.file_id);
        append!(output, "    kind: {:?}\n", issue.kind);
        append!(output, "    source: {:?}\n", issue.source);
        output.push('\n');
    }

    output.push_str("== Cross-File Reactivity Issues ==\n");
    for issue in &result.cross_file_reactivity_issues {
        append!(output, "  File: {:?}\n", issue.file_id);
        append!(output, "    kind: {:?}\n", issue.kind);
        append!(output, "    related_file: {:?}\n", issue.related_file);
        output.push('\n');
    }

    assert_snapshot!(output);
}

#[test]
fn test_snapshot_provide_inject_patterns() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Various provide patterns
    let mut provider = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    provider.analyze_script_setup(
        r#"import { provide, ref, reactive, computed } from 'vue'

// String key provides
provide('stringKey', 'value')
provide('refValue', ref(0))
provide('reactiveValue', reactive({ a: 1 }))
provide('computedValue', computed(() => 42))

// Symbol key provide
const ThemeSymbol = Symbol('theme')
provide(ThemeSymbol, 'dark')"#,
    );

    // Various inject patterns
    let mut consumer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    consumer.analyze_script_setup(
        r#"import { inject } from 'vue'

// Simple inject
const str = inject('stringKey')

// Inject with default
const withDefault = inject('missing', 'default')

// Inject with type assertion
const typed = inject('refValue') as Ref<number>

// Destructuring inject
const { a } = inject('reactiveValue') as { a: number }

// Inject computed
const comp = inject('computedValue')"#,
    );

    analyzer.add_file_with_analysis(Path::new("Provider.vue"), "", provider.finish());
    analyzer.add_file_with_analysis(Path::new("Consumer.vue"), "", consumer.finish());

    // Build output
    let mut output = String::new();
    output.push_str("=== Provide/Inject Patterns ===\n\n");

    for entry in analyzer.registry().iter() {
        append!(output, "File: {}\n", entry.filename);

        if !entry.analysis.provide_inject.provides().is_empty() {
            output.push_str("  Provides:\n");
            for p in entry.analysis.provide_inject.provides() {
                append!(output, "    - key: {:?}\n", p.key);
                append!(output, "      value: {}\n", p.value);
            }
        }

        if !entry.analysis.provide_inject.injects().is_empty() {
            output.push_str("  Injects:\n");
            for i in entry.analysis.provide_inject.injects() {
                append!(output, "    - key: {:?}\n", i.key);
                append!(output, "      has_default: {}\n", i.default_value.is_some());
                append!(output, "      pattern: {:?}\n", i.pattern);
            }
        }
        output.push('\n');
    }

    assert_snapshot!(output);
}
