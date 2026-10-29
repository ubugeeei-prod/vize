<script setup lang="ts">
import { provide, shallowRef } from "vue";
import { FormKey } from "../composables/form";

const fields = shallowRef<{ validate(): boolean }[]>([]);
provide(FormKey, {
  register: field => (fields.value = [...fields.value, field]),
  unregister: field => (fields.value = fields.value.filter(f => f !== field))
});
</script>

<template>
  <form><slot /></form>
</template>
