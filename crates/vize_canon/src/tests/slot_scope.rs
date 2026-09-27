use super::virtual_ts_tests::generate_virtual_ts_from_sfc;

/// A dynamic slot name (`v-slot:[slot]`) selects the declared payloads
/// of every slot the name can resolve to, through the overload-aware
/// payload alias, and never becomes a static slot key.
#[test]
fn virtual_ts_dynamic_component_v_slot_uses_slot_prop_union() {
    let source = r#"<script setup lang="ts">
import MyList from './MyList.vue'

const slot = 'items'
const items = ['a', 'b']
</script>

<template>
  <MyList :items="items" v-slot:[slot]="{ item }">{{ item }}</MyList>
</template>"#;

    let virtual_ts = generate_virtual_ts_from_sfc(source);

    assert!(
        virtual_ts.contains(
            "[__K in keyof __S & __N]-?: NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : never"
        ),
        "dynamic slot names should select matching declared slot props:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains(r#"__S extends { "slot"?: (props: infer __P"#),
        "dynamic slot expression must not be injected as a static slot key:\n{virtual_ts}"
    );
}
