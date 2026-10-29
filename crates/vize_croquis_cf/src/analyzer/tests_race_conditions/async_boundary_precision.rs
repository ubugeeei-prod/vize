use super::*;

fn async_diagnostics(script: &str) -> Vec<(CompactString, CompactString)> {
    let mut analyzer = analyzer_with_single(script);
    let result = analyzer.analyze();
    let mut diagnostics = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::AsyncBoundaryCrossing {
                variable_name,
                async_context,
            } => Some((variable_name.clone(), async_context.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    diagnostics.sort_unstable();
    diagnostics
}

#[test]
fn accepted_boundaries_do_not_report_stale_writes() {
    let diagnostics = async_diagnostics(
        r#"import { nextTick, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'
const open = ref(true)
const items = ref<string[]>([])
const dates = ref<string[]>([])
const step = ref(0)
const percent = ref(0)
const overflowing = ref(false)
const width = ref(0)
const box = useTemplateRef<HTMLElement>('box')
let latest = 0
watch(open, () => {
  const id = ++latest
  fetchDates().then(found => {
    if (id !== latest) return
    dates.value = found
  })
})
watch(open, async () => {
  step.value = 0
  await prefetch()
})
let timer: ReturnType<typeof setInterval> | undefined
function start() {
  clearInterval(timer)
  percent.value = 0
  timer = setInterval(() => {
    percent.value = Math.min(100, percent.value + 2)
  }, 100)
}
onBeforeUnmount(() => clearInterval(timer))
watch(items, () => {
  requestAnimationFrame(() => {
    overflowing.value = (box.value?.scrollWidth ?? 0) > (box.value?.clientWidth ?? 0)
  })
})
watch(items, async () => {
  await nextTick()
  width.value = box.value?.offsetWidth ?? 0
})"#,
    );

    assert_eq!(diagnostics, []);
}

#[test]
fn stale_controls_keep_exact_async_diagnostics() {
    let diagnostics = async_diagnostics(
        r#"import { nextTick, ref, useTemplateRef, watch } from 'vue'
const source = ref(0)
const dates_no_guard = ref<string[]>([])
const dates_wrong_guard = ref<string[]>([])
const result = ref<string[]>([])
const percent = ref(0)
const overflowing = ref(false)
const width = ref(0)
const box = useTemplateRef<HTMLElement>('box')
let latest = 0
watch(source, () => {
  const id = ++latest
  fetchDates().then(found => {
    dates_no_guard.value = found
  })
  fetchDates().then(found => {
    if (id === latest) return
    dates_wrong_guard.value = found
  })
})
watch(source, async () => {
  const found = await fetchDates()
  result.value = found
})
let timer: ReturnType<typeof setInterval> | undefined
function start() {
  timer = setInterval(() => {
    percent.value = Math.min(100, percent.value + 2)
  }, 100)
}
watch(source, () => {
  const measured = box.value?.scrollWidth ?? 0
  requestAnimationFrame(() => {
    overflowing.value = measured > 0
  })
})
watch(source, async () => {
  const measured = box.value?.offsetWidth ?? 0
  await nextTick()
  width.value = measured
})"#,
    );

    assert_eq!(
        diagnostics,
        [
            (
                CompactString::new("dates_no_guard"),
                CompactString::new("then")
            ),
            (
                CompactString::new("dates_wrong_guard"),
                CompactString::new("then"),
            ),
            (
                CompactString::new("overflowing"),
                CompactString::new("requestAnimationFrame"),
            ),
            (
                CompactString::new("percent"),
                CompactString::new("setInterval"),
            ),
            (
                CompactString::new("result"),
                CompactString::new("watch await"),
            ),
            (
                CompactString::new("width"),
                CompactString::new("watch await"),
            ),
        ]
    );
}

#[test]
fn chained_promises_and_async_intervals_remain_unsafe() {
    let diagnostics = async_diagnostics(
        r#"import { onBeforeUnmount, ref, watch } from 'vue'
const source = ref(0)
const chained = ref<string[]>([])
const status = ref(0)
watch(source, () => {
  fetchDates()
    .then(found => found)
    .then(found => {
      chained.value = found
    })
})
let timer: ReturnType<typeof setInterval> | undefined
function start() {
  clearInterval(timer)
  timer = setInterval(async () => {
    const next = await poll()
    status.value = next
  }, 100)
}
onBeforeUnmount(() => clearInterval(timer))"#,
    );

    assert_eq!(
        diagnostics,
        [
            (CompactString::new("chained"), CompactString::new("then"),),
            (
                CompactString::new("status"),
                CompactString::new("setInterval"),
            ),
        ]
    );
}
