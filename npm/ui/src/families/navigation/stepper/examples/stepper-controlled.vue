<!-- Controlled stepper driven by v-model with Back and Next buttons. -->
<script setup lang="ts">
import { computed, ref } from "vue";

import {
  StepperContent,
  StepperItem,
  StepperList,
  StepperRoot,
  StepperTrigger,
} from "../stepper.ts";
import type { StepperValue } from "../stepper.ts";

const steps = ["account", "profile", "done"] as const;
const labels = { account: "Account", profile: "Profile", done: "Done" } as const;
const current = ref<StepperValue>("account");
const index = computed(() => steps.findIndex((step) => step === current.value));

function go(offset: number): void {
  const next = steps[index.value + offset];
  if (next !== undefined) current.value = next;
}
</script>

<template>
  <StepperRoot v-model="current">
    <StepperList aria-label="Sign-up steps">
      <StepperItem
        v-for="(step, position) in steps"
        :key="step"
        :value="step"
        :text-value="labels[step]"
        :completed="position < index"
      >
        <StepperTrigger>{{ labels[step] }}</StepperTrigger>
      </StepperItem>
    </StepperList>
    <StepperContent v-for="step in steps" :key="step" :value="step">
      {{ labels[step] }} details
    </StepperContent>
    <button type="button" :disabled="index <= 0" @click="() => go(-1)">Back</button>
    <button type="button" :disabled="index >= steps.length - 1" @click="() => go(1)">Next</button>
  </StepperRoot>
</template>
