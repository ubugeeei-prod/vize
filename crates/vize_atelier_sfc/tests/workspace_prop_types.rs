#![allow(clippy::disallowed_macros)] // `insta::assert_snapshot!` expands to `format!`.

use std::path::PathBuf;

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_carton::ToCompactString;

fn temp_project_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let name = vize_carton::cstr!("vize-sfc-workspace-props-{}-{nonce}", std::process::id());
    std::env::temp_dir().join(name.as_str())
}

#[test]
fn define_props_extends_unbuilt_workspace_package_source_interface() {
    let project = temp_project_dir();
    let hooks = project.join("node_modules/@element-plus/hooks");
    let empty_values = hooks.join("use-empty-values");
    let cascader_src = project.join("packages/components/cascader/src");
    std::fs::create_dir_all(&empty_values).unwrap();
    std::fs::create_dir_all(&cascader_src).unwrap();

    std::fs::write(
        hooks.join("package.json"),
        r#"{
  "name": "@element-plus/hooks",
  "types": "index.d.ts",
  "module": "index.ts",
  "main": "index.ts"
}"#,
    )
    .unwrap();
    std::fs::write(
        hooks.join("index.ts"),
        "export * from './use-empty-values'\n",
    )
    .unwrap();
    std::fs::write(
        empty_values.join("index.ts"),
        r#"export interface UseEmptyValuesProps {
  emptyValues?: unknown[]
  valueOnClear?: string | number | boolean | null
}
"#,
    )
    .unwrap();
    std::fs::write(
        cascader_src.join("cascader.ts"),
        r#"import type { UseEmptyValuesProps } from '@element-plus/hooks'

export interface CascaderComponentProps extends UseEmptyValuesProps {
  options?: unknown[]
  disabled?: boolean
}
"#,
    )
    .unwrap();

    let component = cascader_src.join("cascader.vue");
    let source = r#"<script lang="ts" setup>
import type { CascaderComponentProps } from './cascader'

const props = withDefaults(defineProps<CascaderComponentProps>(), {
  options: () => [],
  disabled: undefined,
})
</script>

<template>
  <div
    :data-empty-values="props.emptyValues"
    :data-value-on-clear="props.valueOnClear"
    :data-option-count="props.options.length"
  />
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(component.to_string_lossy().as_ref().to_compact_string());

    let result = compile_sfc(&descriptor, options).expect("compile SFC");
    insta::assert_snapshot!(result.code.as_str());

    let _ = std::fs::remove_dir_all(project);
}
