#![allow(clippy::disallowed_macros)] // `insta::assert_snapshot!` expands to `format!`.

use std::path::PathBuf;

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_carton::{String, ToCompactString};

fn temp_project_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut name = String::from("vize-sfc-imported-pick-intersection-props-");
    name.push_str(&std::process::id().to_compact_string());
    name.push('-');
    name.push_str(&nonce.to_compact_string());
    std::env::temp_dir().join(name.as_str())
}

#[test]
fn with_defaults_resolves_imported_pick_from_intersection_type() {
    let project = temp_project_dir();
    let src = project.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        src.join("config.ts"),
        r#"export interface CascaderCommonProps {
  virtualScroll?: boolean
  itemSize?: number
  height?: number
}

export const CASCADER_PANEL_ITEM_SIZE = 34
export const CASCADER_PANEL_HEIGHT = 204
"#,
    )
    .unwrap();

    let component = src.join("Menu.vue");
    let source = r#"<script lang="ts" setup>
import { computed } from 'vue'
import { CASCADER_PANEL_HEIGHT, CASCADER_PANEL_ITEM_SIZE } from './config'
import type { CascaderCommonProps } from './config'

const props = withDefaults(
  defineProps<
    {
      nodes: { uid: number }[]
      index: number
    } & Pick<CascaderCommonProps, 'virtualScroll' | 'itemSize' | 'height'>
  >(),
  {
    virtualScroll: false,
    itemSize: CASCADER_PANEL_ITEM_SIZE,
    height: CASCADER_PANEL_HEIGHT,
  }
)

const virtualScroll = computed(() => props.virtualScroll)
</script>

<template>
  <FixedSizeList v-if="virtualScroll" />
  <Node v-else v-for="node in nodes" :key="node.uid" />
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(component.to_string_lossy().as_ref().to_compact_string());
    let result = compile_sfc(&descriptor, options).expect("compile SFC");
    insta::assert_snapshot!(result.code.as_str());

    let _ = std::fs::remove_dir_all(project);
}
