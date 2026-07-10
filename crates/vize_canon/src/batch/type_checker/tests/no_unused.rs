use super::{
    BatchTypeChecker, TypeChecker, create_project_case_without_node_modules, relative_path,
    resolve_test_tsgo_binary, snapshot_project_diagnostics,
};
use vize_carton::cstr;

#[test]
fn batch_type_checker_marks_art_bindings_as_used_with_no_unused_locals() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "art-bindings-no-unused-locals",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const schema = { fields: [] as string[] }
function handleSubmit() {}
</script>

<art>
  <variant name="Default" default>
    <AfsForm :schema="schema" @submit="handleSubmit" />
  </variant>
</art>
"#,
        )],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/App.vue"
                && *code == Some(6133)
                && (message.contains("schema") || message.contains("handleSubmit")))
        }),
        "art bindings should not report TS6133, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn no_check_template_bindings_still_marks_template_bindings_as_used() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "no-check-template-bindings-no-unused-locals",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'

const isLoading = false
function confirmDialog() {}
const unusedLocal = 1
</script>

<template>
  <Child :busy="isLoading" @save="confirmDialog" />
  {{ missingItems }}
</template>
"#,
            ),
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ busy?: boolean }>()
defineEmits<{ save: [] }>()
</script>

<template><button /></template>
"#,
            ),
        ],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics_without_template_checks(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/App.vue"
                && *code == Some(6133)
                && (message.contains("Child")
                    || message.contains("isLoading")
                    || message.contains("confirmDialog")))
        }),
        "template bindings should not report TS6133, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(6133) && message.contains("unusedLocal")
        }),
        "unreferenced script bindings should still report TS6133, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/App.vue" && *code == Some(2304) && message.contains("missingItems"))
        }),
        "template binding diagnostics should stay disabled, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn define_props_result_binding_is_used_when_template_reads_direct_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "define-props-result-template-props-no-unused-locals",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
interface Props {
  width?: number
  label: string
  required?: boolean
}

const props = defineProps<Props>()
const unusedLocal = 1
</script>

<template>
  <label>
    {{ label }} {{ width }} {{ required }}
  </label>
</template>
"#,
            ),
            (
                "src/WithDefaults.vue",
                r#"<script setup lang="ts">
interface Props {
  count?: number
  label: string
}

const props = withDefaults(defineProps<Props>(), {
  count: 0,
})
const unusedLocal = 1
</script>

<template>
  <button>{{ label }}: {{ count }}</button>
</template>
"#,
            ),
        ],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !((*file == "src/App.vue" || *file == "src/WithDefaults.vue")
                && *code == Some(6133)
                && message.contains("props"))
        }),
        "defineProps result should count as used when template reads direct props, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(6133) && message.contains("unusedLocal")
        }),
        "unrelated unused locals should still report TS6133 for plain defineProps, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/WithDefaults.vue" && *code == Some(6133) && message.contains("unusedLocal")
        }),
        "unrelated unused locals should still report TS6133 for withDefaults, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn define_props_result_binding_still_reports_unused_without_template_prop_reads() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "define-props-result-unused-without-template-props",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
interface Props {
  label: string
}

const props = defineProps<Props>()
const used = 1
</script>

<template>{{ used }}</template>
"#,
        )],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(6133) && message.contains("props")
        }),
        "defineProps result should still report TS6133 when template does not read props, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_does_not_report_default_export_alias_as_unused() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "options-default-no-unused-locals",
        &[(
            "src/Page.vue",
            r#"<script lang="ts">
export default {
  name: "PagesChangePassword",
  layout: "no-header",
};
</script>

<template>
  <main />
</template>
"#,
        )],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Page.vue" && *code == Some(6133) && message.contains("__default__"))
        }),
        "generated default export alias should not report TS6133, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn snapshot_project_diagnostics_without_template_checks(
    project_root: &std::path::Path,
) -> Option<Vec<(vize_carton::String, Option<u32>, vize_carton::String)>> {
    let mut checker = BatchTypeChecker::new(project_root).ok()?;
    checker.set_virtual_ts_checks(true, false, true);
    checker.scan_project().ok()?;
    let result = checker.check_project().ok()?;

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
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

fn write_no_unused_tsconfig(project_root: &std::path::Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "noUnusedLocals": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}
