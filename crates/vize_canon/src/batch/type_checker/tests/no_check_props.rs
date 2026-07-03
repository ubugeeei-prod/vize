use super::{BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;

#[test]
fn disables_keyed_define_props_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "no-check-props-keyed-define-props",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
type Props = { known: string } & { other?: boolean }
defineProps<Props>()
</script>

<template>
  <div>{{ known }} {{ isMini }}</div>
</template>
"#,
        )],
    );

    let Some(snapshot) = diagnostics_with_check_props_disabled(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .all(|(_, _, message)| !message.contains("keyof Props") && !message.contains("isMini")),
        "check_props=false should suppress keyed prop diagnostics: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn no_check_props_and_emits_keep_runtime_prop_helper_parseable() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "no-check-runtime-prop-helper-syntax",
        &[(
            "src/ConfirmDialog.vue",
            r#"<script setup lang="ts">
const props = defineProps({
  title: String,
  visible: Boolean,
});
</script>

<template>
  <dialog v-if="visible">{{ title }}</dialog>
</template>
"#,
        )],
    );

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.set_virtual_ts_checks(false, true, false);
    checker.scan_project().unwrap();

    let virtual_files = checker.virtual_files();
    let helper_source = checker
        .shared_helpers_preamble()
        .unwrap_or_else(|| virtual_files[0].content.as_str());
    assert!(
        helper_source.contains(
            "type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V"
        ),
        "runtime prop helper should use TypeScript construct/function syntax:\n{helper_source}"
    );
    assert!(
        !helper_source.contains("{ new (...args: any[]): infer V }"),
        "runtime prop helper must not use an invalid inferred construct signature:\n{helper_source}"
    );

    let result = checker.check_project().unwrap();
    let syntax_errors = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == Some(1005))
        .collect::<Vec<_>>();
    assert!(
        syntax_errors.is_empty(),
        "disabled prop/emit checks must still emit valid virtual TS: {syntax_errors:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn diagnostics_with_check_props_disabled(
    project_root: &std::path::Path,
) -> Option<Vec<(vize_carton::String, Option<u32>, vize_carton::String)>> {
    let mut checker = BatchTypeChecker::new(project_root).ok()?;
    checker.set_virtual_ts_checks(false, true, true);
    checker.scan_project().ok()?;
    let result = checker.check_project().ok()?;

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                vize_carton::cstr!(
                    "{}:{}:{} {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    match diagnostic.severity {
                        1 => "error",
                        2 => "warning",
                        3 => "info",
                        _ => "hint",
                    },
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    Some(snapshot)
}
