use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn external_module_augmentation_uses_the_source_package_import_condition() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "external-esm-module-augmentation",
        &[
            (
                "src/global.d.ts",
                "import 'dual-pkg';\ndeclare module 'dual-pkg/api/index.js' {\n  interface Extra { name: string }\n  interface Status { extras?: Extra[] }\n}\n",
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import type { api } from 'dual-pkg'

const props = defineProps<{ status: api.Status }>()

function label(extra: api.Extra) {
  return extra.name
}
</script>

<template>
  <span>{{ props.status.extras?.map(label).join(',') }}</span>
</template>
"#,
            ),
        ],
    );
    std::fs::write(project_root.join("package.json"), r#"{"type":"module"}"#).unwrap();
    write_dual_package(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(snapshot, []);
}

fn write_dual_package(project_root: &std::path::Path) {
    let package = project_root.join("node_modules/dual-pkg");
    for dir in ["dist/esm/api", "dist/cjs/api"] {
        std::fs::create_dir_all(package.join(dir)).unwrap();
    }
    std::fs::write(
        package.join("package.json"),
        r#"{
  "name": "dual-pkg",
  "type": "module",
  "exports": {
    ".": {
      "import": { "types": "./dist/esm/index.d.ts", "default": "./dist/esm/index.js" },
      "require": { "types": "./dist/cjs/index.d.ts", "default": "./dist/cjs/index.js" }
    },
    "./*.js": {
      "import": { "types": "./dist/esm/*.d.ts", "default": "./dist/esm/*.js" },
      "require": { "types": "./dist/cjs/*.d.ts", "default": "./dist/cjs/*.js" }
    }
  }
}
"#,
    )
    .unwrap();
    for module in ["esm", "cjs"] {
        std::fs::write(
            package.join(format!("dist/{module}/index.d.ts")),
            "export type * as api from './api/index.js';\n",
        )
        .unwrap();
        std::fs::write(
            package.join(format!("dist/{module}/api/index.d.ts")),
            "export interface Status { id: string }\n",
        )
        .unwrap();
        std::fs::write(package.join(format!("dist/{module}/index.js")), "").unwrap();
        std::fs::write(package.join(format!("dist/{module}/api/index.js")), "").unwrap();
    }
}
