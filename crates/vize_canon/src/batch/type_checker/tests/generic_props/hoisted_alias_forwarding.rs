use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

/// A setup type that names the SFC generic is lifted to module scope with the
/// parameter re-declared (`= any` fallback). A lifted declaration that reads
/// another one must pass the parameter on; read bare, `StepValue` was
/// `StepValue<any>` and the `satisfies` below checked nothing.
#[test]
fn hoisted_generic_declarations_check_through_each_other() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-hoisted-alias-forwarding",
        &[(
            "src/Steps.vue",
            r#"<script setup lang="ts" generic="TStep extends { value: string }">
type StepValue = TStep["value"];
type Expose = {
  readonly step: TStep | null;
  readonly value: StepValue | null;
};

const { steps } = defineProps<{ steps: readonly TStep[] }>();
const exposed = {
  step: steps[0] ?? null,
  value: steps.length,
} satisfies Expose;
defineExpose(exposed);
</script>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let reported: Vec<_> = snapshot
        .iter()
        .map(|(file, code, _)| (file.as_str(), *code))
        .collect();
    assert_eq!(reported, [("src/Steps.vue", Some(2322))], "{snapshot:#?}");
    let _ = std::fs::remove_dir_all(&project_root);
}
