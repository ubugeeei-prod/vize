use super::{findings, lint_sfc};
use crate::{LintPreset, Linter, Severity};
use vize_carton::String;

fn expected_script_finding<'a>(
    sfc: &'a str,
    target: &'a str,
    occurrence: usize,
) -> (&'static str, Severity, u32, u32, String) {
    let start = sfc
        .match_indices(target)
        .nth(occurrence)
        .map(|(start, _)| start)
        .expect("mutation target");
    (
        "vue/no-mutating-props",
        Severity::Error,
        start as u32,
        (start + target.len()) as u32,
        String::new(format!(
            "Unexpected mutation of prop '{target}' in <script setup>"
        )),
    )
}

#[test]
fn reports_assignment_compound_assignment_and_update_on_a_props_object() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ count: number; profile: { name: string } }>()
props.count = 1
props.count += 1
props.count++
props.profile.name = 'Ada'
</script>
"#;
    let result = lint_sfc(sfc);
    let actual = findings(&result);
    let expected = [
        expected_script_finding(sfc, "props.count", 0),
        expected_script_finding(sfc, "props.count", 1),
        expected_script_finding(sfc, "props.count", 2),
        expected_script_finding(sfc, "props.profile.name", 0),
    ];

    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(actual.0, expected.0);
        assert_eq!(actual.1, expected.1);
        assert_eq!(actual.2, expected.2);
        assert_eq!(actual.3, expected.3);
        assert_eq!(actual.4, expected.4);
    }
}

#[test]
fn reports_mutations_of_destructured_and_aliased_props() {
    let sfc = r#"<script setup lang="ts">
let { count, enabled: isEnabled } = defineProps<{
  count: number
  enabled: boolean
}>()
count = 1
isEnabled--
</script>
"#;
    let result = lint_sfc(sfc);
    let actual = findings(&result);
    let expected = [
        expected_script_finding(sfc, "count", 2),
        expected_script_finding(sfc, "isEnabled", 1),
    ];

    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(
            (actual.2, actual.3, actual.4),
            (expected.2, expected.3, expected.4.as_str())
        );
    }
}

#[test]
fn reports_mutations_from_with_defaults() {
    let sfc = r#"<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 })
props.count *= 2
</script>
"#;
    let expected = expected_script_finding(sfc, "props.count", 0);
    let result = lint_sfc(sfc);
    let actual = findings(&result);

    assert_eq!(actual.len(), 1);
    assert_eq!(
        (actual[0].2, actual[0].3, actual[0].4),
        (expected.2, expected.3, expected.4.as_str())
    );
}

#[test]
fn ignores_unrelated_and_shadowed_bindings() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ count: number }>()
const local = { count: 0 }
local.count++

function mutate(props: { count: number }) {
  props.count += 1
}
</script>
"#;

    assert!(findings(&lint_sfc(sfc)).is_empty());
}

#[test]
fn ignores_a_user_defined_define_props_function() {
    let sfc = r#"<script setup lang="ts">
function defineProps<T>(): T {
  return {} as T
}
const ordinary = defineProps<{ count: number }>()
ordinary.count++
</script>
"#;

    assert!(findings(&lint_sfc(sfc)).is_empty());
}

#[test]
fn honors_an_sfc_level_eslint_disable_comment() {
    let sfc = r#"<script setup lang="ts">
/* eslint-disable vue/no-mutating-props */
const props = defineProps<{ count: number }>()
props.count++
</script>
"#;

    assert!(findings(&lint_sfc(sfc)).is_empty());
}

#[test]
fn runs_in_every_expected_preset() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ count: number }>()
props.count += 1
</script>
"#;

    for preset in [
        LintPreset::HappyPath,
        LintPreset::Essential,
        LintPreset::Opinionated,
        LintPreset::Ecosystem,
    ] {
        let result = Linter::with_preset(preset).lint_sfc(sfc, "Probe.vue");
        let count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
            .count();
        assert_eq!(count, 1, "preset {preset:?}");
    }
}
