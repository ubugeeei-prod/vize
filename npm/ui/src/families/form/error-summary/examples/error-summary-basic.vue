<!-- Form that lists invalid fields as links above the form after a failed submit. -->
<script setup lang="ts">
import { ref, shallowRef, useId } from "vue";

import { ErrorSummary, type ErrorSummaryField } from "../error-summary.ts";

const nameId = useId();
const emailId = useId();
const name = ref("");
const email = ref("");
const errors = shallowRef<readonly ErrorSummaryField[]>([]);

function submit(): void {
  const next: ErrorSummaryField[] = [];
  if (name.value.trim() === "")
    next.push({ id: nameId, label: "Full name", message: "Enter your name" });
  if (!email.value.includes("@")) {
    next.push({ id: emailId, label: "Email", message: "Enter a valid email address" });
  }
  errors.value = next;
}
</script>

<template>
  <form novalidate @submit.prevent="submit">
    <ErrorSummary :fields="errors" heading="There is a problem" />
    <label :for="nameId">Full name</label>
    <input
      :id="nameId"
      v-model="name"
      autocomplete="name"
      :aria-invalid="errors.some((field) => field.id === nameId)"
    />
    <label :for="emailId">Email</label>
    <input
      :id="emailId"
      v-model="email"
      type="email"
      autocomplete="email"
      :aria-invalid="errors.some((field) => field.id === emailId)"
    />
    <button type="submit">Create account</button>
  </form>
</template>
