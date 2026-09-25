<!-- Quantity spinbutton bound with v-model, clamped between 1 and 10 with stepper buttons. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  NumberField,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
  type NumberFieldValue,
} from "../number-field.ts";

const quantity = ref<NumberFieldValue>(2);
const inputId = useId();
</script>

<template>
  <div>
    <label :for="inputId">Tickets</label>
    <NumberField :id="inputId" v-model="quantity" name="tickets" :min="1" :max="10" locale="en-US">
      <NumberFieldDecrement aria-label="Remove a ticket">−</NumberFieldDecrement>
      <NumberFieldInput />
      <NumberFieldIncrement aria-label="Add a ticket">+</NumberFieldIncrement>
    </NumberField>
    <output :for="inputId">{{ quantity ?? 0 }} tickets selected</output>
  </div>
</template>
