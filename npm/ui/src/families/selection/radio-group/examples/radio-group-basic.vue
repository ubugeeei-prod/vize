<!-- Controlled radio group bound with v-model, labelled by visible text, with one disabled option. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { RadioGroup, RadioGroupItem } from "../radio-group.ts";

const options = [
  { value: "standard", label: "Standard (5–7 days)", disabled: false },
  { value: "express", label: "Express (2 days)", disabled: false },
  { value: "overnight", label: "Overnight (unavailable)", disabled: true },
];
const shipping = ref<string | null>("standard");
const baseId = useId();
</script>

<template>
  <div>
    <span :id="`${baseId}-label`">Shipping speed</span>
    <RadioGroup v-model="shipping" name="shipping" :aria-labelledby="`${baseId}-label`">
      <div v-for="option in options" :key="option.value">
        <RadioGroupItem
          :id="`${baseId}-${option.value}`"
          :value="option.value"
          :disabled="option.disabled"
        />
        <label :for="`${baseId}-${option.value}`">{{ option.label }}</label>
      </div>
    </RadioGroup>
    <output>{{ shipping ?? "none" }}</output>
  </div>
</template>
