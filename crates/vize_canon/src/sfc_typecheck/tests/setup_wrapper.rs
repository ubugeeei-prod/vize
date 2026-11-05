use super::*;

#[test]
fn test_script_setup_only_virtual_ts_has_no_define_component_wrapper() {
    let source = r#"<script setup lang="ts">
const count = 1
</script>
<template>
  <div>{{ count }}</div>
</template>"#;
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result.virtual_ts.expect("virtual TS should be generated");

    assert!(
        !virtual_ts.contains("__vizeDefineComponent"),
        "script setup virtual TS must stay untouched:\n{virtual_ts}"
    );
}
