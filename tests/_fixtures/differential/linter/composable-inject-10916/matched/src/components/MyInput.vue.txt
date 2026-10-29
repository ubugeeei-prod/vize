<script setup lang="ts">
import { useFormField as bindField } from "../composables/form";

const value = defineModel<string>({ default: "" });
bindField(() => value.value !== "");
</script>

<template>
  <input v-model="value" aria-label="name" />
</template>
