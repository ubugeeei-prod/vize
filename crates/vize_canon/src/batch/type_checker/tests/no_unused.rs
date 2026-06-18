use super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};

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
