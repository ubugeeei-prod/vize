use super::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn generated_emit_props_do_not_reference_vue_emits_to_props() {
    let source = r#"<script setup lang="ts">
defineEmits<{
  (event: "click", value: MouseEvent): void
}>()
</script>

<template>
  <button>Click</button>
</template>"#;
    let options = SfcTypeCheckOptions::new("EmitProps.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");

    assert!(!virtual_ts.contains("EmitsToProps"), "{virtual_ts}");
    assert!(
        virtual_ts.contains("type __EmitProps<T> = { [K in keyof __EmitOptions<T> & string"),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("type __VizeCamelize<S extends string>"),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("as __VizeHandlerKey<K>"),
        "{virtual_ts}"
    );
}
