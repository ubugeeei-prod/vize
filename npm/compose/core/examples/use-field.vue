<script setup lang="ts">
import { useId } from "vue";
import { useField } from "@vizejs/composable/use-field";

const fieldId = useId();
const { value, error, dirty, touched, onBlur, reset } = useField("", {
  name: "displayName",
  validateOn: "blur",
  rules: (name) => (name.trim().length >= 3 ? undefined : "Use at least three characters."),
});
</script>

<template>
  <div class="composable-example">
    <label :for="fieldId">Display name</label>
    <input
      :id="fieldId"
      v-model="value"
      autocomplete="nickname"
      :aria-invalid="Boolean(error)"
      :aria-describedby="`${fieldId}-help ${fieldId}-error`"
      @blur="onBlur"
    />
    <p :id="`${fieldId}-help`">
      Use at least three characters. Validation runs when you leave the field. Further edits wait
      for the next blur before validation.
    </p>
    <p :id="`${fieldId}-error`" role="status">
      {{ error || (touched ? "No recorded validation error." : "Enter a display name.") }}
    </p>
    <output>Changed: {{ dirty ? "yes" : "no" }} · Visited: {{ touched ? "yes" : "no" }}</output>
    <button type="button" @click="() => reset()">Reset name</button>
  </div>
</template>
