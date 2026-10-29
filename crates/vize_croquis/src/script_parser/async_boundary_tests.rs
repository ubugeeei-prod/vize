use super::parse_script_setup;
use crate::race::RaceConditionRiskKind;
use vize_carton::{CompactString, cstr};

#[test]
fn accepted_async_boundary_shapes_have_no_risks() {
    let result = parse_script_setup(
        r#"
import { nextTick, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'
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
  if (!open.value) return
  const id = ++latest
  fetchDates().then(found => {
    if (id !== latest) return
    dates.value = found
  })
})

watch(open, async () => {
  if (!open.value) return
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
})
"#,
    );

    assert_eq!(result.race_conditions.risks(), []);
}

#[test]
fn unsafe_async_boundary_controls_stay_reported() {
    let result = parse_script_setup(
        r#"
import { nextTick, ref, useTemplateRef, watch } from 'vue'
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
})
"#,
    );

    let mut inventory = result
        .race_conditions
        .risks()
        .iter()
        .map(|risk| {
            let category = match &risk.kind {
                RaceConditionRiskKind::AsyncWatcherMutation { .. } => "watcher",
                RaceConditionRiskKind::ScheduledMutation { .. } => "scheduled",
                RaceConditionRiskKind::PromiseContinuationMutation { .. } => "promise",
                RaceConditionRiskKind::AsyncWatchEffect { .. } => "watch-effect",
                RaceConditionRiskKind::AsyncLifecycleMutation { .. } => "lifecycle",
            };
            (
                category,
                risk.kind.async_context(),
                risk.kind.mutated_targets().to_vec(),
            )
        })
        .collect::<Vec<(&str, CompactString, Vec<CompactString>)>>();
    inventory.sort_unstable_by(|left, right| left.2.cmp(&right.2));

    assert_eq!(
        inventory,
        [
            ("promise", cstr!("then"), vec![cstr!("dates_no_guard")]),
            ("promise", cstr!("then"), vec![cstr!("dates_wrong_guard")],),
            (
                "scheduled",
                cstr!("requestAnimationFrame"),
                vec![cstr!("overflowing")],
            ),
            ("scheduled", cstr!("setInterval"), vec![cstr!("percent")]),
            ("watcher", cstr!("watch await"), vec![cstr!("result")]),
            ("watcher", cstr!("watch await"), vec![cstr!("width")]),
        ]
    );
}

#[test]
fn chained_promise_calls_keep_distinct_safe_spans() {
    let result = parse_script_setup(
        r#"
import { ref, watch } from 'vue'
const source = ref(0)
const result = ref<string[]>([])
watch(source, () => {
  fetchDates()
    .then(found => found)
    .then(found => {
      result.value = found
    })
})
"#,
    );
    let kinds = result
        .race_conditions
        .risks()
        .iter()
        .map(|risk| risk.kind.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        [RaceConditionRiskKind::PromiseContinuationMutation {
            async_operation: cstr!("then"),
            mutated_targets: vec![cstr!("result")],
        }]
    );
}

#[test]
fn cleared_async_interval_stays_risky() {
    let result = parse_script_setup(
        r#"
import { onBeforeUnmount, ref } from 'vue'
const status = ref(0)
let timer: ReturnType<typeof setInterval> | undefined
function start() {
  clearInterval(timer)
  timer = setInterval(async () => {
    const next = await poll()
    status.value = next
  }, 100)
}
onBeforeUnmount(() => clearInterval(timer))
"#,
    );
    let kinds = result
        .race_conditions
        .risks()
        .iter()
        .map(|risk| risk.kind.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        [RaceConditionRiskKind::ScheduledMutation {
            scheduler_name: cstr!("setInterval"),
            mutated_targets: vec![cstr!("status")],
        }]
    );
}

#[test]
fn equivalent_interval_cleanup_shapes_are_allowed() {
    let result = parse_script_setup(
        r#"
import { onScopeDispose, onUnmounted, ref } from 'vue'
const first = ref(0)
const second = ref(0)
let unmountedTimer: ReturnType<typeof setInterval> | undefined
let scopedTimer: ReturnType<typeof setInterval> | undefined
function restartUnmounted() {
  if (unmountedTimer) clearInterval(unmountedTimer)
  unmountedTimer = setInterval(() => {
    first.value++
  }, 100)
}
function restartScoped() {
  if (scopedTimer) {
    clearInterval(scopedTimer)
  }
  scopedTimer = setInterval(() => {
    second.value++
  }, 100)
}
onUnmounted(() => {
  if (unmountedTimer) clearInterval(unmountedTimer)
})
onScopeDispose(() => {
  if (scopedTimer) clearInterval(scopedTimer)
})
"#,
    );

    assert_eq!(result.race_conditions.risks(), []);
}
