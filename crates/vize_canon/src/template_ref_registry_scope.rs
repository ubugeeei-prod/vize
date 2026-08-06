//! Scope of the static template ref registry in virtual TS (#3896).
//!
//! Kept as its own crate-root test module (declared in `lib.rs`) rather than in
//! `tests.rs`: that file is already over the 350-line source-length limit, so
//! adding to it trips the source-length guard. Keep new cases here instead.

use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};

/// Retyping the `useTemplateRef` shim is the registry's only route to a
/// diagnostic, so an SFC that never names it must not pay for the extra
/// declarations: `ref="name"` alone is far more common than the macro.
#[test]
fn virtual_ts_omits_the_template_ref_registry_without_use_template_ref() {
    let source = r#"<script setup lang="ts">
const label = 'hi'
</script>

<template>
  <div>
    <input ref="inputRef" />
    <span>{{ label }}</span>
  </div>
</template>"#;

    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result
        .virtual_ts
        .expect("type_check_sfc must produce virtual TypeScript");

    assert!(
        !virtual_ts.contains("__VizeTemplateRefs"),
        "registry must stay out of an SFC that never names useTemplateRef:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("__VizeDomElement"),
        "element helper must stay out with the registry:\n{virtual_ts}"
    );
}
