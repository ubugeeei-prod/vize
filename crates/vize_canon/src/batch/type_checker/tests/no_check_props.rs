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
