<!-- Multiple-selection formatting toggle group bound with v-model. -->
<script setup lang="ts">
import { computed, ref } from "vue";

import { ToggleGroup, ToggleGroupItem } from "../toggle-group.ts";
import type { ToggleGroupValue } from "../toggle-group.ts";

const formatting = ref<ToggleGroupValue>(["bold"]);
const summary = computed(() =>
  Array.isArray(formatting.value) && formatting.value.length > 0
    ? formatting.value.join(", ")
    : "none",
);
</script>

<template>
  <div>
    <ToggleGroup v-model="formatting" type="multiple" aria-label="Text formatting">
      <ToggleGroupItem value="bold">Bold</ToggleGroupItem>
      <ToggleGroupItem value="italic">Italic</ToggleGroupItem>
      <ToggleGroupItem value="underline">Underline</ToggleGroupItem>
    </ToggleGroup>
    <output>Active: {{ summary }}</output>
  </div>
</template>
