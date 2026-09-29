use super::parse_script_setup;
use crate::race::RaceConditionRiskKind;
use crate::reactivity::ReactivityLossKind;

fn mutates_results(kind: &RaceConditionRiskKind) -> bool {
    kind.mutated_targets()
        .iter()
        .any(|target| target == "results")
}

#[test]
fn compare_after_await_is_not_an_async_boundary() {
    let guarded = parse_script_setup(
        r#"
import { ref, watch } from 'vue'
const { query } = defineProps<{ query: string }>()
const results = ref<string[]>([])
watch(() => query, async () => {
  const requested = query
  const found = await search(requested)
  if (requested !== query) return
  results.value = found
})
"#,
    );
    assert!(
        guarded
            .race_conditions
            .risks()
            .iter()
            .all(|risk| !mutates_results(&risk.kind)),
        "{:?}",
        guarded.race_conditions.risks()
    );

    let open = parse_script_setup(
        r#"
import { ref, watch } from 'vue'
const query = ref('')
const results = ref<string[]>([])
watch(query, async () => {
  const next = await load(query.value)
  results.value = next
})
"#,
    );
    assert!(
        open.race_conditions
            .risks()
            .iter()
            .any(|risk| mutates_results(&risk.kind))
    );
}

#[test]
fn synchronous_timer_id_is_not_an_async_boundary() {
    let result = parse_script_setup(
        r#"
import { ref, watch } from 'vue'
const query = ref('')
const timer = ref<number | null>(null)
watch(query, () => {
  if (timer.value != null) clearTimeout(timer.value)
  timer.value = setTimeout(() => {}, 0)
})
"#,
    );
    assert!(
        result.race_conditions.risks().iter().all(|risk| !matches!(
            risk.kind,
            RaceConditionRiskKind::AsyncWatcherMutation { .. }
        )),
        "{:?}",
        result.race_conditions.risks()
    );
}
