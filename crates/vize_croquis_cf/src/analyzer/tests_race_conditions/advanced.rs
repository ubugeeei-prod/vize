use super::*;

#[test]
fn scheduled_mutation_without_cleanup_is_error() {
    let mut analyzer = analyzer_with_single(
        r#"import { ref } from 'vue'
const count = ref(0)
setTimeout(() => {
  count.value++
}, 10)"#,
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::AsyncBoundaryCrossing {
                variable_name,
                async_context,
            } if variable_name == "count" && async_context == "setTimeout"
        )
    }));
}

#[test]
fn promise_continuation_mutation_is_error() {
    let mut analyzer = analyzer_with_single(
        r#"import { ref } from 'vue'
const result = ref(null)
fetch('/api').then((response) => {
  result.value = response
})"#,
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::AsyncBoundaryCrossing {
                variable_name,
                async_context,
            } if variable_name == "result" && async_context == "then"
        )
    }));
}

#[test]
fn async_lifecycle_mutation_is_error() {
    let mut analyzer = analyzer_with_single(
        r#"import { onMounted, ref } from 'vue'
const result = ref(null)
onMounted(async () => {
  result.value = await load()
})"#,
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::AsyncBoundaryCrossing {
                variable_name,
                async_context,
            } if variable_name == "result" && async_context.contains("onMounted")
        )
    }));
}

#[test]
fn injected_state_async_mutation_reports_provider_and_sibling_writers() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_race_conditions(true));
    let provider_id = analyzer.add_file_with_analysis(
        Path::new("Provider.vue"),
        "",
        script_analysis(
            r#"import { provide, reactive } from 'vue'
import First from './First.vue'
import Second from './Second.vue'
const store = reactive({ count: 0 })
provide('store', store)"#,
            &["First", "Second"],
        ),
    );
    let first_id = analyzer.add_file_with_analysis(
        Path::new("First.vue"),
        "",
        script_analysis(
            r#"import { inject, ref, watch } from 'vue'
const store = inject('store')!
const query = ref('')
watch(query, async () => {
  await load()
  store.count = 1
})"#,
            &[],
        ),
    );
    let second_id = analyzer.add_file_with_analysis(
        Path::new("Second.vue"),
        "",
        script_analysis(
            r#"import { inject, ref, watch } from 'vue'
const store = inject('store')!
const query = ref('')
watch(query, async () => {
  await load()
  store.count = 2
})"#,
            &[],
        ),
    );
    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let first_diag = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.primary_file == first_id
                && matches!(
                    &diagnostic.kind,
                    CrossFileDiagnosticKind::InjectedAsyncMutationRace {
                        key,
                        writer_count,
                        ..
                    } if key == "store" && *writer_count == 2
                )
        })
        .expect("first async injected mutation should be reported");

    assert!(
        first_diag
            .related_files
            .iter()
            .any(|(file_id, _, _)| *file_id == provider_id)
    );
    assert!(
        first_diag
            .related_files
            .iter()
            .any(|(file_id, _, _)| *file_id == second_id)
    );
}
