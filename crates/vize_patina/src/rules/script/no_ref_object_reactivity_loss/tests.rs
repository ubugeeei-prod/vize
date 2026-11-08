use super::{MESSAGE, NoRefObjectReactivityLoss};
use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::rules::script::{ScriptLintResult, ScriptLinter};

const RULE: &str = super::RULE;

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoRefObjectReactivityLoss));
    linter.lint(source, 0)
}

fn messages(source: &str) -> Vec<std::string::String> {
    lint(source)
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

fn assert_copies(source: &str, count: usize) {
    assert_eq!(messages(source), vec![MESSAGE.to_owned(); count]);
}

#[test]
fn reports_same_scope_value_copies() {
    assert_copies(
        r#"
import { ref } from 'vue'
const count = ref(0)
const value1 = count.value
const { value: value2 } = count
const value3 = fn(count.value)
const { value: value4 = 42 } = count
if (foo) {
  const value1 = count.value
}
const direct = ref(0).value
const copiedIntoAnotherRef = ref(count.value)
"#,
        7,
    );
}

#[test]
fn reports_to_refs_copies() {
    assert_copies(
        r#"
import { toRefs } from 'vue'
const { count } = toRefs(foo)
const value1 = count.value
const { value: value2 } = count
const value3 = fn(count.value)
const refs = toRefs(foo)
const nested = refs.count.value
const { value: value4 } = refs.count
const called = fn(refs.count.value)
const { foo: named = 42 } = refs
const loose = named.value
"#,
        7,
    );
}

#[test]
fn reports_factory_and_define_model_copies() {
    assert_copies(
        r#"
import { ref, computed, shallowRef, customRef, toRef } from 'vue'
const r = ref(0)
const c = computed(() => r.value)
const sr = shallowRef({ count: 1 })
const cr = customRef((track, trigger) => {
  return {
    get() { return sr.value },
    set(newValue) { sr.value = newValue }
  }
})
const tr = toRef(sr, 'count')
console.log(r.value, c.value, sr.value, cr.value, tr.value)
const model1 = defineModel()
const [model2, mod] = defineModel()
console.log(model1.value, model2.value, mod.value)
"#,
        7,
    );
}

#[test]
fn allows_nested_reads_and_writes() {
    let source = r#"
import { ref, computed, toRefs } from 'vue'
const count = ref(0)
const value1 = computed(() => count.value)
const value2 = fn(count)
const value3 = computed(() => fn(count.value))
count.value = 42
count.value++
;({ foo: count.value } = bar)
function later() { return count.value }
const { item } = toRefs(foo)
const watched = computed(() => item.value)
const refs = toRefs(foo)
const watchedBag = computed(() => refs.item.value)
const [a] = ref(0)
const copiedIntoAnotherRef = ref(count.value) as Ref<number>
foo.bar = ref(0)
let alias = ref(0)
alias = alias
unknown.value
"#;
    assert!(messages(source).is_empty(), "{:?}", messages(source));
}

#[test]
fn reports_nested_value_copies_inside_a_ref_initializer() {
    assert_copies(
        r#"
import { ref } from 'vue'
const locale = ref('en-US')
const month = ref(createMonth({ locale: locale.value })) as Ref<Month>
"#,
        1,
    );
}

#[test]
fn type_asserted_ref_factories_do_not_define_tracked_bindings() {
    let source = r#"
import type { Ref } from 'vue'
import { ref } from 'vue'
const value = ref({ start: 1, end: 2 }) as Ref<{ start: number, end: number }>
const first = ref(value.value.start) as Ref<number>
const second = ref(value.value.end) as Ref<number>
consume(first.value, second.value)
"#;
    assert!(messages(source).is_empty(), "{:?}", messages(source));
}

#[test]
fn tracks_a_ref_factory_assigned_after_declaration() {
    assert_copies(
        r#"
import { shallowRef } from 'vue'
function ensure(file) {
  let blobRef = cache.get(file.id)
  if (!blobRef) {
    blobRef = shallowRef(file)
  }
  return blobRef.value
}
"#,
        1,
    );
}

#[test]
fn stays_opt_in_until_same_scope_value_reads_are_precise() {
    let meta = crate::linter::script_rules::builtin_script_rules()
        .into_iter()
        .find(|rule| rule.name == RULE)
        .expect(RULE);
    assert_eq!(meta.default_severity, Severity::Warning);
    assert!(meta.presets.is_empty());
}

#[test]
fn resolves_vue_factories_imported_by_the_sibling_script_block() {
    let source = r#"<script lang="ts">
import { computed, toRefs } from 'vue'
</script>
<script setup lang="ts">
const refs = toRefs(props)
const item = refs.item.value
const selected = computed(() => item).value
</script>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(source, "Probe.vue");
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .collect();
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
}
