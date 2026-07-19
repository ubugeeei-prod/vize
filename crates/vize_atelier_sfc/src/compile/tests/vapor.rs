//! Vapor-mode SFC compilation regression coverage.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit.

use super::super::compile_sfc;
use crate::types::{ScriptCompileOptions, SfcCompileOptions, TemplateCompileOptions};
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn test_template_only_sfc_vapor_output_mode() {
    let source = r#"<template><div>{{ msg }}</div></template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_script_setup_sfc_vapor_output_mode() {
    let source = r#"<script setup lang="ts">
import { computed, ref } from 'vue'

const count = ref(1)
const doubled = computed(() => count.value * 2)
</script>

<template>
  <div>{{ count }} {{ doubled }}</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

// Regression test for #3073: a Vapor SFC `<slot>` must lower to the Vapor
// runtime's `createSlot`, never the vdom `renderSlot` helper, and nested slot
// blocks must insert with the runtime's `insert(block, parent)` argument order.
#[test]
fn test_script_setup_sfc_vapor_slot_outlet() {
    let source = r#"<script setup lang="ts"></script>

<template>
  <div><slot /></div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        script: ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}
