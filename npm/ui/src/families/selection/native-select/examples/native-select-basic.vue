<!-- Native select bound with v-model, rendering options from data with one disabled choice. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { NativeSelect, type NativeSelectOption, type NativeSelectValue } from "../native-select.ts";

const plan = ref<NativeSelectValue>("team");
const selectId = useId();
const plans: readonly NativeSelectOption[] = [
  { value: "starter", label: "Starter — free" },
  { value: "team", label: "Team — $12 per seat" },
  { value: "enterprise", label: "Enterprise — contact sales", disabled: true },
];
</script>

<template>
  <div>
    <label :for="selectId">Billing plan</label>
    <NativeSelect :id="selectId" v-model="plan" name="plan" :options="plans" />
    <output :for="selectId">Selected plan: {{ plan }}</output>
  </div>
</template>
