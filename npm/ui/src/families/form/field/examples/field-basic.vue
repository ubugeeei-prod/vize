<!-- Email field whose label, help text, and validation error are wired to the native input. -->
<script setup lang="ts">
import { computed, ref } from "vue";

import { Field, FieldDescription, FieldErrorMessage, FieldLabel } from "../field.ts";

const email = ref("");
const touched = ref(false);
const errors = computed(() =>
  touched.value && !email.value.includes("@")
    ? [{ name: "email", message: "Enter an email address like name@example.com", path: ["email"] }]
    : [],
);

function markTouched(): void {
  touched.value = true;
}
</script>

<template>
  <Field v-slot="{ fieldProps }" name="email" :errors has-description>
    <FieldLabel>Work email</FieldLabel>
    <input
      v-bind="fieldProps"
      v-model="email"
      :aria-labelledby="fieldProps['aria-labelledby']"
      name="email"
      type="email"
      autocomplete="email"
      @blur="markTouched"
    />
    <FieldDescription>We send receipts to this address.</FieldDescription>
    <FieldErrorMessage />
  </Field>
</template>
