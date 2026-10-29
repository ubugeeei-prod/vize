//! The shared batch preamble is anchored at the root tsconfig, while generated
//! SFC modules retain their package-relative paths. Vue-dependent helpers must
//! therefore stay in each generated module so nested packages use their own
//! `ComponentCustomProps` augmentation and nominal `Ref` identity.

use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics, write_test_vue_stub,
};

#[test]
fn package_local_vue_public_attrs_and_exposed_refs_keep_the_app_identity() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "package-local-vue-helper-identity",
        &[
            (
                "packages/app/src/Child.vue",
                r#"<script setup lang="ts">
import { ref } from 'vue'

const count = ref(1)
defineExpose({ count })
</script>

<template><span>{{ count }}</span></template>
"#,
            ),
            (
                "packages/app/src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template><Child app-only="accepted" /></template>
"#,
            ),
            (
                "packages/app/src/consumer.ts",
                r#"import Child from './Child.vue'

declare const child: InstanceType<typeof Child>
const count: number = child.count
void count
"#,
            ),
        ],
    );

    let root_modules = project_root.join("node_modules");
    let app_modules = project_root.join("packages/app/node_modules");
    write_test_vue_stub(&root_modules).unwrap();
    write_test_vue_stub(&app_modules).unwrap();

    std::fs::write(
        root_modules.join("vue/package.json"),
        r#"{"name":"vue","version":"3.4.0-root","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        app_modules.join("vue/package.json"),
        r#"{"name":"vue","version":"3.5.0-app","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        app_modules.join("@vue/runtime-core/index.d.ts"),
        "export interface ComponentCustomProps { appOnly?: string }\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "skipLibCheck": false
  },
  "vueCompilerOptions": { "strictTemplates": true },
  "include": ["packages/app/src/**/*"]
}"#,
    )
    .unwrap();

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert!(
        snapshot.is_empty(),
        "package-local Vue attrs and exposed Ref identity must survive batch checking: {snapshot:#?}"
    );
}
