//! Vapor-mode SFC compilation regression coverage.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit.
#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]

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

#[test]
fn test_normal_script_sfc_vapor_output_mode() {
    let source = r#"<script>
export default {
  name: 'NormalVapor'
}
</script>

<template>
  <div>Hello</div>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let opts = SfcCompileOptions {
        vapor: true,
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("Failed to compile SFC");

    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_vapor_slot_hydration_creates_nodes_in_document_order() {
    let child = r#"<script setup vapor>
const label = "child"
</script>
<template><button><slot /></button></template>"#;
    let app = r#"<script setup vapor>
import { ref } from "vue"
import Child from "./Child.vue"
const msg = ref("hi")
</script>
<template><Child>Open</Child><p>{{ msg }}</p></template>"#;

    let compile = |source| {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
        compile_sfc(
            &descriptor,
            SfcCompileOptions {
                vapor: true,
                ..Default::default()
            },
        )
        .expect("compile SFC")
        .code
    };
    let child_code = compile(child);
    let app_code = compile(app);

    let insertion = child_code.find("_setInsertionState(").expect(&child_code);
    let slot = child_code
        .find("_createSlot(\"default\")")
        .expect(&child_code);
    assert!(
        insertion < slot,
        "slot needs its parent cursor: {child_code}"
    );

    let component = app_code
        .find("= _createComponentWithFallback(")
        .expect(&app_code);
    let paragraph = app_code.find("= t1()").expect(&app_code);
    assert!(
        component < paragraph,
        "component must hydrate first: {app_code}"
    );
}
