export const toggleGroupRendererFixtures = [
  {
    filename: "ToggleGroupConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { ToggleGroup, ToggleGroupItem } from "./families/selection/toggle-group/toggle-group.ts";

const value = ref<readonly string[]>(["bold"]);
</script>

<template>
  <ToggleGroup v-model="value" type="multiple" aria-label="Formatting">
    <template #default="{ pressedValues }">
      <output>{{ pressedValues.join(",") }}</output>
      <ToggleGroupItem value="bold">Bold</ToggleGroupItem>
      <ToggleGroupItem value="italic">Italic</ToggleGroupItem>
    </template>
  </ToggleGroup>
</template>
`,
  },
] as const;
