export const statusLightRendererFixtures = [
  {
    filename: "StatusLightConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { StatusLight } from "./families/feedback/status-light/status-light.ts";

const state = ref<"away" | "busy" | "offline" | "online" | "unknown">("online");
</script>

<template>
  <StatusLight aria-label="Service status" :state="state" size="sm" tone="success">
    <template #default="{ state: currentState, tone }">
      <span>{{ currentState }} {{ tone }}</span>
    </template>
  </StatusLight>
</template>
`,
  },
] as const;
