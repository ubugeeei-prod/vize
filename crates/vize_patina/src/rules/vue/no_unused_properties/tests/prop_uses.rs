//! Attribute names, loop aliases and member names are not prop uses.
//! Method signatures in a props type are props.

use super::{lint_sfc, owned, unused};

#[test]
fn reports_props_that_only_share_a_spelling_with_something_else() {
    let sfc = r#"<script setup lang="ts">
import { ref } from "vue";

const { title } = defineProps<{
  title: string;
  id?: string;
  label?: string;
  row?: string;
  value?: number;
  unused?: string;
}>();
const count = ref(0);

function bump() {
  count.value += 1;
}
</script>

<template>
  <ul :id="title" :aria-label="title" @click="bump">
    <li v-for="row in [title]" :key="row">{{ row }}</li>
  </ul>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![
            unused(sfc, "id", "id?: string;"),
            unused(sfc, "label", "label?: string;"),
            unused(sfc, "row", "row?: string;"),
            unused(sfc, "value", "value?: number;"),
            unused(sfc, "unused", "unused?: string;"),
        ]
    );
}

#[test]
fn reports_an_unused_method_signature_prop() {
    let sfc = r#"<script setup lang="ts">
const { title } = defineProps<{
  title: string;
  on_open?: () => void;
  on_select?(): void;
}>();
</script>

<template>
  <p>{{ title }}</p>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![
            unused(sfc, "on_open", "on_open?: () => void;"),
            unused(sfc, "on_select", "on_select?(): void;"),
        ]
    );
}

#[test]
fn reports_a_method_signature_on_a_local_interface() {
    let sfc = r#"<script setup lang="ts">
interface Props {
  title: string;
  on_select?(): void;
}
defineProps<Props>();
</script>

<template>{{ title }}</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "on_select", "on_select?(): void;")]
    );
}
