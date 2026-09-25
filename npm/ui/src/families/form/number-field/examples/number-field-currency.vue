<!-- Locale-aware currency spinbutton that formats and parses euros in German notation. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { NumberField, NumberFieldInput, type NumberFieldValue } from "../number-field.ts";

const budget = ref<NumberFieldValue>(1250);
const inputId = useId();
const hintId = useId();
</script>

<template>
  <div>
    <label :for="inputId">Monthly budget</label>
    <NumberField
      :id="inputId"
      v-model="budget"
      name="budget"
      locale="de-DE"
      :format-options="{ style: 'currency', currency: 'EUR' }"
      :min="0"
      :step="50"
      :aria-describedby="hintId"
    >
      <NumberFieldInput />
    </NumberField>
    <p :id="hintId">Use the arrow keys to adjust in steps of 50 €.</p>
  </div>
</template>
