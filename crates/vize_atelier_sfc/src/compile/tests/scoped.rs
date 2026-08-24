//! Scoped-style SFC compilation regressions.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit.

use super::super::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn test_scoped_hoisted_static_vnode_carries_scope_id() {
    // Regression: module-level hoisted static vnodes are created at import time,
    // when the runtime's `currentScopeId` is null, so the runtime cannot stamp
    // the scoped-CSS attribute on them. The compiler must bake `data-v-*` into
    // their props directly. A nested static element (e.g. `<rect>` inside a
    // dynamic `<svg>` subtree) is hoisted to module scope and must keep the
    // scope id so scoped CSS selectors continue to match it.
    //
    // Note: `template.scoped` is intentionally NOT set here — the fix derives
    // the scoped signal from the descriptor's `<style scoped>` block.
    let source = r#"<script setup>
import { ref } from 'vue'
const active = ref(false)
</script>
<template>
    <div class="fixture-body">
    <div class="wrapper" :class="{ active }">
      <svg><rect class="marker" x="1" y="1" /></svg>
    </div>
  </div>
</template>
<style scoped>.marker{fill:black}</style>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let opts = SfcCompileOptions {
        scope_id: Some("abc123".into()),
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("compile");
    let hoisted_line = result
        .code
        .lines()
        .find(|line| line.contains("_hoisted_") && line.contains("\"rect\""))
        .unwrap_or_else(|| panic!("expected a hoisted rect vnode in:\n{}", result.code));
    assert!(
        hoisted_line.contains("\"data-v-abc123\""),
        "hoisted static vnode is missing the scope id attribute:\n{}",
        hoisted_line
    );
}

#[test]
fn test_scoped_runtime_vnodes_do_not_bake_scope_id() {
    let source = r#"<template>
  <div :class="[]">Hoge</div>
  <Test :x="1" />
</template>

<script setup>
const marker = true
</script>

<style scoped lang="scss">
div {
  color: red;
}
</style>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let opts = SfcCompileOptions {
        scope_id: Some("abc123".into()),
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, opts).expect("compile");

    assert!(
        !result.code.contains("data-v-abc123"),
        "runtime-created VNodes should not receive baked scoped attrs:\n{}",
        result.code
    );
    assert!(
        result
            .code
            .contains("_createVNode(_component_Test, { x: 1 })"),
        "scoped SFC component props should stay inline like Vue's compiler:\n{}",
        result.code
    );
    assert!(
        !result
            .code
            .contains("_createVNode(_component_Test, _hoisted_"),
        "scoped SFC component props should not use a hoisted props alias:\n{}",
        result.code
    );
}
