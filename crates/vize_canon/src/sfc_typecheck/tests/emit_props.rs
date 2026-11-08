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
        virtual_ts.contains("type __EmitProps<T> = $ResolvedEmitProps<T>;"),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(
            "type $ResolvedEmitProps<T, O = __EmitOptions<T>, L = $LiteralEmitOptions<T>>"
        ),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("$IsAny<T> extends true ? $EmitPropsFromOptions<O>"),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("string extends keyof O ? [keyof L] extends [never]"),
        "a catch-all signature must retain any literal overload handlers:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("type $Camelize<S extends string>"),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("as ($Camelize<K> extends keyof T ? string extends keyof T ? $HandlerKey<K> : never : $HandlerKey<K>) | `on${Capitalize<K>}`]"),
        "Vue keys kebab events as `onScroll-top`; the camelized key alone rejects it, and a declared camel twin owns its key (#11453):\n{virtual_ts}"
    );
}
