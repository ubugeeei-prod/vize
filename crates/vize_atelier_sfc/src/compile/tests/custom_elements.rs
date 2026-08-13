//! Custom-element tag pattern coverage for SFC compilation.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit.

use super::super::compile_sfc;
use crate::types::{ScriptCompileOptions, SfcCompileOptions, TemplateCompileOptions};
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn test_script_setup_sfc_custom_elements_preserve_imports_and_pascal_case_intrinsics() {
    let source = r#"<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
const visible = true
</script>

<template>
  <TresCanvas>
    <TresMesh v-if="visible">
      <TresSpotLight></TresSpotLight>
    </TresMesh>
  </TresCanvas>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            custom_renderer: true,
            custom_elements: vec!["Tres*".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    assert!(
        result.code.contains(r#"_createBlock(_unref(TresCanvas)"#),
        "{}",
        result.code
    );
    assert!(
        result.code.contains(r#"_createElementBlock("TresMesh""#),
        "{}",
        result.code
    );
    assert!(
        result
            .code
            .contains(r#"_createElementVNode("TresSpotLight""#),
        "{}",
        result.code
    );
    assert!(!result.code.contains(r#"_resolveComponent("TresCanvas")"#));
    assert!(!result.code.contains(r#"_resolveComponent("TresMesh")"#));
    assert!(
        !result
            .code
            .contains(r#"_resolveComponent("TresSpotLight")"#)
    );
}
