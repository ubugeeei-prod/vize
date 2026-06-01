use super::*;

#[test]
fn async_watch_mutating_ref_without_cleanup_is_error() {
    let mut analyzer = analyzer_with_single(
        r#"import { ref, watch } from 'vue'
const query = ref('')
const result = ref(null)
watch(query, async () => {
  const next = await load(query.value)
  result.value = next
})"#,
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::Error
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::AsyncBoundaryCrossing {
                    variable_name,
                    async_context,
                } if variable_name == "result" && async_context.contains("watch")
            )
    }));
}

#[test]
fn async_watch_with_cleanup_is_allowed() {
    let mut analyzer = analyzer_with_single(
        r#"import { ref, watch } from 'vue'
const query = ref('')
const result = ref(null)
watch(query, async (_value, _oldValue, onCleanup) => {
  let cancelled = false
  onCleanup(() => { cancelled = true })
  const next = await load(query.value)
  if (!cancelled) result.value = next
})"#,
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        &diagnostic.kind,
        CrossFileDiagnosticKind::AsyncBoundaryCrossing { variable_name, .. }
            if variable_name == "result"
    )));
}

#[test]
fn async_watch_with_on_watcher_cleanup_is_allowed() {
    let mut analyzer = analyzer_with_single(
        r#"import { onWatcherCleanup, ref, watch } from 'vue'
const query = ref('')
const result = ref(null)
watch(query, async () => {
  const controller = new AbortController()
  onWatcherCleanup(() => controller.abort())
  const next = await load(query.value, controller.signal)
  result.value = next
})"#,
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        &diagnostic.kind,
        CrossFileDiagnosticKind::AsyncBoundaryCrossing { variable_name, .. }
            if variable_name == "result"
    )));
}

#[test]
fn async_watch_effect_mutation_is_error() {
    let mut analyzer = analyzer_with_single(
        r#"import { ref, watchEffect } from 'vue'
const result = ref(null)
watchEffect(async () => {
  result.value = await load()
})"#,
    );

    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::Error
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::WatchEffectWithAsync { async_operation }
                    if async_operation == "await"
            )
    }));
}

#[test]
fn watch_alias_async_mutation_is_error() {
    let mut analyzer = analyzer_with_single(
        r#"import { ref, watch as observe } from 'vue'
const query = ref('')
const result = ref(null)
observe(query, async () => {
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
            } if variable_name == "result" && async_context.contains("watch")
        )
    }));
}
