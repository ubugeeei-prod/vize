import { statusLightRendererFixtures } from "./renderer-fixtures-status-light.ts";

const calloutRendererFixtures = [
  {
    filename: "CalloutConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Callout } from "./families/feedback/callout/callout.ts";

const role = ref<"note" | "status">("status");
</script>

<template>
  <Callout role="status" tone="info" density="compact" aria-label="Sync notice">
    <template #icon="{ tone }">
      <span>{{ tone }}</span>
    </template>
    <template #title="{ role: currentRole }">
      {{ role }} {{ currentRole }}
    </template>
    <template #description="{ live }">
      {{ live }}
    </template>
    <template #default="{ state }">
      <span>{{ state }}</span>
    </template>
    <template #actions>
      <button type="button">Review</button>
    </template>
  </Callout>
</template>
`,
  },
] as const;

export const feedbackRendererFixtures = [
  ...calloutRendererFixtures,
  ...statusLightRendererFixtures,
] as const;
